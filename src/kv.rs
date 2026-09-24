//! Key/value text handling: header text, query params and the Authorization
//! header. This is the ONLY place that parses these formats.
//!
//! Header text is one `Key: Value` per line; a leading `#` disables a line.
//! Params use the same format with `=`.

use base64::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct KeyValue {
    pub enabled: bool,
    pub key: String,
    pub value: String,
}

impl KeyValue {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            enabled: true,
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.key.is_empty() && self.value.is_empty()
    }
}

/// Parse editor text into rows (for the key/value editor). Keys are trimmed,
/// values are kept verbatim so trailing spaces survive editing.
pub fn parse_lines(text: &str, sep: &str) -> Vec<KeyValue> {
    text.lines()
        .map(str::trim_start)
        .filter(|l| !l.is_empty())
        .map(|line| {
            let (enabled, line) = match line.strip_prefix('#') {
                Some(rest) => (false, rest.trim_start()),
                None => (true, line),
            };
            let (key, value) = line.split_once(sep).unwrap_or((line, ""));
            KeyValue {
                enabled,
                key: key.trim().to_string(),
                value: value.to_string(),
            }
        })
        .collect()
}

/// Inverse of [`parse_lines`]; drops empty rows.
pub fn format_lines(rows: &[KeyValue], sep: &str) -> String {
    rows.iter()
        .filter(|r| !r.is_empty())
        .map(|r| {
            let line = if r.value.is_empty() {
                r.key.clone()
            } else {
                format!("{}{}{}", r.key, sep, r.value)
            };
            if r.enabled {
                line
            } else {
                format!("# {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn count_enabled(rows: &[KeyValue]) -> usize {
    rows.iter()
        .filter(|r| r.enabled && !r.key.is_empty())
        .count()
}

/// The headers that actually get sent/saved: enabled `Key: Value` lines,
/// trimmed. Lines without a colon are ignored.
pub fn headers_to_map(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

pub fn map_to_headers<'a>(headers: impl IntoIterator<Item = (&'a String, &'a String)>) -> String {
    headers
        .into_iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Authorization header — the header text is the single source of truth; the
// Auth tab is only a view over it.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    None,
    Basic,
    Bearer,
    Custom,
}

impl AuthMode {
    pub const ALL: [AuthMode; 4] = [
        AuthMode::None,
        AuthMode::Basic,
        AuthMode::Bearer,
        AuthMode::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AuthMode::None => "None",
            AuthMode::Basic => "Basic",
            AuthMode::Bearer => "Bearer",
            AuthMode::Custom => "Custom",
        }
    }
}

fn is_auth_line(line: &str) -> bool {
    line.len() >= 14 && line[..14].eq_ignore_ascii_case("authorization:")
}

/// Value of the first enabled `Authorization:` line. Only the space after the
/// colon is stripped so users can type trailing spaces (e.g. "Digest ").
pub fn auth_value(headers_text: &str) -> Option<&str> {
    headers_text
        .lines()
        .map(str::trim_start)
        .find(|l| is_auth_line(l))
        .map(|l| l[14..].trim_start())
}

pub fn auth_mode(headers_text: &str) -> AuthMode {
    match auth_value(headers_text) {
        None => AuthMode::None,
        Some(v) if v == "Basic" || v.starts_with("Basic ") => AuthMode::Basic,
        Some(v) if v == "Bearer" || v.starts_with("Bearer ") => AuthMode::Bearer,
        Some(_) => AuthMode::Custom,
    }
}

/// Replace, add (`Some`) or remove (`None`) the enabled Authorization line.
/// Disabled `# Authorization:` lines are left alone.
pub fn set_auth(headers_text: &str, value: Option<&str>) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut replaced = false;
    for line in headers_text.lines() {
        if !replaced && is_auth_line(line.trim_start()) {
            replaced = true;
            if let Some(v) = value {
                lines.push(format!("Authorization: {v}"));
            }
        } else {
            lines.push(line.to_string());
        }
    }
    if let (false, Some(v)) = (replaced, value) {
        lines.push(format!("Authorization: {v}"));
    }
    lines.join("\n")
}

pub fn basic_auth(username: &str, password: &str) -> String {
    format!(
        "Basic {}",
        BASE64_STANDARD.encode(format!("{username}:{password}"))
    )
}

/// Decode `Basic <base64>` into (username, password); empty on failure.
pub fn decode_basic(value: &str) -> (String, String) {
    let encoded = value.strip_prefix("Basic").unwrap_or("").trim();
    BASE64_STANDARD
        .decode(encoded)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|s| {
            s.split_once(':')
                .map(|(u, p)| (u.to_string(), p.to_string()))
        })
        .unwrap_or_default()
}

pub fn bearer_token(value: &str) -> &str {
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("Bearer"))
        .unwrap_or(value)
}

