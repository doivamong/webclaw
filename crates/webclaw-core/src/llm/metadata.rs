/// Metadata header building for LLM-optimized output.
///
/// Produces `> ` prefixed lines with URL, title, author, etc.
/// Omits empty/zero fields to minimize token waste.
use std::fmt::Write as _;

use crate::types::ExtractionResult;

pub(crate) fn build_metadata_header(
    out: &mut String,
    result: &ExtractionResult,
    url: Option<&str>,
) {
    let meta = &result.metadata;

    // URL: prefer explicit arg, fall back to metadata
    let effective_url = url.or(meta.url.as_deref());
    if let Some(u) = effective_url {
        let _ = writeln!(out, "> URL: {u}");
    }
    if let Some(t) = &meta.title
        && !t.is_empty()
    {
        let _ = writeln!(out, "> Title: {t}");
    }
    if let Some(d) = &meta.description
        && !d.is_empty()
    {
        let _ = writeln!(out, "> Description: {d}");
    }
    if let Some(a) = &meta.author
        && !a.is_empty()
    {
        let _ = writeln!(out, "> Author: {a}");
    }
    if let Some(d) = &meta.published_date
        && !d.is_empty()
    {
        let _ = writeln!(out, "> Published: {d}");
    }
    if let Some(l) = &meta.language
        && !l.is_empty()
    {
        let _ = writeln!(out, "> Language: {l}");
    }
    if meta.word_count > 0 {
        let _ = writeln!(out, "> Word count: {}", meta.word_count);
    }
}
