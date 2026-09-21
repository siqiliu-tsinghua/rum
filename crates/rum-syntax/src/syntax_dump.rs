//! Alternate-Debug layout with one indentation writer and bounded traversal.
//!
//! Only leaves use derived Debug: their nesting is independent of tree depth.
//! Every traversal step and every padding write costs at most its output size.

use std::fmt::{self, Write};

use crate::{SyntaxExpr, SyntaxExprKind};

const FRONT_DEPTH: usize = 16;

pub(super) struct Dump<'a>(pub(super) &'a SyntaxExpr);

impl fmt::Display for Dump<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut writer = Indented {
            output,
            depth: 0,
            line_start: true,
        };
        emit(Task::Expr(self.0), &mut writer, 0)
    }
}

struct Indented<'a, 'b> {
    output: &'a mut fmt::Formatter<'b>,
    depth: usize,
    line_start: bool,
}

impl Write for Indented<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        const SPACES: &str = "                                                                ";
        for part in text.split_inclusive('\n') {
            if self.line_start {
                let mut padding = self.depth * 4;
                while padding > 0 {
                    let count = padding.min(SPACES.len());
                    self.output.write_str(&SPACES[..count])?;
                    padding -= count;
                }
            }
            self.output.write_str(part)?;
            self.line_start = part.ends_with('\n');
        }
        Ok(())
    }
}

// All tree edges are borrowed. The iterative tail's work stack cannot acquire
// recursive destruction, and sequence continuations keep wide lists off it.
enum Task<'a> {
    Expr(&'a SyntaxExpr),
    Kind(&'a SyntaxExprKind),
    List(&'a [SyntaxExpr]),
    Entries(&'a [SyntaxExpr]),
    Optional(Option<&'a SyntaxExpr>),
    Leaf(&'a dyn fmt::Debug),
    Text(&'static str),
    Indent,
    Dedent,
}

type Expansion<'a> = [Option<Task<'a>>; 11];

fn parts<const N: usize>(tasks: [Task<'_>; N]) -> Expansion<'_> {
    const { assert!(N <= 11) };
    let mut result = [const { None }; 11];
    for (slot, task) in result.iter_mut().zip(tasks) {
        *slot = Some(task);
    }
    result
}

fn emit(task: Task<'_>, writer: &mut Indented<'_, '_>, depth: usize) -> fmt::Result {
    if depth == FRONT_DEPTH {
        return emit_iterative(task, writer);
    }
    for child in expand(task, writer)?.into_iter().flatten() {
        emit(child, writer, depth + 1)?;
    }
    Ok(())
}

#[cold]
fn emit_iterative(task: Task<'_>, writer: &mut Indented<'_, '_>) -> fmt::Result {
    let mut pending = vec![task];
    while let Some(task) = pending.pop() {
        pending.extend(expand(task, writer)?.into_iter().flatten().rev());
    }
    Ok(())
}

fn tuple<'a>(open: &'static str, value: Task<'a>) -> Expansion<'a> {
    use Task::*;
    parts([Text(open), Indent, value, Text(",\n"), Dedent, Text(")")])
}

fn fields<'a>(
    open: &'static str,
    first: &'static str,
    first_value: Task<'a>,
    second: &'static str,
    second_value: Task<'a>,
) -> Expansion<'a> {
    use Task::*;
    parts([
        Text(open),
        Indent,
        Text(first),
        first_value,
        Text(second),
        second_value,
        Text(",\n"),
        Dedent,
        Text("}"),
    ])
}

fn expand<'a>(task: Task<'a>, writer: &mut Indented<'_, '_>) -> Result<Expansion<'a>, fmt::Error> {
    use Task::*;
    let children = match task {
        Text(text) => {
            writer.write_str(text)?;
            parts([])
        }
        Indent => {
            writer.depth += 1;
            parts([])
        }
        Dedent => {
            writer.depth -= 1;
            parts([])
        }
        Leaf(value) => {
            write!(writer, "{value:#?}")?;
            parts([])
        }
        Expr(expr) => fields(
            "SyntaxExpr {\n",
            "kind: ",
            Kind(&expr.kind),
            ",\nspan: ",
            Leaf(&expr.span),
        ),
        List([]) => parts([Text("[]")]),
        List(items) => parts([Text("[\n"), Indent, Entries(items), Dedent, Text("]")]),
        Entries([]) => parts([]),
        Entries([first, rest @ ..]) => parts([Expr(first), Text(",\n"), Entries(rest)]),
        Optional(None) => parts([Text("None")]),
        Optional(Some(expr)) => tuple("Some(\n", Expr(expr)),
        Kind(kind) => match kind {
            // These variants contain only strings or a SlotSpec, never an AST.
            SyntaxExprKind::Integer(_)
            | SyntaxExprKind::String(_)
            | SyntaxExprKind::Symbol(_)
            | SyntaxExprKind::Slot(_) => parts([Leaf(kind)]),
            SyntaxExprKind::List(items) => tuple("List(\n", List(items)),
            SyntaxExprKind::Association(items) => tuple("Association(\n", List(items)),
            SyntaxExprKind::Blank { kind, head } => fields(
                "Blank {\n",
                "kind: ",
                Leaf(kind),
                ",\nhead: ",
                Optional(head.as_deref()),
            ),
            SyntaxExprKind::Pattern { name, pattern } => fields(
                "Pattern {\n",
                "name: ",
                Expr(name),
                ",\npattern: ",
                Expr(pattern),
            ),
            SyntaxExprKind::Prefix { op, expr } => {
                fields("Prefix {\n", "op: ", Leaf(op), ",\nexpr: ", Expr(expr))
            }
            SyntaxExprKind::Postfix { op, expr } => {
                fields("Postfix {\n", "op: ", Leaf(op), ",\nexpr: ", Expr(expr))
            }
            SyntaxExprKind::Binary { op, lhs, rhs } => parts([
                Text("Binary {\n"),
                Indent,
                Text("op: "),
                Leaf(op),
                Text(",\nlhs: "),
                Expr(lhs),
                Text(",\nrhs: "),
                Expr(rhs),
                Text(",\n"),
                Dedent,
                Text("}"),
            ]),
            SyntaxExprKind::ComparisonChain {
                operands,
                operators,
            } => fields(
                "ComparisonChain {\n",
                "operands: ",
                List(operands),
                ",\noperators: ",
                Leaf(operators),
            ),
            SyntaxExprKind::Call { head, args } => {
                fields("Call {\n", "head: ", Expr(head), ",\nargs: ", List(args))
            }
        },
    };
    Ok(children)
}