// ---------------------------------------------------------------------------
// Query params <-> URL
// ---------------------------------------------------------------------------

pub fn parse_query_params(url: &str) -> Vec<KeyValue> {
    let Some((_, query)) = url.split_once('?') else {
        return Vec::new();
    };
    let query = query.split('#').next().unwrap_or("");
    query
        .split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            KeyValue::new(url_decode(k), url_decode(v))
        })
        .collect()
}

/// Replace the URL's query string with the enabled params.
pub fn build_url(url: &str, params: &[KeyValue]) -> String {
    let base = url.split('?').next().unwrap_or(url);
    let query: Vec<String> = params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| {
            if p.value.is_empty() {
                url_encode(&p.key)
            } else {
                format!("{}={}", url_encode(&p.key), url_encode(&p.value))
            }
        })
        .collect();
    if query.is_empty() {
        base.to_string()
    } else {
        format!("{base}?{}", query.join("&"))
    }
}

/// Percent-decode (`+` is a space). Invalid escapes are kept as-is.
pub fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (b'+', _) => {
                out.push(b' ');
                i += 1;
            }
            (b, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Percent-encode everything except RFC 3986 unreserved chars, leaving
/// `{{variable}}` blocks intact so they can be substituted later.
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("{{") {
            if let Some(end) = after.find("}}") {
                out.push_str(&rest[..end + 4]);
                rest = &rest[end + 4..];
                continue;
            }
        }
        let c = rest.chars().next().unwrap();
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
            out.push(c);
        } else {
            let mut buf = [0; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        }
        rest = &rest[c.len_utf8()..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lines_preserves_value_spaces_and_disabled_rows() {
        let rows = parse_lines("  Key  : Value   \n# Off: x\nflag", ":");
        assert_eq!(rows[0].key, "Key");
        assert_eq!(rows[0].value, " Value   ");
        assert!(!rows[1].enabled);
        assert_eq!(rows[1].key, "Off");
        assert_eq!(rows[2], KeyValue::new("flag", ""));
        assert_eq!(parse_lines("Key:   ", ":")[0].value, "   ");
    }

    #[test]
    fn format_lines_is_inverse_of_parse() {
        let text = "a=1\n# b=2\nflag";
        assert_eq!(format_lines(&parse_lines(text, "="), "="), text);
    }

    #[test]
    fn headers_to_map_skips_disabled_and_colonless_lines() {
        let m = headers_to_map("A: 1\n# B: 2\nnocolon\n  C :  3  \n");
        assert_eq!(m.len(), 2);
        assert_eq!(m["A"], "1");
        assert_eq!(m["C"], "3");
    }

    #[test]
    fn count_enabled_ignores_disabled_and_empty_keys() {
        let rows = parse_lines("H: V\n# Off: x\nH3: V3", ":");
        assert_eq!(count_enabled(&rows), 2);
        assert_eq!(count_enabled(&[KeyValue::new("", "v")]), 0);
    }

    #[test]
    fn auth_mode_and_values_come_from_header_text() {
        let h = "Content-Type: a\nAuthorization: Basic dXNlcjpwYXNz";
        assert_eq!(auth_mode(h), AuthMode::Basic);
        assert_eq!(
            decode_basic(auth_value(h).unwrap()),
            ("user".into(), "pass".into())
        );

        assert_eq!(auth_mode("authorization: Bearer t"), AuthMode::Bearer);
        assert_eq!(bearer_token("Bearer Bearer Token"), "Bearer Token");
        assert_eq!(bearer_token("Bearer "), "");
        assert_eq!(auth_mode("Authorization: ApiKey x"), AuthMode::Custom);
        assert_eq!(auth_mode("Authorization: "), AuthMode::Custom);
        assert_eq!(auth_mode("Accept: */*"), AuthMode::None);
        // disabled lines don't count
        let h = "# Authorization: Bearer old\nAuthorization: Bearer new";
        assert_eq!(bearer_token(auth_value(h).unwrap()), "new");
    }

    #[test]
    fn auth_value_keeps_trailing_spaces_while_typing() {
        assert_eq!(auth_value("Authorization: Digest "), Some("Digest "));
    }

    #[test]
    fn set_auth_adds_replaces_and_removes() {
        let h = set_auth("A: 1", Some("Bearer x"));
        assert_eq!(h, "A: 1\nAuthorization: Bearer x");
        let h = set_auth("authorization: Old\nB: 2", Some("New"));
        assert_eq!(h, "Authorization: New\nB: 2");
        assert_eq!(set_auth("Authorization: x\nB: 2", None), "B: 2");
        // disabled line is kept, a new active line is added
        assert_eq!(
            set_auth("# Authorization: a", Some("b")),
            "# Authorization: a\nAuthorization: b"
        );
        // custom with empty value keeps the header (so the mode sticks)
        assert_eq!(auth_mode(&set_auth("", Some(""))), AuthMode::Custom);
    }

    #[test]
    fn basic_auth_encodes() {
        assert_eq!(basic_auth("user", "pass"), "Basic dXNlcjpwYXNz");
        assert_eq!(basic_auth("admin", "1234"), "Basic YWRtaW46MTIzNA==");
    }

    #[test]
    fn query_params_parse() {
        let p = parse_query_params("https://api.com/s?q=test&page=1#frag");
        assert_eq!(
            p,
            vec![KeyValue::new("q", "test"), KeyValue::new("page", "1")]
        );
        assert!(parse_query_params("https://api.com/users").is_empty());
        assert!(parse_query_params("https://api.com?").is_empty());
        assert_eq!(
            parse_query_params("x?flag&debug")[0],
            KeyValue::new("flag", "")
        );
        assert_eq!(parse_query_params("x?tag=a&tag=b").len(), 2);
    }

    #[test]
    fn url_decode_handles_utf8_plus_and_bad_escapes() {
        assert_eq!(url_decode("John%20Doe"), "John Doe");
        assert_eq!(url_decode("New+York"), "New York");
        assert_eq!(url_decode("caf%C3%A9"), "café");
        assert_eq!(url_decode("100%"), "100%");
        assert_eq!(url_decode("%zz"), "%zz");
    }

    #[test]
    fn build_url_encodes_and_keeps_variables() {
        let mut off = KeyValue::new("page", "1");
        off.enabled = false;
        let p = [KeyValue::new("q", "hello world"), off];
        assert_eq!(
            build_url("https://a.com/s?old=1", &p),
            "https://a.com/s?q=hello%20world"
        );
        assert_eq!(build_url("https://a.com", &[]), "https://a.com");
        let p = [KeyValue::new("token", "{{API_KEY}}")];
        assert_eq!(
            build_url("https://a.com", &p),
            "https://a.com?token={{API_KEY}}"
        );
    }

    #[test]
    fn url_encode_basics() {
        assert_eq!(url_encode("a=b&c=d"), "a%3Db%26c%3Dd");
        assert_eq!(url_encode("é"), "%C3%A9");
        assert_eq!(url_encode("{{id}}/x"), "{{id}}%2Fx");
        assert_eq!(url_encode("{{unclosed"), "%7B%7Bunclosed");
    }
}
