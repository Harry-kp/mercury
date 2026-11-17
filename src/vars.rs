//! Environment variables: `.env` parsing and `{{name}}` substitution.

use std::collections::HashMap;

/// Parse dotenv content: `KEY=VALUE` lines, `#` comments, matching quotes
/// stripped, a shell-style `export ` prefix and a byte-order mark ignored.
pub fn parse_env(content: &str) -> HashMap<String, String> {
    content
        .trim_start_matches('\u{feff}')
        .lines()
        .map(|l| {
            let l = l.trim();
            l.strip_prefix("export ").unwrap_or(l)
        })
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| {
            let v = v.trim();
            let quoted = v.len() >= 2
                && ((v.starts_with('"') && v.ends_with('"'))
                    || (v.starts_with('\'') && v.ends_with('\'')));
            let v = match (quoted, v.starts_with('"')) {
                // `env_value` writes `"` as `\"` and a newline as `\n`
                (true, true) => v[1..v.len() - 1].replace("\\n", "\n").replace("\\\"", "\""),
                (true, false) => v[1..v.len() - 1].to_string(),
                _ => v.to_string(),
            };
            (k.trim().to_string(), v)
        })
        .collect()
}

/// A key as `.env` can hold it. Kept as-is (requests reference them as
/// `{{api-key}}`); only characters that would break a `KEY=VALUE` line change.
pub fn env_key(key: &str) -> String {
    let key: String = key
        .trim()
        .chars()
        .map(|c| {
            if c == '=' || c.is_whitespace() {
                '_'
            } else {
                c
            }
        })
        .collect();
    if key.starts_with('#') {
        format!("_{key}")
    } else {
        key
    }
}

/// A value as `.env` can hold it, so `parse_env` reads back exactly `value`.
pub fn env_value(value: &str) -> String {
    if value.contains(['=', '\n', '"', ' ']) {
        format!("\"{}\"", value.replace('"', "\\\"").replace('\n', "\\n"))
    } else {
        value.to_string()
    }
}

/// `content` with `key` set to `value`: its line replaced where it is (comments
/// and order kept), or added at the end.
pub fn set_env_var(content: &str, key: &str, value: &str) -> String {
    let key = env_key(key);
    let line = format!("{key}={}", env_value(value));
    let mut found = false;
    let mut out: Vec<String> = content
        .lines()
        .map(|l| {
            let t = l.trim();
            let bare = t.strip_prefix("export ").unwrap_or(t);
            match bare.split_once('=') {
                Some((k, _)) if k.trim() == key && !t.starts_with('#') => {
                    found = true;
                    line.clone()
                }
                _ => l.to_string(),
            }
        })
        .collect();
    if !found {
        out.push(line);
    }
    out.join("\n") + "\n"
}

/// Calls `f` with each `{{name}}` (trimmed) and its byte range in `text`.
pub fn for_each_var(text: &str, mut f: impl FnMut(&str, std::ops::Range<usize>)) {
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

/// The `{{name}}` under character `char_index` (what a text galley's cursor counts).
pub fn var_at(text: &str, char_index: usize) -> Option<String> {
    let byte = text.char_indices().nth(char_index)?.0;
    let mut hit = None;
    for_each_var(text, |name, range| {
        if range.contains(&byte) {
            hit = Some(name.to_string());
        }
    });
    hit
}

/// Postman's built-in variables, which imported collections use as they
/// are: a fresh value at every send. `None` for any other name.
pub fn dynamic(name: &str) -> Option<String> {
    use std::hash::{BuildHasher, Hasher};
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    // std's per-hasher random keys: enough for test data, no dependency
    let random = || {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(now.as_nanos());
        h.finish()
    };
    match name {
        "$guid" | "$randomUUID" => {
            let (a, b) = (random(), random());
            // a v4 UUID: version 4, variant 10xx
            Some(format!(
                "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
                a >> 32,
                (a >> 16) & 0xffff,
                a & 0xfff,
                0x8000 | (b >> 48) & 0x3fff,
                b & 0xffff_ffff_ffff
            ))
        }
        "$timestamp" => Some(now.as_secs().to_string()),
        "$isoTimestamp" => Some(iso_utc(now)),
        "$randomInt" => Some((random() % 1001).to_string()),
        _ => None,
    }
}

/// `2026-11-17T15:04:05.123Z`, without a date crate (days → civil date).
fn iso_utc(t: std::time::Duration) -> String {
    let secs = t.as_secs();
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        t.subsec_millis()
    )
}

