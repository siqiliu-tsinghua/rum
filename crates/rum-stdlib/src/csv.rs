use rum_core::SharedArgs;
use std::collections::{HashMap, HashSet};

use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::{
    EvalError, Session, association_entries_from_expr, expr_as_string, expr_as_symbol,
    is_system_symbol_name, list_items, system_expr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FromCsvConfig {
    delimiter: char,
    headers: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ToCsvConfig {
    delimiter: char,
    headers: Option<bool>,
}

impl Default for FromCsvConfig {
    fn default() -> Self {
        Self {
            delimiter: ',',
            headers: false,
        }
    }
}

impl Default for ToCsvConfig {
    fn default() -> Self {
        Self {
            delimiter: ',',
            headers: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CsvParseError {
    offset: usize,
    message: String,
}

impl CsvParseError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }

    fn message(&self) -> String {
        format!(
            "Invalid CSV at byte offset {}: {}.",
            self.offset, self.message
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CsvEncodeError {
    message: String,
}

impl CsvEncodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CsvField {
    value: String,
    offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CsvRow {
    fields: Vec<CsvField>,
    offset: usize,
}

pub(crate) fn eval_from_csv(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (source, config) = match parse_from_csv_args(args) {
        Ok(parsed) => parsed,
        Err(message) => return Ok(fail_csv(session, "FromCsv", message)),
    };

    match from_csv(source, config) {
        Ok(expr) => Ok(expr),
        Err(error) => Ok(fail_csv(session, "FromCsv", error.message())),
    }
}

pub(crate) fn eval_to_csv(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (rows, config) = match parse_to_csv_args(args) {
        Ok(parsed) => parsed,
        Err(message) => return Ok(fail_csv(session, "ToCsv", message)),
    };

    match to_csv(rows, config) {
        Ok(output) => Ok(Expr::string(output)),
        Err(error) => Ok(fail_csv(session, "ToCsv", error.message())),
    }
}

fn parse_from_csv_args(args: &[Expr]) -> Result<(&str, FromCsvConfig), String> {
    let Some((source, options)) = args.split_first() else {
        return Err(
            "FromCsv expects a CSV string and optional \"Delimiter\"/\"Headers\" rules."
                .to_string(),
        );
    };
    let Some(source) = expr_as_string(source) else {
        return Err("FromCsv expects its first argument to be a String.".to_string());
    };

    let mut config = FromCsvConfig::default();
    let mut seen = CsvOptionSeen::default();
    for option in options {
        apply_csv_option("FromCsv", option, &mut seen, |name, value| match name {
            "Delimiter" => {
                config.delimiter = parse_delimiter("FromCsv", value)?;
                Ok(())
            }
            "Headers" => {
                config.headers = parse_headers_option("FromCsv", value)?;
                Ok(())
            }
            _ => Err(format!("Unknown FromCsv option {name}.")),
        })?;
    }

    Ok((source, config))
}

fn parse_to_csv_args(args: &[Expr]) -> Result<(&Expr, ToCsvConfig), String> {
    let Some((rows, options)) = args.split_first() else {
        return Err("ToCsv expects rows and optional \"Delimiter\"/\"Headers\" rules.".to_string());
    };

    let mut config = ToCsvConfig::default();
    let mut seen = CsvOptionSeen::default();
    for option in options {
        apply_csv_option("ToCsv", option, &mut seen, |name, value| match name {
            "Delimiter" => {
                config.delimiter = parse_delimiter("ToCsv", value)?;
                Ok(())
            }
            "Headers" => {
                config.headers = Some(parse_headers_option("ToCsv", value)?);
                Ok(())
            }
            _ => Err(format!("Unknown ToCsv option {name}.")),
        })?;
    }

    Ok((rows, config))
}

#[derive(Default)]
struct CsvOptionSeen {
    delimiter: bool,
    headers: bool,
}

fn apply_csv_option(
    symbol: &str,
    option: &Expr,
    seen: &mut CsvOptionSeen,
    mut apply: impl FnMut(&str, &Expr) -> Result<(), String>,
) -> Result<(), String> {
    let Some((name, value)) = option_rule(option) else {
        return Err(format!(
            "{symbol} options must be rules such as \"Delimiter\" -> \",\"."
        ));
    };

    match name {
        "Delimiter" if seen.delimiter => {
            return Err(format!(
                "{symbol} option \"Delimiter\" was provided more than once."
            ));
        }
        "Delimiter" => seen.delimiter = true,
        "Headers" if seen.headers => {
            return Err(format!(
                "{symbol} option \"Headers\" was provided more than once."
            ));
        }
        "Headers" => seen.headers = true,
        _ => {}
    }

    apply(name, value)
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

    let name = expr_as_string(&args[0])
        .or_else(|| expr_as_symbol(&args[0]).map(|symbol| symbol.name()))?;
    Some((name, &args[1]))
}

fn parse_delimiter(symbol: &str, value: &Expr) -> Result<char, String> {
    let Some(value) = expr_as_string(value) else {
        return Err(format!("{symbol} option \"Delimiter\" must be a String."));
    };
    let mut chars = value.chars();
    let Some(delimiter) = chars.next() else {
        return Err(format!(
            "{symbol} option \"Delimiter\" must be a single character."
        ));
    };
    if chars.next().is_some() {
        return Err(format!(
            "{symbol} option \"Delimiter\" must be a single character."
        ));
    }
    if matches!(delimiter, '"' | '\n' | '\r') {
        return Err(format!(
            "{symbol} option \"Delimiter\" cannot be a quote or newline."
        ));
    }
    Ok(delimiter)
}

fn parse_headers_option(symbol: &str, value: &Expr) -> Result<bool, String> {
    if is_system_symbol_name(value, "True") {
        Ok(true)
    } else if is_system_symbol_name(value, "False") {
        Ok(false)
    } else {
        Err(format!(
            "{symbol} option \"Headers\" must be True or False."
        ))
    }
}

fn from_csv(source: &str, config: FromCsvConfig) -> Result<Expr, CsvParseError> {
    let rows = CsvParser::new(source, config.delimiter).parse()?;
    if config.headers {
        rows_with_headers_expr(&rows)
    } else {
        Ok(rows_expr(&rows))
    }
}

struct CsvParser<'a> {
    source: &'a str,
    pos: usize,
    delimiter: char,
}

impl<'a> CsvParser<'a> {
    fn new(source: &'a str, delimiter: char) -> Self {
        Self {
            source,
            pos: 0,
            delimiter,
        }
    }

    fn parse(mut self) -> Result<Vec<CsvRow>, CsvParseError> {
        let mut rows = Vec::new();
        while !self.is_eof() {
            if self.at_newline() {
                let offset = self.pos;
                self.consume_newline();
                rows.push(CsvRow {
                    fields: Vec::new(),
                    offset,
                });
                continue;
            }

            rows.push(self.parse_row()?);
        }
        Ok(rows)
    }

    fn parse_row(&mut self) -> Result<CsvRow, CsvParseError> {
        let offset = self.pos;
        let mut fields = Vec::new();

        loop {
            fields.push(self.parse_field()?);

            if self.is_eof() {
                break;
            }
            if self.at_delimiter() {
                self.consume_delimiter();
                continue;
            }
            if self.at_newline() {
                self.consume_newline();
                break;
            }
            if self.peek_char() == Some('\r') {
                return Err(CsvParseError::new(
                    self.pos,
                    "bare carriage return outside a quoted field",
                ));
            }
            return Err(CsvParseError::new(
                self.pos,
                "expected delimiter or newline after CSV field",
            ));
        }

        Ok(CsvRow { fields, offset })
    }

    fn parse_field(&mut self) -> Result<CsvField, CsvParseError> {
        let offset = self.pos;
        if self.peek_char() == Some('"') {
            return self.parse_quoted_field(offset);
        }

        let mut value = String::new();
        while !self.is_eof() && !self.at_delimiter() && !self.at_newline() {
            let ch = self
                .peek_char()
                .expect("non-EOF parser position should have a char");
            if ch == '"' {
                return Err(CsvParseError::new(self.pos, "quote in an unquoted field"));
            }
            if ch == '\r' {
                return Err(CsvParseError::new(
                    self.pos,
                    "bare carriage return outside a quoted field",
                ));
            }
            self.bump_char();
            value.push(ch);
        }

        Ok(CsvField { value, offset })
    }

    fn parse_quoted_field(&mut self, offset: usize) -> Result<CsvField, CsvParseError> {
        self.bump_char();
        let mut value = String::new();

        loop {
            let Some(ch) = self.peek_char() else {
                return Err(CsvParseError::new(offset, "unterminated quoted field"));
            };

            if ch == '"' {
                self.bump_char();
                if self.peek_char() == Some('"') {
                    self.bump_char();
                    value.push('"');
                    continue;
                }
                break;
            }

            self.bump_char();
            value.push(ch);
        }

        if self.is_eof() || self.at_delimiter() || self.at_newline() {
            Ok(CsvField { value, offset })
        } else {
            Err(CsvParseError::new(
                self.pos,
                "characters after a closing quote before the next delimiter or newline",
            ))
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn at_delimiter(&self) -> bool {
        self.peek_char() == Some(self.delimiter)
    }

    fn consume_delimiter(&mut self) {
        self.pos += self.delimiter.len_utf8();
    }

    fn at_newline(&self) -> bool {
        self.source[self.pos..].starts_with("\r\n") || self.source[self.pos..].starts_with('\n')
    }

    fn consume_newline(&mut self) {
        if self.source[self.pos..].starts_with("\r\n") {
            self.pos += 2;
        } else {
            self.pos += 1;
        }
    }
}

fn rows_expr(rows: &[CsvRow]) -> Expr {
    Expr::call(
        system_expr("List"),
        rows.iter()
            .map(|row| {
                Expr::call(
                    system_expr("List"),
                    row.fields
                        .iter()
                        .map(|field| Expr::string(field.value.clone()))
                        .collect(),
                )
            })
            .collect(),
    )
}

fn rows_with_headers_expr(rows: &[CsvRow]) -> Result<Expr, CsvParseError> {
    let Some((header, data_rows)) = rows.split_first() else {
        return Ok(Expr::call(system_expr("List"), Vec::new()));
    };

    const HEADER_INDEX_THRESHOLD: usize = 16;

    let headers = if header.fields.len() <= HEADER_INDEX_THRESHOLD {
        let mut headers: Vec<String> = Vec::with_capacity(header.fields.len());
        for field in &header.fields {
            if headers.iter().any(|seen| seen == &field.value) {
                return Err(CsvParseError::new(
                    field.offset,
                    format!("duplicate header name {:?}", field.value),
                ));
            }
            headers.push(field.value.clone());
        }
        headers
    } else {
        let mut seen = HashSet::with_capacity(header.fields.len());
        for field in &header.fields {
            if !seen.insert(field.value.as_str()) {
                return Err(CsvParseError::new(
                    field.offset,
                    format!("duplicate header name {:?}", field.value),
                ));
            }
        }
        header
            .fields
            .iter()
            .map(|field| field.value.clone())
            .collect()
    };

    let mut records = Vec::new();
    for (index, row) in data_rows.iter().enumerate() {
        if row.fields.len() != headers.len() {
            return Err(CsvParseError::new(
                row.offset,
                format!(
                    "row {} has {} fields but the header has {} fields",
                    index + 2,
                    row.fields.len(),
                    headers.len()
                ),
            ));
        }

        records.push(Expr::call(
            system_expr("Association"),
            headers
                .iter()
                .zip(row.fields.iter())
                .map(|(header, field)| {
                    Expr::call(
                        system_expr("Rule"),
                        vec![
                            Expr::string(header.clone()),
                            Expr::string(field.value.clone()),
                        ],
                    )
                })
                .collect(),
        ));
    }

    Ok(Expr::call(system_expr("List"), records))
}

fn to_csv(rows: &Expr, config: ToCsvConfig) -> Result<String, CsvEncodeError> {
    let Some(items) = list_items(rows) else {
        return Err(CsvEncodeError::new("ToCsv expects a List of rows."));
    };

    if items.is_empty() {
        return Ok(String::new());
    }

    if items.iter().all(|row| list_items(row).is_some()) {
        let rows = list_rows_from_exprs(items)?;
        return Ok(write_csv_rows(&rows, config.delimiter));
    }

    let association_rows = items
        .iter()
        .map(association_entries_from_expr)
        .collect::<Option<Vec<_>>>();
    if let Some(rows) = association_rows {
        if config.headers == Some(false) {
            return Err(CsvEncodeError::new(
                "ToCsv cannot encode Association rows with \"Headers\" -> False.",
            ));
        }
        return association_rows_to_csv(&rows, config.delimiter);
    }

    Err(CsvEncodeError::new(
        "ToCsv expects either a List of List rows or a List of Association rows.",
    ))
}

fn list_rows_from_exprs(rows: &[Expr]) -> Result<Vec<Vec<String>>, CsvEncodeError> {
    rows.iter()
        .enumerate()
        .map(|(row_index, row)| {
            let items = list_items(row).expect("caller checked row shape");
            items
                .iter()
                .enumerate()
                .map(|(column_index, item)| match item {
                    Expr::Atom(Atom::String(value)) => Ok(value.clone()),
                    _ => Err(CsvEncodeError::new(format!(
                        "Non-String CSV field at row {}, column {}.",
                        row_index + 1,
                        column_index + 1
                    ))),
                })
                .collect()
        })
        .collect()
}

fn association_rows_to_csv(
    rows: &[Vec<(Expr, Expr)>],
    delimiter: char,
) -> Result<String, CsvEncodeError> {
    let first = rows
        .first()
        .expect("association_rows_to_csv is called only for non-empty rows");
    let headers = association_header_strings(first, 1)?;
    let mut output_rows = vec![headers.clone()];

    for (row_index, row) in rows.iter().enumerate() {
        let keyed = association_row_strings(row, row_index + 1)?;
        if keyed.len() != headers.len() {
            return Err(CsvEncodeError::new(format!(
                "Association row {} has different CSV header keys.",
                row_index + 1
            )));
        }

        let output_row = association_row_in_header_order(keyed, &headers, row_index + 1)?;
        output_rows.push(output_row);
    }

    Ok(write_csv_rows(&output_rows, delimiter))
}

fn association_row_in_header_order(
    keyed: Vec<(String, Option<String>)>,
    headers: &[String],
    row_index: usize,
) -> Result<Vec<String>, CsvEncodeError> {
    const INDEX_THRESHOLD: usize = 16;

    if keyed.iter().map(|(key, _)| key).eq(headers.iter()) {
        return keyed
            .into_iter()
            .enumerate()
            .map(|(column_index, (_, value))| csv_string_field(value, row_index, column_index))
            .collect();
    }

    if headers.len() <= INDEX_THRESHOLD {
        return headers
            .iter()
            .enumerate()
            .map(|(column_index, header)| {
                let Some((_, value)) = keyed.iter().find(|(key, _)| key == header) else {
                    return Err(missing_csv_header_error(row_index, header));
                };
                csv_string_field(value.clone(), row_index, column_index)
            })
            .collect();
    }

    let mut keyed = keyed.into_iter().collect::<HashMap<_, _>>();
    headers
        .iter()
        .enumerate()
        .map(|(column_index, header)| {
            let Some(value) = keyed.remove(header) else {
                return Err(missing_csv_header_error(row_index, header));
            };
            csv_string_field(value, row_index, column_index)
        })
        .collect()
}

fn csv_string_field(
    value: Option<String>,
    row_index: usize,
    column_index: usize,
) -> Result<String, CsvEncodeError> {
    value.ok_or_else(|| {
        CsvEncodeError::new(format!(
            "Non-String CSV field at row {}, column {}.",
            row_index,
            column_index + 1
        ))
    })
}

fn missing_csv_header_error(row_index: usize, header: &str) -> CsvEncodeError {
    CsvEncodeError::new(format!(
        "Association row {} is missing CSV header key {:?}.",
        row_index, header
    ))
}

fn association_header_strings(
    row: &[(Expr, Expr)],
    row_index: usize,
) -> Result<Vec<String>, CsvEncodeError> {
    row.iter()
        .map(|(key, _)| match key {
            Expr::Atom(Atom::String(value)) => Ok(value.clone()),
            _ => Err(CsvEncodeError::new(format!(
                "Non-String CSV header key in Association row {row_index}."
            ))),
        })
        .collect()
}

fn association_row_strings(
    row: &[(Expr, Expr)],
    row_index: usize,
) -> Result<Vec<(String, Option<String>)>, CsvEncodeError> {
    row.iter()
        .map(|(key, value)| {
            let Expr::Atom(Atom::String(key)) = key else {
                return Err(CsvEncodeError::new(format!(
                    "Non-String CSV header key in Association row {row_index}."
                )));
            };
            Ok((
                key.clone(),
                match value {
                    Expr::Atom(Atom::String(value)) => Some(value.clone()),
                    _ => None,
                },
            ))
        })
        .collect()
}

fn write_csv_rows(rows: &[Vec<String>], delimiter: char) -> String {
    let mut output = String::new();
    for (row_index, row) in rows.iter().enumerate() {
        if row_index > 0 {
            output.push('\n');
        }
        for (column_index, field) in row.iter().enumerate() {
            if column_index > 0 {
                output.push(delimiter);
            }
            write_csv_field(field, delimiter, &mut output);
        }
    }
    output
}

fn write_csv_field(field: &str, delimiter: char, output: &mut String) {
    let quote = field
        .chars()
        .any(|ch| ch == delimiter || matches!(ch, '"' | '\n' | '\r'));
    if !quote {
        output.push_str(field);
        return;
    }

    output.push('"');
    for ch in field.chars() {
        if ch == '"' {
            output.push_str("\"\"");
        } else {
            output.push(ch);
        }
    }
    output.push('"');
}

fn fail_csv(session: &mut Session, symbol: &str, message: impl Into<String>) -> Expr {
    session.push_message(symbol, "csv", message);
    system_expr("$Failed")
}
