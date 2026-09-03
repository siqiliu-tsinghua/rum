use rum_core::{BigInt, Expr};
use rum_runtime::kernel_api::{
    EntropyError, EvalError, Session, ToPrimitive, expr_as_exact_integer, expr_as_string,
    is_system_symbol_name, list_items, system_expr,
};

use super::{
    DEFAULT_RANDOM_ALPHABET, alphabet_chars, entropy_usize_below, random_string_projected_bytes,
};

const RANDOM_PASSWORD_POLICY_ATTEMPTS: usize = 10_000;

#[derive(Debug)]
struct PasswordPolicy {
    required_groups: Vec<Vec<char>>,
    no_adjacent_repeats: bool,
    exclude_needles: Vec<String>,
    ignore_case: bool,
}

#[derive(Debug)]
struct PasswordPolicyCall<'a> {
    alphabet: Vec<char>,
    count: usize,
    options: &'a [Expr],
}

enum PasswordPolicyCallParse<'a> {
    Parsed(PasswordPolicyCall<'a>),
    Inert,
    Failed(String),
}

#[derive(Default)]
struct PasswordOptionSeen {
    require: bool,
    no_adjacent_repeats: bool,
    exclude: bool,
    ignore_case: bool,
}

pub(super) fn eval_random_password_with_policy(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let call = match parse_policy_call(session, args)? {
        PasswordPolicyCallParse::Parsed(call) => call,
        PasswordPolicyCallParse::Inert => return Ok(Expr::call(head.clone(), args.to_vec())),
        PasswordPolicyCallParse::Failed(message) => return Ok(fail_password(session, message)),
    };

    let policy = match parse_policy(&call.alphabet, call.count, call.options) {
        Ok(policy) => policy,
        Err(message) => return Ok(fail_password(session, message)),
    };

    for _ in 0..RANDOM_PASSWORD_POLICY_ATTEMPTS {
        let candidate = match draw_candidate(session, &call.alphabet, call.count) {
            Ok(candidate) => candidate,
            Err(error) => {
                super::push_entropy_error(session, "RandomPassword", error.message());
                return Ok(system_expr("$Failed"));
            }
        };
        if policy.accepts(&candidate) {
            return Ok(Expr::string(candidate));
        }
    }

    Ok(fail_password(
        session,
        format!(
            "could not satisfy the password policy within {RANDOM_PASSWORD_POLICY_ATTEMPTS} attempts; loosen the constraints or increase length"
        ),
    ))
}

fn parse_policy_call<'a>(
    session: &Session,
    args: &'a [Expr],
) -> Result<PasswordPolicyCallParse<'a>, EvalError> {
    let (alphabet, count, options) = match args {
        [alphabet, count, options @ ..] if expr_as_string(alphabet).is_some() => {
            let alphabet = expr_as_string(alphabet).expect("pattern checked string alphabet");
            (alphabet, count, options)
        }
        [count, options @ ..] => (DEFAULT_RANDOM_ALPHABET, count, options),
        _ => return Ok(PasswordPolicyCallParse::Inert),
    };

    let Some(alphabet) = alphabet_chars(alphabet) else {
        return Ok(PasswordPolicyCallParse::Inert);
    };
    let Some(count) = expr_as_exact_integer(count) else {
        return Ok(PasswordPolicyCallParse::Inert);
    };
    if count < BigInt::from(1_u8) {
        return Ok(PasswordPolicyCallParse::Failed(
            "password policy requires a positive length".to_string(),
        ));
    }
    session.enforce_generated_element_limit_for_big_count(&count)?;
    let count = count
        .to_usize()
        .expect("generated element limit keeps count representable");
    session.enforce_active_memory_limit_for_projected_bytes(random_string_projected_bytes(
        &alphabet, count,
    ))?;

    Ok(PasswordPolicyCallParse::Parsed(PasswordPolicyCall {
        alphabet,
        count,
        options,
    }))
}