/// Replace `{{name}}` with its value (or a [`dynamic`] one); unknown
/// variables are left untouched.
pub fn substitute(text: &str, vars: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for_each_var(text, |name, range| {
        if let Some(value) = vars.get(name).cloned().or_else(|| dynamic(name)) {
            out.push_str(&text[last..range.start]);
            out.push_str(&value);
            last = range.end;
        }
    });
    out.push_str(&text[last..]);
    out
}

/// Names of all `{{variables}}` in `text` an environment has to define, in
/// order (may repeat). Built-ins like `{{$guid}}` need no definition.
pub fn extract(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for_each_var(text, |name, _| {
        if !name.is_empty() && dynamic(name).is_none() {
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
    fn a_saved_variable_reads_back_exactly() {
        let env = "# dev\nBASE=http://x\nTOKEN=old\n";
        let tricky = "line one\nsays \"hi\" a=b";
        let out = set_env_var(env, "TOKEN", tricky);
        assert!(out.starts_with("# dev\nBASE=http://x\nTOKEN="), "{out}");
        assert_eq!(parse_env(&out)["TOKEN"], tricky);
        let added = set_env_var(&out, "user id", "42");
        assert_eq!(parse_env(&added)["user_id"], "42");
        assert_eq!(parse_env(&added)["BASE"], "http://x");
    }

    #[test]
    fn var_at_finds_the_variable_under_a_character() {
        let url = "é{{ host }}/x/{{id}}";
        assert_eq!(var_at(url, 0), None);
        assert_eq!(var_at(url, 1).as_deref(), Some("host"));
        assert_eq!(var_at(url, 10).as_deref(), Some("host"));
        assert_eq!(var_at(url, 11), None);
        assert_eq!(var_at(url, 16).as_deref(), Some("id"));
        assert_eq!(var_at(url, 99), None);
    }

    #[test]
    fn parse_env_handles_comments_and_quotes() {
        let v = parse_env("# c\n\nA=1\nB=\"two words\"\nC='x'\nD=\"unbalanced\nE=a=b");
        assert_eq!(v["A"], "1");
        assert_eq!(v["B"], "two words");
        assert_eq!(v["C"], "x");
        assert_eq!(v["D"], "\"unbalanced");
        assert_eq!(v["E"], "a=b");
        assert_eq!(parse_env(r#"J="{\"a\":1}""#)["J"], r#"{"a":1}"#);
        // shell-style files and Windows editors
        let v = parse_env("\u{feff}A=1\nexport B=2\n  export C = 3");
        assert_eq!(
            (v["A"].as_str(), v["B"].as_str(), v["C"].as_str()),
            ("1", "2", "3")
        );
    }

    /// Imported Postman collections use these as they are.
    #[test]
    fn postman_dynamic_variables() {
        let none = HashMap::new();
        let a = substitute("{{$guid}}", &none);
        let b = substitute("{{$guid}}", &none);
        assert_eq!((a.len(), &a[14..15]), (36, "4"), "{a}");
        assert!(a.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
        assert_ne!(a, b, "fresh at every send");
        assert!(substitute("{{$timestamp}}", &none).parse::<u64>().unwrap() > 1_700_000_000);
        assert!(substitute("{{$randomInt}}", &none).parse::<u32>().unwrap() <= 1000);
        assert_eq!(
            iso_utc(std::time::Duration::from_millis(1_763_391_845_123)),
            "2025-11-17T15:04:05.123Z"
        );
        assert_eq!(
            iso_utc(std::time::Duration::ZERO),
            "1970-01-01T00:00:00.000Z"
        );
        // nothing for an environment to define, so no "undefined" warning
        assert_eq!(extract("{{$guid}} {{token}}"), vec!["token"]);
        // a .env value wins over a built-in of the same name
        let mine = HashMap::from([("$timestamp".to_string(), "1".to_string())]);
        assert_eq!(substitute("{{$timestamp}}", &mine), "1");
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
