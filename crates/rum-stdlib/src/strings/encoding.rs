use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[cfg(feature = "hash")]
use crc32fast::Hasher as Crc32Hasher;
#[cfg(feature = "hash")]
use md5::Md5;
#[cfg(feature = "hash")]
use sha1::Sha1;
#[cfg(feature = "hash")]
use sha2::{Digest, Sha256, Sha512};

pub(crate) fn eval_string_to_byte_array(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=2).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringToByteArray", args, 1, 2));
    }

    if let Some(encoding) = args.get(1).and_then(expr_as_string) {
        if !matches!(encoding, "UTF8" | "UTF-8") {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
    } else if args.len() == 2 {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(byte_array_expr(value.as_bytes()));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| byte_array_expr(value.as_bytes()))
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

#[cfg(feature = "hash")]
pub(crate) fn eval_hash(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=3).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("Hash", args, 1, 3));
    }

    let algorithm = if let Some(algorithm_expr) = args.get(1) {
        let Some(name) = expr_as_string(algorithm_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(algorithm) = HashAlgorithm::from_name(name) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        algorithm
    } else {
        HashAlgorithm::Sha256
    };

    let digest = hash_digest_bytes(algorithm, &hash_input_bytes(&args[0]));
    let integer_value = digest_integer(&digest);

    match args.get(2).and_then(|format| expr_as_string(format)) {
        None => Ok(Expr::integer(integer_value)),
        Some("Integer") => Ok(Expr::integer(integer_value)),
        Some("HexString") => Ok(Expr::string(bytes_to_hex(&digest))),
        Some("DecimalString") => Ok(Expr::string(digest_decimal_string(
            &integer_value,
            algorithm.output_len(),
        ))),
        Some("ByteArray") => Ok(byte_array_expr(&digest)),
        Some(_) => Ok(Expr::call(head.clone(), args.to_vec())),
    }
}

pub(crate) fn eval_compress(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count("Compress", args, 1));
    }

    Ok(Expr::string(compress_envelope(&args[0])))
}

pub(crate) fn eval_uncompress(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.is_empty() {
        return Ok(session.invalid_call_with_arg_range("Uncompress", args, 1, 2));
    }

    let Some(source) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let (wrapper, option_args) = uncompress_wrapper_and_options(&args[1..]);
    let Some(strict) = parse_strict_option(session, "Uncompress", option_args)? else {
        return Ok(session.invalid_call_with_arg_range("Uncompress", args, 1, 2));
    };
    let Some((payload, original_form)) = parse_compress_envelope(source) else {
        return Ok(invalid_uncompress_envelope(session, head, args, strict));
    };
    let Some(restored) = session.restore_transport_expr(&payload, original_form) else {
        return Ok(invalid_uncompress_envelope(session, head, args, strict));
    };

    if let Some(wrapper) = wrapper {
        let wrapped_result = session.evaluate(Expr::call(wrapper.clone(), vec![restored]));
        return rum_runtime::resume_or_suspend!(
            session,
            wrapped_result,
            ok(value) => Ok(value),
            suspend => EvalUncompressSuspensionFrame,
        );
    }

    Ok(restored)
}

fn uncompress_wrapper_and_options(args: &[Expr]) -> (Option<&Expr>, &[Expr]) {
    match args {
        [] => (None, &[]),
        [option, ..] if is_strict_option_rule(option) => (None, args),
        [wrapper, rest @ ..] => (Some(wrapper), rest),
    }
}

fn invalid_uncompress_envelope(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    strict: bool,
) -> Expr {
    if strict {
        session.push_message(
            "Uncompress",
            "env",
            "Uncompress found a malformed or corrupt compressed string envelope.",
        );
        system_expr("$Failed")
    } else {
        Expr::call(head.clone(), args.to_vec())
    }
}

fn byte_array_expr(bytes: &[u8]) -> Expr {
    Expr::call(
        system_expr("ByteArray"),
        vec![Expr::string(base64_encode(bytes))],
    )
}

#[cfg(feature = "hash")]
fn hash_input_bytes(expr: &Expr) -> Vec<u8> {
    if let Some(value) = expr_as_string(expr) {
        value.as_bytes().to_vec()
    } else if let Some(bytes) = expr_as_byte_array_bytes(expr) {
        bytes
    } else {
        render_input_like_expr(expr).into_bytes()
    }
}

#[cfg(feature = "hash")]
#[derive(Clone, Copy)]
enum HashAlgorithm {
    Sha256,
    Sha1,
    Sha512,
    Md5,
    Crc32,
}

#[cfg(feature = "hash")]
impl HashAlgorithm {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "SHA256" => Some(Self::Sha256),
            "SHA1" => Some(Self::Sha1),
            "SHA512" => Some(Self::Sha512),
            "MD5" => Some(Self::Md5),
            "CRC32" => Some(Self::Crc32),
            _ => None,
        }
    }

    fn output_len(self) -> usize {
        match self {
            Self::Sha256 => 32,
            Self::Sha1 => 20,
            Self::Sha512 => 64,
            Self::Md5 => 16,
            Self::Crc32 => 4,
        }
    }
}

