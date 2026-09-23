use super::*;

mod args;
mod html;
mod paths;
mod shell;
mod text_shape;
mod url;

pub(crate) use args::eval_parse_args;
pub(crate) use html::{eval_html_escape, eval_html_unescape};
pub(crate) use paths::{eval_file_base_name, eval_file_extension, eval_file_name_split};
pub(crate) use shell::{eval_shell_quote, eval_shell_split};
pub(crate) use text_shape::{
    eval_align_columns, eval_string_dedent, eval_string_expand_tabs, eval_string_indent,
    eval_string_wrap,
};
pub(crate) use url::{eval_url_decode, eval_url_encode, eval_url_parse};

fn failed_with_message(
    session: &mut Session,
    symbol: &str,
    tag: &str,
    message: impl Into<String>,
) -> Expr {
    session.push_message(symbol, tag, message);
    system_expr("$Failed")
}

pub(crate) use html::eval_html_unescape_with_options;

pub(crate) use args::eval_parse_args_with_options;

pub(crate) use url::{
    eval_url_decode_with_options, eval_url_encode_with_options, eval_url_parse_with_options,
};

pub(crate) use text_shape::{
    eval_align_columns_with_options, eval_string_indent_with_options, eval_string_wrap_with_options,
};