fn parse_policy(
    alphabet: &[char],
    count: usize,
    options: &[Expr],
) -> Result<PasswordPolicy, String> {
    let mut seen = PasswordOptionSeen::default();
    let mut required_groups = Vec::new();
    let mut no_adjacent_repeats = false;
    let mut excludes = Vec::new();
    let mut ignore_case = false;

    for option in options {
        let Some((name, value)) = option_rule(option) else {
            return Err(
                "RandomPassword policy options must be string-keyed rules such as \"Require\" -> {\"ABC\"}."
                    .to_string(),
            );
        };

        match name {
            "Require" => {
                reject_duplicate_option(&mut seen.require, name)?;
                required_groups = parse_required_groups(alphabet, value)?;
            }
            "NoAdjacentRepeats" => {
                reject_duplicate_option(&mut seen.no_adjacent_repeats, name)?;
                no_adjacent_repeats = parse_bool_option(name, value)?;
            }
            "Exclude" => {
                reject_duplicate_option(&mut seen.exclude, name)?;
                excludes = parse_excludes(value)?;
            }
            "IgnoreCase" => {
                reject_duplicate_option(&mut seen.ignore_case, name)?;
                ignore_case = parse_bool_option(name, value)?;
            }
            other => {
                return Err(format!("unknown RandomPassword policy option \"{other}\"."));
            }
        }
    }

    if required_groups.len() > count {
        return Err(format!(
            "password policy has {} required group(s), which exceeds length {count}.",
            required_groups.len()
        ));
    }

    let exclude_needles = if ignore_case {
        excludes
            .into_iter()
            .map(|needle| needle.to_lowercase())
            .collect()
    } else {
        excludes
    };

    Ok(PasswordPolicy {
        required_groups,
        no_adjacent_repeats,
        exclude_needles,
        ignore_case,
    })
}

fn option_rule(expr: &Expr) -> Option<(&str, &Expr)> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    let name = expr_as_string(&args[0])?;
    Some((name, &args[1]))
}

fn reject_duplicate_option(seen: &mut bool, name: &str) -> Result<(), String> {
    if *seen {
        return Err(format!(
            "RandomPassword policy option \"{name}\" was provided more than once."
        ));
    }
    *seen = true;
    Ok(())
}

fn parse_required_groups(alphabet: &[char], value: &Expr) -> Result<Vec<Vec<char>>, String> {
    let Some(items) = list_items(value) else {
        return Err("RandomPassword option \"Require\" must be a list of strings.".to_string());
    };

    let mut groups = Vec::with_capacity(items.len());
    for item in items {
        let Some(group) = expr_as_string(item) else {
            return Err("RandomPassword option \"Require\" must be a list of strings.".to_string());
        };
        if group.is_empty() {
            return Err("RandomPassword required groups cannot be empty strings.".to_string());
        }
        if group.chars().any(|ch| !alphabet.contains(&ch)) {
            return Err(format!(
                "RandomPassword required group {group:?} is not a subset of the alphabet."
            ));
        }
        groups.push(group.chars().collect());
    }
    Ok(groups)
}

fn parse_excludes(value: &Expr) -> Result<Vec<String>, String> {
    let Some(items) = list_items(value) else {
        return Err("RandomPassword option \"Exclude\" must be a list of strings.".to_string());
    };

    let mut excludes = Vec::with_capacity(items.len());
    for item in items {
        let Some(needle) = expr_as_string(item) else {
            return Err("RandomPassword option \"Exclude\" must be a list of strings.".to_string());
        };
        if needle.is_empty() {
            return Err("RandomPassword excluded substrings cannot be empty strings.".to_string());
        }
        excludes.push(needle.to_string());
    }
    Ok(excludes)
}

fn parse_bool_option(name: &str, value: &Expr) -> Result<bool, String> {
    if is_system_symbol_name(value, "True") {
        Ok(true)
    } else if is_system_symbol_name(value, "False") {
        Ok(false)
    } else {
        Err(format!(
            "RandomPassword option \"{name}\" must be True or False."
        ))
    }
}

fn draw_candidate(
    session: &Session,
    alphabet: &[char],
    count: usize,
) -> Result<String, EntropyError> {
    let mut out = String::with_capacity(random_string_projected_bytes(alphabet, count));
    for _ in 0..count {
        let index = entropy_usize_below(session, alphabet.len())?;
        out.push(alphabet[index]);
    }
    Ok(out)
}

fn fail_password(session: &mut Session, message: impl Into<String>) -> Expr {
    session.push_message("RandomPassword", "password", message);
    system_expr("$Failed")
}

impl PasswordPolicy {
    fn accepts(&self, candidate: &str) -> bool {
        if self.no_adjacent_repeats && has_adjacent_repeat(candidate) {
            return false;
        }

        let chars = candidate.chars().collect::<Vec<_>>();
        if self
            .required_groups
            .iter()
            .any(|group| !chars.iter().any(|ch| group.contains(ch)))
        {
            return false;
        }

        if self.exclude_needles.is_empty() {
            return true;
        }
        if self.ignore_case {
            let folded = candidate.to_lowercase();
            !self
                .exclude_needles
                .iter()
                .any(|needle| folded.contains(needle))
        } else {
            !self
                .exclude_needles
                .iter()
                .any(|needle| candidate.contains(needle))
        }
    }
}

fn has_adjacent_repeat(candidate: &str) -> bool {
    let mut previous = None;
    for ch in candidate.chars() {
        if previous == Some(ch) {
            return true;
        }
        previous = Some(ch);
    }
    false
}