#[cfg(feature = "hash")]
fn hash_digest_bytes(algorithm: HashAlgorithm, bytes: &[u8]) -> Vec<u8> {
    match algorithm {
        HashAlgorithm::Sha256 => Sha256::digest(bytes).to_vec(),
        HashAlgorithm::Sha1 => Sha1::digest(bytes).to_vec(),
        HashAlgorithm::Sha512 => Sha512::digest(bytes).to_vec(),
        HashAlgorithm::Md5 => Md5::digest(bytes).to_vec(),
        HashAlgorithm::Crc32 => {
            let mut hasher = Crc32Hasher::new();
            hasher.update(bytes);
            hasher.finalize().to_be_bytes().to_vec()
        }
    }
}

#[cfg(feature = "hash")]
fn digest_integer(digest: &[u8]) -> BigInt {
    BigInt::parse_bytes(bytes_to_hex(digest).as_bytes(), 16)
        .expect("digest bytes always parse as hexadecimal integers")
}

#[cfg(feature = "hash")]
fn digest_decimal_string(value: &BigInt, digest_len: usize) -> String {
    let decimal = value.to_str_radix(10);
    let width = decimal_width_for_digest_len(digest_len);
    format!("{decimal:0>width$}")
}

#[cfg(feature = "hash")]
fn decimal_width_for_digest_len(digest_len: usize) -> usize {
    match digest_len {
        4 => 10,
        16 => 39,
        20 => 49,
        32 => 78,
        64 => 155,
        _ => 0,
    }
}

fn compress_envelope(expr: &Expr) -> String {
    let (payload, original_form) = expr_transport_bytes(expr);
    let kind = if is_system_symbol_name(&original_form, "String") {
        "s"
    } else if is_system_symbol_name(&original_form, "ByteArray") {
        "b"
    } else {
        "e"
    };
    format!("rum:1:{kind}:{}", base64_encode(&payload))
}

fn parse_compress_envelope(source: &str) -> Option<(Vec<u8>, Expr)> {
    let rest = source.strip_prefix("rum:1:")?;
    let (kind, encoded) = rest.split_once(':')?;
    let payload = base64_decode(encoded)?;
    let original_form = match kind {
        "s" => system_expr("String"),
        "b" => system_expr("ByteArray"),
        "e" => system_expr("InputForm"),
        _ => return None,
    };
    Some((payload, original_form))
}

fn expr_transport_bytes(expr: &Expr) -> (Vec<u8>, Expr) {
    if let Some(value) = expr_as_string(expr) {
        (value.as_bytes().to_vec(), system_expr("String"))
    } else if let Some(bytes) = expr_as_byte_array_bytes(expr) {
        (bytes, system_expr("ByteArray"))
    } else {
        (
            render_input_like_expr(expr).into_bytes(),
            system_expr("InputForm"),
        )
    }
}

fn expr_as_byte_array_bytes(expr: &Expr) -> Option<Vec<u8>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "ByteArray") || args.len() != 1 {
        return None;
    }
    let encoded = expr_as_string(&args[0])?;
    base64_decode(encoded)
}

fn base64_decode(encoded: &str) -> Option<Vec<u8>> {
    let bytes = encoded.as_bytes();
    if bytes.len() % 4 != 0 {
        return None;
    }

    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);
    let mut index = 0;
    while index < bytes.len() {
        let a = decode_base64_nibble(bytes[index])?;
        let b = decode_base64_nibble(bytes[index + 1])?;
        let c = bytes[index + 2];
        let d = bytes[index + 3];

        let c_value = if c == b'=' {
            0
        } else {
            decode_base64_nibble(c)?
        };
        let d_value = if d == b'=' {
            0
        } else {
            decode_base64_nibble(d)?
        };

        let chunk =
            ((a as u32) << 18) | ((b as u32) << 12) | ((c_value as u32) << 6) | (d_value as u32);

        output.push(((chunk >> 16) & 0xff) as u8);
        if c != b'=' {
            output.push(((chunk >> 8) & 0xff) as u8);
        }
        if d != b'=' {
            output.push((chunk & 0xff) as u8);
        }

        index += 4;
    }

    Some(output)
}

fn decode_base64_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

#[cfg(feature = "hash")]
fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        rendered.push(nibble_to_hex(byte >> 4));
        rendered.push(nibble_to_hex(byte & 0x0f));
    }
    rendered
}

#[cfg(feature = "hash")]
fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + (nibble - 10)),
        _ => unreachable!("nibbles are always in 0..=15"),
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct EvalUncompressSuspensionFrame;

impl rum_runtime::kernel_api::SuspensionFrame for EvalUncompressSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
