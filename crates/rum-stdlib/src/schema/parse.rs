//! Parse only the current container/entry, preserving malformed-schema precedence.
use super::format::{malformed, render};
use super::state::{EntrySpec, SchemaAbort, SchemaEntry, SchemaOptions, SchemaPath};
use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::{
    association_entries_from_expr, expr_as_string, expr_as_symbol, is_system_symbol_name,
    list_items,
};

impl EntrySpec<'_> {
    pub(super) fn validation_spec(&self) -> &Expr {
        match self {
            Self::Required(spec) | Self::Optional { spec, .. } => spec,
        }
    }
}

pub(super) fn parse_schema_call<'a>(
    symbol: &str,
    value: Option<&'a Expr>,
    tail: &'a [Expr],
) -> Result<(&'a Expr, &'a Expr, SchemaOptions), String> {
    let (Some(value), [schema, option_exprs @ ..]) = (value, tail) else {
        return Err(format!(
            "{symbol} expects a value, a schema, and optional option rules."
        ));
    };

    let options = parse_options(option_exprs)?;
    Ok((value, schema, options))
}

fn parse_options(option_exprs: &[Expr]) -> Result<SchemaOptions, String> {
    let mut options = SchemaOptions::default();
    for option in option_exprs {
        let Some((name, value)) = option_rule(option) else {
            return Err(format!("Invalid schema option {}.", render(option)));
        };
        if name != "AllowExtraKeys" {
            return Err(format!("Unknown schema option {name}."));
        }

        if is_system_symbol_name(value, "True") {
            options.allow_extra_keys = true;
        } else if is_system_symbol_name(value, "False") {
            options.allow_extra_keys = false;
        } else {
            return Err("AllowExtraKeys must be True or False.".to_string());
        }
    }
    Ok(options)
}

fn option_rule(expr: &Expr) -> Option<(&str, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    let name = expr_as_string(&args[0])
        .or_else(|| expr_as_symbol(&args[0]).map(|symbol| symbol.name()))?;
    Some((name, &args[1]))
}

pub(super) fn parse_association_schema(
    schema: &Expr,
    path: &SchemaPath,
) -> Result<Vec<SchemaEntry>, SchemaAbort> {
    let entries = association_entries_from_expr(schema)
        .ok_or_else(|| malformed(path, "association schema entries must be rules"))?;

    entries
        .into_iter()
        .map(|(key, spec)| {
            let Expr::Atom(Atom::String(key)) = key else {
                return Err(malformed(
                    path,
                    format!(
                        "association schema keys must be Strings; got {}",
                        render(&key)
                    ),
                ));
            };
            Ok(SchemaEntry { key, spec })
        })
        .collect()
}

pub(super) fn parse_list_schema<'a>(
    schema: &'a Expr,
    path: &SchemaPath,
) -> Result<&'a Expr, SchemaAbort> {
    let Some(items) = list_items(schema) else {
        return Err(malformed(path, "list schema must use List"));
    };
    match items {
        [item] => Ok(item),
        _ => Err(malformed(
            path,
            format!(
                "list schema must contain exactly one element; got {}",
                items.len()
            ),
        )),
    }
}

pub(super) fn parse_entry_spec<'a>(
    spec: &'a Expr,
    path: &SchemaPath,
) -> Result<EntrySpec<'a>, SchemaAbort> {
    let Expr::Call { head, args } = spec else {
        return Ok(EntrySpec::Required(spec));
    };
    if !is_system_symbol_name(head, "Optional") {
        return Ok(EntrySpec::Required(spec));
    }

    match args.as_slice() {
        [inner] => Ok(EntrySpec::Optional {
            spec: inner,
            default: None,
        }),
        [inner, default] => Ok(EntrySpec::Optional {
            spec: inner,
            default: Some(default),
        }),
        _ => Err(malformed(
            path,
            format!(
                "Optional schema must have one or two arguments; got {}",
                args.len()
            ),
        )),
    }
}
