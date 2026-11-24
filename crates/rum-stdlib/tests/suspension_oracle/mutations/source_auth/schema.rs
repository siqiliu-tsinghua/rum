//! Exact current-source overlays; no recipe is allowed to bless its own output.
use super::{AuthError, AuthResult, digest};
use std::{collections::BTreeMap, sync::OnceLock};
pub(super) const SOURCE: &str = "crates/rum-runtime/src/runtime/eval_dispatch/mod.rs";

fn rows() -> &'static BTreeMap<&'static str, [&'static str; 5]> {
    static ROWS: OnceLock<BTreeMap<&str, [&str; 5]>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let mut lines = include_str!("overlays.tsv").lines();
        assert_eq!(
            lines.next(),
            Some("file\tpreimage\tdisabled_sha256\tarmed_sha256\tdisabled\tarmed")
        );
        let mut rows = BTreeMap::new();
        for line in lines {
            let c: Vec<_> = line.split('\t').collect();
            assert_eq!(c.len(), 6);
            assert!(rows.insert(c[0], [c[1], c[2], c[3], c[4], c[5]]).is_none());
        }
        rows
    })
}

pub(super) fn pins() -> BTreeMap<&'static str, &'static str> {
    rows().iter().map(|(&file, c)| (file, c[0])).collect()
}

pub(super) fn file_postimage(file: &str, before: &[u8], enabled: bool) -> AuthResult<Vec<u8>> {
    let c = rows().get(file).ok_or(AuthError::OverlaySet)?;
    if digest(before) != c[0] {
        return Err(AuthError::Preimage);
    }
    let text = c[3 + usize::from(enabled)];
    if !text.is_ascii() || text.len() % 2 != 0 {
        return Err(AuthError::Postimage);
    }
    let bytes = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).map_err(|_| AuthError::Postimage))
        .collect::<AuthResult<Vec<_>>>()?;
    if digest(&bytes) != c[1 + usize::from(enabled)] {
        return Err(AuthError::Postimage);
    }
    Ok(bytes)
}

pub(super) fn postimage(before: &[u8], enabled: bool) -> AuthResult<Vec<u8>> {
    file_postimage(SOURCE, before, enabled)
}

pub(super) fn parse_additions(text: &str) -> AuthResult<BTreeMap<&str, (&str, usize)>> {
    let mut lines = text.lines();
    if lines.next() != Some("file\tsha256\tbytes") {
        return Err(AuthError::OverlaySet);
    }
    let mut rows = BTreeMap::new();
    for line in lines {
        let cells: Vec<_> = line.split('\t').collect();
        if cells.len() != 3 || cells[0].is_empty() {
            return Err(AuthError::OverlaySet);
        }
        let digest_text = cells[1];
        if digest_text.len() != 64
            || !digest_text
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(AuthError::OverlaySet);
        }
        let Ok(len) = cells[2].parse::<usize>() else {
            return Err(AuthError::OverlaySet);
        };
        if rows.insert(cells[0], (digest_text, len)).is_some() {
            return Err(AuthError::OverlaySet);
        }
    }
    if rows.is_empty() {
        return Err(AuthError::OverlaySet);
    }
    Ok(rows)
}

pub(super) fn additions() -> AuthResult<BTreeMap<&'static str, (&'static str, usize)>> {
    parse_additions(include_str!("additions.tsv"))
}
