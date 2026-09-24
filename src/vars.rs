//! Environment variables: `.env` parsing and `{{name}}` substitution.

use std::collections::HashMap;

/// Parse dotenv content: `KEY=VALUE` lines, `#` comments, matching quotes stripped.
pub fn parse_env(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| {
            let v = v.trim();
            let quoted = v.len() >= 2
                && ((v.starts_with('"') && v.ends_with('"'))
                    || (v.starts_with('\'') && v.ends_with('\'')));
            let v = if quoted { &v[1..v.len() - 1] } else { v };
            (k.trim().to_string(), v.to_string())
        })
        .collect()
}

/// Calls `f` with each `{{name}}` (trimmed) and its byte range in `text`.
fn for_each_var(text: &str, mut f: impl FnMut(&str, std::ops::Range<usize>)) {
    let mut pos = 0;
    while let Some(start) = text[pos..].find("{{").map(|i| pos + i) {
        let Some(len) = text[start + 2..].find("}}") else {
            return;
        };
        let end = start + 2 + len + 2;
        f(text[start + 2..end - 2].trim(), start..end);
        pos = end;
    }
}

/// Replace `{{name}}` with its value; unknown variables are left untouched.
pub fn substitute(text: &str, vars: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for_each_var(text, |name, range| {
        if let Some(value) = vars.get(name) {
            out.push_str(&text[last..range.start]);
            out.push_str(value);
            last = range.end;
        }
    });
    out.push_str(&text[last..]);
    out
}

/// Names of all `{{variables}}` in `text`, in order (may repeat).
pub fn extract(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for_each_var(text, |name, _| {
        if !name.is_empty() {
            names.push(name.to_string());
        }
    });
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> HashMap<String, String> {
        parse_env("host=api.example.com\ntoken=abc123")
    }

    #[test]
    fn parse_env_handles_comments_and_quotes() {
        let v = parse_env("# c\n\nA=1\nB=\"two words\"\nC='x'\nD=\"unbalanced\nE=a=b");
        assert_eq!(v["A"], "1");
        assert_eq!(v["B"], "two words");
        assert_eq!(v["C"], "x");
        assert_eq!(v["D"], "\"unbalanced");
        assert_eq!(v["E"], "a=b");
    }

    #[test]
    fn substitute_replaces_known_and_keeps_unknown() {
        let out = substitute("https://{{host}}/u?t={{ token }}&x={{missing}}", &vars());
        assert_eq!(out, "https://api.example.com/u?t=abc123&x={{missing}}");
        assert_eq!(substitute("{{unclosed", &vars()), "{{unclosed");
    }

    #[test]
    fn substitute_does_not_recurse_into_values() {
        let v = parse_env("a={{b}}\nb=x");
        assert_eq!(substitute("{{a}}", &v), "{{b}}");
    }

    #[test]
    fn extract_lists_names() {
        assert_eq!(extract("{{a}} and {{ b }} and {{}}"), vec!["a", "b"]);
    }
}
