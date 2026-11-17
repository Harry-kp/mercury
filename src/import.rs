//! Import Postman (v2.1 JSON) and Insomnia (JSON/YAML) exports into a
//! workspace folder as Mercury request files + `.env.<name>` files.

use crate::kv::{self, url_encode, KeyValue};
use crate::model::{HttpMethod, RequestFile};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

/// What an import produced, and what it could not bring across.
#[derive(Debug, Default)]
pub struct ImportCount {
    pub requests: usize,
    pub envs: usize,
    /// One line per request whose auth or body Mercury cannot express.
    pub skipped: Vec<String>,
    /// The folder the requests went into (the workspace itself, or a new
    /// folder in it when it already had files).
    pub folder: std::path::PathBuf,
    /// The first environment file written, to select after the import.
    pub env: Option<String>,
}

// ---------------------------------------------------------------------------
// Shared writers
// ---------------------------------------------------------------------------

/// Lowercase, dashes for spaces and characters invalid on any OS.
fn sanitize_filename(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.to_lowercase().chars() {
        if matches!(
            ch,
            ' ' | '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
        ) {
            if !out.is_empty() && !out.ends_with('-') {
                out.push('-');
            }
        } else {
            out.push(ch);
        }
    }
    let out = out.trim_end_matches('-').trim_start_matches('.');
    if out.is_empty() {
        "untitled".to_string()
    } else {
        out.to_string()
    }
}

/// Keep keys as-is (requests reference them as `{{api-key}}`); only replace
/// characters that would break a `KEY=VALUE` line.
fn sanitize_env_key(key: &str) -> String {
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

fn escape_env_value(value: &str) -> String {
    if value.contains(['=', '\n', '"', ' ']) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// `path` if nothing is there yet, else `name-2`, `name-3`… (before a
/// `.json` extension). An import never overwrites what is already on disk:
/// a request or `.env` the user had, or an earlier request of the same name.
fn unique(path: std::path::PathBuf) -> std::path::PathBuf {
    if !path.exists() {
        return path;
    }
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let (stem, ext) = match name.strip_suffix(".json") {
        Some(stem) => (stem.to_string(), ".json"),
        None => (name.clone(), ""),
    };
    (2..)
        .map(|n| path.with_file_name(format!("{stem}-{n}{ext}")))
        .find(|p| !p.exists())
        .expect("some suffix is free")
}

/// Where an import's requests go: straight into `out` when it's empty (a
/// new workspace), else into a new folder named after the collection, so
/// importing into an open workspace can't mix with or replace its files.
fn import_folder(out: &Path, name: &str) -> Result<std::path::PathBuf, String> {
    let empty = fs::read_dir(out)
        .map(|entries| {
            entries
                .flatten()
                .all(|e| e.file_name().to_string_lossy() == ".DS_Store")
        })
        .unwrap_or(true);
    let dir = if empty {
        out.to_path_buf()
    } else {
        unique(out.join(sanitize_filename(name)))
    };
    create_dir(&dir)?;
    Ok(dir)
}

fn write_file(path: &Path, content: String) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Failed to write '{}': {e}", path.display()))
}

fn create_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| format!("Failed to create '{}': {e}", path.display()))
}

fn write_request(
    dir: &Path,
    name: &str,
    method: &str,
    url: String,
    headers: BTreeMap<String, String>,
    body: String,
) -> Result<(), String> {
    // One header is one line: a newline in a name or value would become a
    // second header once the file is loaded.
    let flat = |s: String| s.replace(['\r', '\n'], " ");
    let headers: BTreeMap<String, String> = headers
        .into_iter()
        .map(|(k, v)| (flat(k), flat(v)))
        .collect();
    // Whatever path the body took (a raw body under a multipart
    // Content-Type, an odd mimeType), if it could be sent as Multipart, no
    // line of it is a live file upload until the user ticks it: a collection
    // from someone else could otherwise post ~/.ssh/id_rsa. "Could" is
    // decided before variables: `Content-Type: {{CT}}`, or a header *named*
    // by a variable, can turn multipart at send time.
    let maybe_multipart = headers.iter().any(|(k, v)| {
        k.contains("{{")
            || (k.eq_ignore_ascii_case("content-type")
                && (v.contains("{{") || v.to_ascii_lowercase().contains("multipart")))
    });
    let body = if maybe_multipart {
        let mut rows = kv::parse_lines(&body, "=");
        for row in &mut rows {
            if row.value.starts_with('@') && !row.value.starts_with("@@") {
                row.enabled = false;
            }
        }
        kv::format_lines(&rows, "=")
    } else {
        body
    };
    let request = RequestFile {
        method: HttpMethod::parse(method).unwrap_or_default(),
        url,
        headers,
        body,
        ..Default::default()
    };
    let path = unique(dir.join(format!("{}.json", sanitize_filename(name))));
    write_file(&path, request.to_json())
}

/// Writes `.env.<name>` (never over an existing one) and returns its name.
fn write_env<'a>(
    dir: &Path,
    name: &str,
    vars: impl IntoIterator<Item = (&'a String, &'a Value)>,
) -> Result<String, String> {
    let content: String = vars
        .into_iter()
        .map(|(k, v)| {
            format!(
                "{}={}\n",
                sanitize_env_key(k),
                escape_env_value(&value_to_string(v))
            )
        })
        .collect();
    let path = unique(dir.join(format!(".env.{}", sanitize_filename(name))));
    write_file(&path, content)?;
    Ok(path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned())
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {e}", path.display()))
}

// ---------------------------------------------------------------------------
// Postman
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PostmanCollection {
    info: PostmanInfo,
    item: Vec<PostmanItem>,
    #[serde(default)]
    auth: Option<PostmanAuth>,
    #[serde(default)]
    variable: Vec<PostmanVariable>,
}

#[derive(Deserialize)]
struct PostmanInfo {
    name: String,
}

#[derive(Deserialize)]
struct PostmanItem {
    name: String,
    #[serde(default)]
    item: Vec<PostmanItem>,
    #[serde(default)]
    request: Option<PostmanRequest>,
    /// A folder can carry auth for everything under it.
    #[serde(default)]
    auth: Option<PostmanAuth>,
}

#[derive(Deserialize)]
struct PostmanRequest {
    method: String,
    #[serde(default)]
    header: Vec<PostmanKeyValue>,
    url: PostmanUrl,
    #[serde(default)]
    body: Option<PostmanBody>,
    #[serde(default)]
    auth: Option<PostmanAuth>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PostmanUrl {
    String(String),
    Object {
        #[serde(default)]
        raw: Option<String>,
        #[serde(default)]
        protocol: Option<String>,
        #[serde(default)]
        host: Vec<String>,
        #[serde(default)]
        path: Vec<String>,
        #[serde(default)]
        query: Vec<PostmanKeyValue>,
        /// path variables: `/users/:userId` with `{"key": "userId", "value": "42"}`
        #[serde(default)]
        variable: Vec<PostmanKeyValue>,
    },
}

#[derive(Deserialize)]
struct PostmanKeyValue {
    #[serde(default)]
    key: String,
    /// Postman leaves `value` out entirely for a valueless param or header
    /// (`?archived`), and writes `null` for a cleared one. Both are ordinary
    /// exports, and requiring a string here failed the whole import.
    #[serde(default)]
    value: Option<Value>,
    #[serde(default)]
    disabled: bool,
    /// `formdata` rows: "text" or "file"
    #[serde(default, rename = "type")]
    kind: Option<String>,
    /// `formdata` file rows: a path, or a list of paths
    #[serde(default)]
    src: Option<Value>,
}

impl PostmanKeyValue {
    fn value(&self) -> String {
        self.value
            .as_ref()
            .map_or_else(String::new, value_to_string)
    }
}

#[derive(Deserialize)]
struct PostmanBody {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    raw: Option<String>,
    #[serde(default)]
    urlencoded: Vec<PostmanKeyValue>,
    #[serde(default)]
    formdata: Vec<PostmanKeyValue>,
    /// `{"raw": {"language": "json"}}`: Postman sends the matching
    /// `Content-Type` itself, so the request file has to say it
    #[serde(default)]
    options: Option<Value>,
}

/// One imported Multipart field as one body line, never more and never a
/// live upload: a collection from someone else could otherwise smuggle in an
/// enabled `name=@~/.ssh/id_rsa` through a newline, an `=` in a key, or a
/// text value starting with `@` (written `@@`, Mercury's literal `@`). File
/// fields arrive switched off until the user ticks them. (Variables are
/// filled in per field at send time: [`crate::http::substitute_multipart`].)
fn multipart_line(key: &str, value: &str, file: bool) -> String {
    let flat = |s: &str| s.replace(['\r', '\n'], " ");
    let key = flat(key).replace('=', "%3D");
    let value = flat(value);
    if file {
        format!("# {key}=@{value}")
    } else if value.starts_with('@') {
        format!("{key}=@{value}")
    } else {
        format!("{key}={value}")
    }
}

/// A Postman body in Mercury's terms, plus the `Content-Type` it implies when
/// the request did not set one. The modes that come back `None` are the ones
/// Mercury has nowhere to put — a raw `file` body, GraphQL — so the import
/// can say what it could not
/// bring across, instead of writing an empty body under a `Content-Type` that
/// promises one.
fn postman_body(body: &PostmanBody) -> Option<(String, Option<&'static str>)> {
    match body.mode.as_deref() {
        Some("urlencoded") => {
            let rows: Vec<KeyValue> = body
                .urlencoded
                .iter()
                .filter(|r| !r.disabled)
                .map(|r| KeyValue::new(r.key.clone(), r.value()))
                .collect();
            Some((
                kv::build_form(&rows),
                Some("application/x-www-form-urlencoded"),
            ))
        }
        // Mercury's Multipart body: `name=value` lines, `name=@path` files.
        // File rows come in switched off: a collection from someone else
        // could point at ~/.ssh/id_rsa and post it anywhere on one Send.
        // The user ticks the row once they've seen which file it is.
        Some("formdata") => {
            let lines: Vec<String> = body
                .formdata
                .iter()
                .filter(|r| !r.disabled && !r.key.is_empty())
                .map(|r| match r.kind.as_deref() {
                    Some("file") => {
                        let src = match &r.src {
                            Some(Value::Array(a)) => a.first().map(value_to_string),
                            Some(v) => Some(value_to_string(v)),
                            None => None,
                        }
                        .unwrap_or_default();
                        multipart_line(&r.key, &src, true)
                    }
                    _ => multipart_line(&r.key, &r.value(), false),
                })
                .collect();
            Some((lines.join("\n"), Some("multipart/form-data")))
        }
        Some("file") | Some("graphql") => None,
        // "raw", or a body with no mode at all
        _ => {
            let language = body
                .options
                .as_ref()
                .and_then(|o| o.pointer("/raw/language"))
                .and_then(Value::as_str);
            let content_type = match language {
                Some("json") => Some("application/json"),
                Some("xml") => Some("application/xml"),
                Some("html") => Some("text/html"),
                Some("javascript") => Some("application/javascript"),
                Some("text") => Some("text/plain"),
                _ => None,
            };
            let raw = body.raw.clone().unwrap_or_default();
            Some((raw.clone(), content_type.filter(|_| !raw.is_empty())))
        }
    }
}

#[derive(Deserialize)]
struct PostmanAuth {
    #[serde(rename = "type")]
    kind: String,
    #[serde(flatten)]
    params: BTreeMap<String, Value>,
}

impl PostmanAuth {
    /// The auth parameters are a list of `{key, value}` under a field named
    /// after the type: `{"type": "bearer", "bearer": [{"key": "token", …}]}`.
    fn param(&self, name: &str) -> String {
        self.params
            .get(&self.kind)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|e| e.get("key").and_then(Value::as_str) == Some(name))
            .and_then(|e| e.get("value"))
            .map_or_else(String::new, value_to_string)
    }
}

/// Postman's auth as a header, or as a query parameter for an API key that
/// goes in the URL. `None` means Mercury cannot express it — OAuth 2, AWS
/// signatures, NTLM and the rest need a token exchange a request file cannot
/// describe, so the import reports them rather than dropping them quietly.
fn postman_auth(auth: &PostmanAuth) -> Option<Auth> {
    match auth.kind.as_str() {
        "noauth" => Some(Auth::None),
        "bearer" => Some(Auth::Header(
            "Authorization".into(),
            format!("Bearer {}", auth.param("token")),
        )),
        "basic" => Some(Auth::Header(
            "Authorization".into(),
            kv::basic_auth(&auth.param("username"), &auth.param("password")),
        )),
        "apikey" => {
            let (key, value) = (auth.param("key"), auth.param("value"));
            if auth.param("in") == "query" {
                Some(Auth::Query(key, value))
            } else {
                Some(Auth::Header(key, value))
            }
        }
        _ => None,
    }
}

enum Auth {
    None,
    Header(String, String),
    Query(String, String),
}

#[derive(Deserialize)]
struct PostmanVariable {
    key: String,
    value: Value,
}

/// Use `raw` when present, else rebuild from parts (disabled params dropped).
/// Path variables are filled in (`:userId` → `42`), or become `{{userId}}`
/// when the collection gives no value, so they can come from an environment.
fn postman_url(url: &PostmanUrl) -> String {
    let mut out = postman_url_text(url);
    if let PostmanUrl::Object { variable, .. } = url {
        for var in variable.iter().filter(|v| !v.key.is_empty()) {
            let value = var.value();
            let value = if value.is_empty() {
                format!("{{{{{}}}}}", var.key)
            } else {
                value
            };
            out = replace_path_var(&out, &var.key, &value);
        }
    }
    out
}

/// `:key` as a whole path segment (not `:keys`, not a port), in the path only.
fn replace_path_var(url: &str, key: &str, value: &str) -> String {
    let path_end = url.find(['?', '#']).unwrap_or(url.len());
    let (path, rest) = url.split_at(path_end);
    let needle = format!("/:{key}");
    let mut out = String::new();
    let mut tail = path;
    while let Some(at) = tail.find(&needle) {
        let after = &tail[at + needle.len()..];
        out.push_str(&tail[..at]);
        if after.is_empty() || after.starts_with('/') {
            out.push('/');
            out.push_str(value);
        } else {
            out.push_str(&needle);
        }
        tail = after;
    }
    out.push_str(tail);
    out + rest
}

fn postman_url_text(url: &PostmanUrl) -> String {
    match url {
        PostmanUrl::String(s) => s.clone(),
        PostmanUrl::Object { raw: Some(raw), .. } => raw.clone(),
        PostmanUrl::Object {
            protocol,
            host,
            path,
            query,
            ..
        } => {
            // (no raw: rebuilt from parts below)
            let host = if host.is_empty() {
                "localhost".to_string()
            } else {
                host.join(".")
            };
            let path: String = path.iter().map(|p| format!("/{}", url_encode(p))).collect();
            let query: Vec<String> = query
                .iter()
                .filter(|q| !q.disabled)
                .map(|q| format!("{}={}", url_encode(&q.key), url_encode(&q.value())))
                .collect();
            let query = if query.is_empty() {
                String::new()
            } else {
                format!("?{}", query.join("&"))
            };
            format!(
                "{}://{host}{path}{query}",
                protocol.as_deref().unwrap_or("https")
            )
        }
    }
}

fn postman_item(
    item: &PostmanItem,
    dir: &Path,
    inherited: Option<&PostmanAuth>,
    skipped: &mut Vec<String>,
) -> Result<usize, String> {
    // innermost auth wins: request, then folder, then collection
    let auth = item.auth.as_ref().or(inherited);
    if let Some(req) = &item.request {
        let auth = req.auth.as_ref().or(auth);
        let mut headers: BTreeMap<String, String> = req
            .header
            .iter()
            .filter(|h| !h.disabled)
            .map(|h| (h.key.clone(), h.value()))
            .collect();
        let mut url = postman_url(&req.url);
        match auth.map(postman_auth) {
            Some(Some(Auth::Header(name, value))) => {
                headers.insert(name, value);
            }
            Some(Some(Auth::Query(key, value))) => {
                let params = [KeyValue::new(key, value)];
                url = kv::build_url(
                    &url,
                    &[kv::parse_query_params(&url), params.to_vec()].concat(),
                );
            }
            Some(Some(Auth::None)) | None => {}
            Some(None) => skipped.push(format!(
                "{}: {} auth",
                item.name,
                auth.map_or("", |a| a.kind.as_str())
            )),
        }
        let body = match req.body.as_ref().map(postman_body) {
            Some(Some((body, content_type))) => {
                if let Some(ct) = content_type {
                    headers.entry("Content-Type".into()).or_insert(ct.into());
                }
                body
            }
            Some(None) => {
                let mode = req.body.as_ref().and_then(|b| b.mode.clone());
                skipped.push(format!("{}: {} body", item.name, mode.unwrap_or_default()));
                String::new()
            }
            None => String::new(),
        };
        write_request(dir, &item.name, &req.method, url, headers, body)?;
        return Ok(1);
    }
    if item.item.is_empty() {
        return Ok(0);
    }
    let folder = dir.join(sanitize_filename(&item.name));
    create_dir(&folder)?;
    item.item
        .iter()
        .map(|child| postman_item(child, &folder, auth, skipped))
        .sum()
}

/// A Postman environment export: what people keep their URLs and tokens in,
/// and a separate file from the collection.
#[derive(Deserialize)]
struct PostmanEnvironment {
    name: String,
    values: Vec<PostmanEnvValue>,
}

#[derive(Deserialize)]
struct PostmanEnvValue {
    key: String,
    #[serde(default)]
    value: Value,
    #[serde(default = "enabled")]
    enabled: bool,
}

fn enabled() -> bool {
    true
}

/// A Postman collection (v2.1) or environment export.
pub fn import_postman(path: &Path, out: &Path) -> Result<ImportCount, String> {
    let json: Value = serde_json::from_str(&read(path)?).map_err(|_| {
        "That file isn't JSON, so it isn't a Postman export. In Postman, use \
         Export on a collection (v2.1) or an environment."
            .to_string()
    })?;
    if json.get("values").is_some() && json.get("item").is_none() {
        let env: PostmanEnvironment = serde_json::from_value(json)
            .map_err(|e| format!("That Postman environment couldn't be read: {e}"))?;
        let vars: Vec<_> = env
            .values
            .iter()
            .filter(|v| v.enabled && !v.key.is_empty())
            .map(|v| (&v.key, &v.value))
            .collect();
        let name = write_env(out, &env.name, vars)?;
        return Ok(ImportCount {
            envs: 1,
            folder: out.to_path_buf(),
            env: Some(name),
            ..Default::default()
        });
    }
    if json.get("item").is_none() || json.get("info").is_none() {
        return Err(
            "That isn't a Postman collection or environment. In Postman, use \
             Export on a collection (Collection v2.1) or an environment."
                .into(),
        );
    }
    let collection: PostmanCollection = serde_json::from_value(json)
        .map_err(|e| format!("That Postman collection couldn't be read: {e}"))?;

    // (the folder first: the .env written next would make `out` look taken)
    let folder = import_folder(out, &collection.info.name)?;
    let mut env = None;
    if !collection.variable.is_empty() {
        let vars = collection.variable.iter().map(|v| (&v.key, &v.value));
        env = Some(write_env(out, &collection.info.name, vars)?);
    }
    let mut skipped = Vec::new();
    let requests = collection
        .item
        .iter()
        .map(|item| postman_item(item, &folder, collection.auth.as_ref(), &mut skipped))
        .sum::<Result<usize, String>>()?;
    Ok(ImportCount {
        requests,
        envs: usize::from(env.is_some()),
        skipped,
        folder,
        env,
    })
}

// ---------------------------------------------------------------------------
// Insomnia
// ---------------------------------------------------------------------------

/// Insomnia writes variables as `{{ _.base_url }}`; Mercury's are
/// `{{base_url}}`. Template tags (`{% response ... %}`, `{% uuid %}`) have no
/// Mercury equivalent and are left as they are (and reported).
fn insomnia_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    crate::vars::for_each_var(text, |name, range| {
        out.push_str(&text[at..range.start]);
        match name.strip_prefix("_.") {
            Some(name) => out.push_str(&format!("{{{{{name}}}}}")),
            None => out.push_str(&text[range.clone()]),
        }
        at = range.end;
    });
    out.push_str(&text[at..]);
    out
}

fn str_of<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

/// `{"api": {"url": …}}` (referenced as `_.api.url`) → `api.url`.
fn flatten_env(prefix: &str, data: &Value, out: &mut BTreeMap<String, Value>) {
    if let Value::Object(map) = data {
        for (k, v) in map {
            let key = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{prefix}.{k}")
            };
            match v {
                Value::Object(_) => flatten_env(&key, v, out),
                other => {
                    out.insert(key, other.clone());
                }
            }
        }
    }
}

/// One Insomnia request (a v4 `request` resource or a v5 collection item)
/// written as a Mercury request file into `dir`.
fn insomnia_request(req: &Value, dir: &Path, skipped: &mut Vec<String>) -> Result<(), String> {
    let name = str_of(req, "name");
    let mut headers: BTreeMap<String, String> = req
        .get("headers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|h| !h.get("disabled").and_then(Value::as_bool).unwrap_or(false))
        .filter(|h| !str_of(h, "name").is_empty())
        .map(|h| {
            (
                insomnia_text(str_of(h, "name")),
                insomnia_text(str_of(h, "value")),
            )
        })
        .collect();
    let mut url = insomnia_text(str_of(req, "url"));

    // authentication
    let auth = req.get("authentication").filter(|a| {
        !a.get("disabled").and_then(Value::as_bool).unwrap_or(false)
            && !str_of(a, "type").is_empty()
    });
    if let Some(auth) = auth {
        let text = |k: &str| insomnia_text(str_of(auth, k));
        match str_of(auth, "type") {
            "bearer" => {
                let prefix = match str_of(auth, "prefix") {
                    "" => "Bearer",
                    p => p,
                };
                headers.insert(
                    "Authorization".into(),
                    format!("{prefix} {}", text("token")),
                );
            }
            "basic" => {
                let (user, pass) = (text("username"), text("password"));
                if format!("{user}{pass}").contains("{{") {
                    // the header is base64 of user:pass, which hides a
                    // variable from substitution
                    skipped.push(format!("{name}: basic auth with variables"));
                } else {
                    headers.insert("Authorization".into(), kv::basic_auth(&user, &pass));
                }
            }
            "apikey" => {
                let (key, value) = (text("key"), text("value"));
                if str_of(auth, "addTo") == "queryParams" {
                    let params = [KeyValue::new(key, value)];
                    url = kv::build_url(
                        &url,
                        &[kv::parse_query_params(&url), params.to_vec()].concat(),
                    );
                } else if !key.is_empty() {
                    headers.insert(key, value);
                }
            }
            "none" => {}
            other => skipped.push(format!("{name}: {other} auth")),
        }
    }

    // body
    let body = req.get("body").cloned().unwrap_or(Value::Null);
    // the media type alone, as the sender compares it ("Multipart/Form-Data;
    // boundary=x" is multipart too)
    let mime = str_of(&body, "mimeType")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let params: Vec<&Value> = body
        .get("params")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|p| !p.get("disabled").and_then(Value::as_bool).unwrap_or(false))
        .filter(|p| !str_of(p, "name").is_empty())
        .collect();
    let text = match mime.as_str() {
        "application/x-www-form-urlencoded" => kv::build_form(
            &params
                .iter()
                .map(|p| {
                    KeyValue::new(
                        insomnia_text(str_of(p, "name")),
                        insomnia_text(str_of(p, "value")),
                    )
                })
                .collect::<Vec<_>>(),
        ),
        "multipart/form-data" => params
            .iter()
            .map(|p| {
                let key = insomnia_text(str_of(p, "name"));
                if str_of(p, "type") == "file" {
                    multipart_line(&key, str_of(p, "fileName"), true)
                } else {
                    multipart_line(&key, &insomnia_text(str_of(p, "value")), false)
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        "application/graphql" => {
            skipped.push(format!("{name}: GraphQL body"));
            String::new()
        }
        _ => insomnia_text(str_of(&body, "text")),
    };
    if !mime.is_empty() && !text.is_empty() {
        let has_type = headers
            .keys()
            .any(|k| k.eq_ignore_ascii_case("content-type"));
        if !has_type {
            headers.insert("Content-Type".into(), mime);
        }
    }
    if [url.as_str(), text.as_str()]
        .iter()
        .any(|t| t.contains("{%"))
        || headers.values().any(|v| v.contains("{%"))
    {
        skipped.push(format!("{name}: template tags ({{% … %}})"));
    }
    write_request(dir, name, str_of(req, "method"), url, headers, text)
}

/// Writes one `.env` per environment the user can pick: each sub-environment
/// on top of the base one, as Insomnia merges them, or the base alone when
/// there are none. Returns how many were written and the first one's name.
fn insomnia_envs(
    out: &Path,
    collection: &str,
    base: Option<&Value>,
    subs: &[(&str, &Value)],
) -> Result<(usize, Option<String>), String> {
    let mut base_vars = BTreeMap::new();
    if let Some(base) = base {
        flatten_env("", base, &mut base_vars);
    }
    if subs.is_empty() {
        if base_vars.is_empty() {
            return Ok((0, None));
        }
        let name = write_env(out, collection, &base_vars)?;
        return Ok((1, Some(name)));
    }
    let mut first = None;
    for (name, data) in subs {
        let mut vars = base_vars.clone();
        flatten_env("", data, &mut vars);
        let file = write_env(out, name, &vars)?;
        first.get_or_insert(file);
    }
    Ok((subs.len(), first))
}

/// An Insomnia export: v4 (JSON or YAML, `resources`) from Insomnia 9 and
/// earlier, or v5 (YAML, `collection`) from Insomnia 10 on.
pub fn import_insomnia(path: &Path, out: &Path) -> Result<ImportCount, String> {
    let content = read(path)?;
    let not_insomnia = "That isn't an Insomnia export. In Insomnia, use Export on a \
         collection (v5 YAML) or Export Data (v4 JSON).";
    let doc: Value = serde_json::from_str(&content)
        .or_else(|_| serde_yaml::from_str(&content))
        .map_err(|_| not_insomnia.to_string())?;
    if let Some(resources) = doc.get("resources").and_then(Value::as_array) {
        insomnia_v4(resources, out)
    } else if doc.get("collection").is_some()
        || str_of(&doc, "type").starts_with("collection.insomnia.rest/5")
    {
        insomnia_v5(&doc, out)
    } else {
        Err(not_insomnia.into())
    }
}

/// A v4 request's folder: its chain of request groups, outermost first.
fn insomnia_folder<'a>(
    by_id: &HashMap<&'a str, &'a Value>,
    root: &Path,
    mut id: &'a str,
) -> std::path::PathBuf {
    let mut parts = Vec::new();
    // (bounded: a malformed export could make the parents a cycle)
    while let Some(group) = by_id
        .get(id)
        .filter(|g| str_of(g, "_type") == "request_group" && parts.len() < 32)
    {
        parts.push(sanitize_filename(str_of(group, "name")));
        id = str_of(group, "parentId");
    }
    parts.reverse();
    parts.iter().fold(root.to_path_buf(), |dir, p| dir.join(p))
}

fn insomnia_v4(resources: &[Value], out: &Path) -> Result<ImportCount, String> {
    let by_id: HashMap<&str, &Value> = resources
        .iter()
        .filter_map(|r| r.get("_id").and_then(Value::as_str).map(|id| (id, r)))
        .collect();
    let workspace = resources
        .iter()
        .find(|r| str_of(r, "_type") == "workspace")
        .map_or("insomnia", |w| str_of(w, "name"));
    let folder = import_folder(out, workspace)?;

    let mut count = ImportCount::default();
    for req in resources.iter().filter(|r| str_of(r, "_type") == "request") {
        let dir = insomnia_folder(&by_id, &folder, str_of(req, "parentId"));
        create_dir(&dir)?;
        insomnia_request(req, &dir, &mut count.skipped)?;
        count.requests += 1;
    }

    // the base environment hangs off the workspace; sub-environments off it
    let envs: Vec<&Value> = resources
        .iter()
        .filter(|r| str_of(r, "_type") == "environment")
        .collect();
    let base = envs.iter().find(|e| {
        by_id
            .get(str_of(e, "parentId"))
            .is_none_or(|p| str_of(p, "_type") != "environment")
    });
    let base_id = base.map(|b| str_of(b, "_id")).unwrap_or("");
    let subs: Vec<(&str, &Value)> = envs
        .iter()
        .filter(|e| !base_id.is_empty() && str_of(e, "parentId") == base_id)
        .map(|e| (str_of(e, "name"), e.get("data").unwrap_or(&Value::Null)))
        .collect();
    let (n, env) = insomnia_envs(out, workspace, base.and_then(|b| b.get("data")), &subs)?;
    count.envs = n;
    count.env = env;
    count.folder = folder;
    Ok(count)
}

fn insomnia_v5(doc: &Value, out: &Path) -> Result<ImportCount, String> {
    let name = match str_of(doc, "name") {
        "" => "insomnia",
        n => n,
    };
    let folder = import_folder(out, name)?;
    let mut count = ImportCount::default();
    fn walk(items: &Value, dir: &Path, count: &mut ImportCount) -> Result<(), String> {
        for item in items.as_array().into_iter().flatten() {
            if let Some(children) = item.get("children") {
                let sub = dir.join(sanitize_filename(str_of(item, "name")));
                create_dir(&sub)?;
                walk(children, &sub, count)?;
            } else if item.get("url").is_some() || item.get("method").is_some() {
                insomnia_request(item, dir, &mut count.skipped)?;
                count.requests += 1;
            }
        }
        Ok(())
    }
    walk(
        doc.get("collection").unwrap_or(&Value::Null),
        &folder,
        &mut count,
    )?;

    let envs = doc.get("environments");
    let subs: Vec<(&str, &Value)> = envs
        .and_then(|e| e.get("subEnvironments"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|e| (str_of(e, "name"), e.get("data").unwrap_or(&Value::Null)))
        .collect();
    let (n, env) = insomnia_envs(out, name, envs.and_then(|e| e.get("data")), &subs)?;
    count.envs = n;
    count.env = env;
    count.folder = folder;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn import(
        f: fn(&Path, &Path) -> Result<ImportCount, String>,
        content: &str,
    ) -> (TempDir, Result<ImportCount, String>) {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("export");
        fs::write(&file, content).unwrap();
        let out = dir.path().join("out");
        fs::create_dir(&out).unwrap();
        let result = f(&file, &out);
        (dir, result)
    }

    fn read_out(dir: &TempDir, rel: &str) -> String {
        fs::read_to_string(dir.path().join("out").join(rel)).unwrap()
    }

    const POSTMAN: &str = r#"{
        "info": {"name": "Comprehensive API"},
        "item": [
            {"name": "Auth", "item": [{"name": "Login", "request": {
                "method": "POST",
                "header": [{"key": "Content-Type", "value": "application/json"},
                           {"key": "Off", "value": "x", "disabled": true}],
                "url": "{{host}}/auth/login",
                "body": {"mode": "raw", "raw": "{\"user\":\"test\"}"}}}]},
            {"name": "Users", "item": [{"name": "V1", "item": [{"name": "List", "request": {
                "method": "GET",
                "url": {"protocol": "https", "host": ["{{host}}"], "path": ["v1", "users", "{{id}}"],
                        "query": [{"key": "page", "value": "1"}, {"key": "x", "value": "y", "disabled": true}]}}}]}]},
            {"name": "Health", "request": {"method": "GET", "url": "{{host}}/health"}},
            {"name": "Empty folder", "item": []}
        ],
        "variable": [{"key": "host", "value": "api.test.com"}, {"key": "1 bad-key", "value": "has space"}]
    }"#;

    #[test]
    fn postman_writes_tree_env_and_requests() {
        let (dir, result) = import(import_postman, POSTMAN);
        let count = result.unwrap();
        assert_eq!((count.requests, count.envs), (3, 1));

        let login = RequestFile::from_json(&read_out(&dir, "auth/login.json")).unwrap();
        assert_eq!(login.method, HttpMethod::POST);
        assert_eq!(login.body, "{\"user\":\"test\"}");
        assert!(login.headers.contains_key("Content-Type") && !login.headers.contains_key("Off"));

        let list = RequestFile::from_json(&read_out(&dir, "users/v1/list.json")).unwrap();
        assert_eq!(list.url, "https://{{host}}/v1/users/{{id}}?page=1");

        let env = read_out(&dir, ".env.comprehensive-api");
        assert!(env.contains("host=api.test.com"));
        assert!(env.contains("1_bad-key=\"has space\""));
    }

    /// The shapes a real Postman export contains that a minimal fixture does
    /// not: a query param written with no `value` at all, collection- and
    /// request-level auth, and a body that is not `raw`. Missing any of them
    /// used to fail the whole import or drop the request's credentials.
    const POSTMAN_REAL: &str = r#"{
        "info": {"name": "Acme"},
        "auth": {"type": "bearer", "bearer": [{"key": "token", "value": "{{token}}"}]},
        "item": [
            {"name": "Login", "request": {
                "method": "POST",
                "header": [],
                "body": {"mode": "urlencoded", "urlencoded": [
                    {"key": "grant_type", "value": "password"},
                    {"key": "scope"},
                    {"key": "skip", "value": "x", "disabled": true}
                ]},
                "url": {"raw": "https://acme.test/token"}
            }},
            {"name": "List", "request": {
                "method": "GET",
                "auth": {"type": "apikey", "apikey": [
                    {"key": "key", "value": "X-Api-Key"},
                    {"key": "value", "value": "k-123"},
                    {"key": "in", "value": "header"}
                ]},
                "url": {"host": ["acme.test"], "path": ["users"],
                        "query": [{"key": "page", "value": "1"}, {"key": "archived"}]}
            }},
            {"name": "Upload", "request": {
                "method": "POST",
                "auth": {"type": "oauth2", "oauth2": []},
                "body": {"mode": "formdata", "formdata": [{"key": "file", "type": "file"},
                    {"key": "key", "type": "file", "src": "/home/me/.ssh/id_rsa"},
                    {"key": "note", "value": "hi"},
                    {"key": "a", "value": "x\nb=@/etc/passwd"},
                    {"key": "c=@/etc/hosts", "value": "1"},
                    {"key": "d", "value": "@/etc/shadow"}]},
                "url": {"raw": "https://acme.test/files"}
            }}
        ]
    }"#;

    #[test]
    fn postman_carries_auth_and_non_raw_bodies() {
        let (dir, result) = import(import_postman, POSTMAN_REAL);
        let count = result.expect("a valueless query param must not fail the import");
        assert_eq!(count.requests, 3);

        // a urlencoded body becomes Mercury's Form body, with the header that
        // makes the body type picker agree
        let login = RequestFile::from_json(&read_out(&dir, "login.json")).unwrap();
        assert_eq!(login.body, "grant_type=password&scope=");
        assert_eq!(
            login.headers.get("Content-Type").map(String::as_str),
            Some("application/x-www-form-urlencoded")
        );
        // collection-level auth reaches a request that declares none
        assert_eq!(
            login.headers.get("Authorization").map(String::as_str),
            Some("Bearer {{token}}")
        );

        // request-level auth wins over the collection's, and a valueless
        // param survives as a bare flag
        let list = RequestFile::from_json(&read_out(&dir, "list.json")).unwrap();
        assert_eq!(
            list.headers.get("X-Api-Key").map(String::as_str),
            Some("k-123")
        );
        assert!(!list.headers.contains_key("Authorization"));
        assert!(list.url.ends_with("?page=1&archived="), "got {}", list.url);

        // what Mercury cannot express is reported, not dropped in silence
        assert_eq!(count.skipped, vec!["Upload: oauth2 auth".to_string()]);
        // a formdata body becomes a Multipart one; file rows arrive switched
        // off, so an imported collection can't upload a local file unseen
        let upload = RequestFile::from_json(&read_out(&dir, "upload.json")).unwrap();
        // and nothing in a field can turn into a live upload: no newline
        // makes a second line, no `=` in a key moves the split, and a text
        // value that reads as a file is switched off too
        assert_eq!(
            upload.body,
            "# file=@\n# key=@/home/me/.ssh/id_rsa\nnote=hi\na=x b=@/etc/passwd\n\
             c%3D@/etc/hosts=1\nd=@@/etc/shadow"
        );
        let live: Vec<_> = kv::parse_lines(&upload.body, "=")
            .into_iter()
            .filter(|r| r.enabled && r.value.starts_with('@') && !r.value.starts_with("@@"))
            .collect();
        assert!(live.is_empty(), "{live:?}");
        assert_eq!(
            upload.headers.get("Content-Type").map(String::as_str),
            Some("multipart/form-data")
        );
    }

    #[test]
    fn postman_prefers_raw_url() {
        let url = PostmanUrl::Object {
            raw: Some("https://example.com/api".into()),
            protocol: None,
            host: vec![],
            path: vec![],
            query: vec![],
            variable: vec![],
        };
        assert_eq!(postman_url(&url), "https://example.com/api");
    }

    #[test]
    fn postman_invalid_json() {
        let (_dir, result) = import(import_postman, "NOT JSON");
        assert!(result.unwrap_err().contains("isn't JSON"));
        // JSON, but neither a collection nor an environment
        let (_dir, result) = import(import_postman, r#"{"resources": []}"#);
        assert!(result
            .unwrap_err()
            .contains("isn't a Postman collection or environment"));
    }

    const INSOMNIA_YAML: &str = r#"
resources:
  - _type: request_group
    _id: fld_1
    name: User API
  - _type: request
    name: Get ../../Secret?
    method: POST
    url: https://example.com/api
    headers:
      - {name: A, value: "1"}
      - {name: B, value: "2", disabled: true}
    body: {text: "{}"}
    parentId: fld_1
  - _type: request
    name: Root One
    method: GET
    url: https://example.com
    parentId: wrk_1
  - _type: environment
    name: Base Env
    data: {token: "a b", port: 8080}
  - _type: workspace
    name: ignored
"#;

    #[test]
    fn insomnia_yaml_sanitizes_names_and_groups_requests() {
        let (dir, result) = import(import_insomnia, INSOMNIA_YAML);
        let count = result.unwrap();
        assert_eq!((count.requests, count.envs), (2, 1));

        // a malicious name can't escape the output folder
        let req =
            RequestFile::from_json(&read_out(&dir, "user-api/get-..-..-secret.json")).unwrap();
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.body, "{}");
        // a request outside any folder stays at the top
        assert!(dir.path().join("out/root-one.json").exists());

        // a base environment with no sub-environments is named after the
        // workspace (it's the only one to pick)
        let env = read_out(&dir, ".env.ignored");
        assert_eq!(env, "port=8080\ntoken=\"a b\"\n");
    }

    #[test]
    fn insomnia_json_and_invalid() {
        let json =
            r#"{"resources": [{"_type": "request", "name": "T", "method": "GET", "url": "u"}]}"#;
        let count = import(import_insomnia, json).1.unwrap();
        assert_eq!((count.requests, count.envs), (1, 0));
        let err = import(import_insomnia, "NOT JSON OR YAML: [")
            .1
            .unwrap_err();
        assert!(err.contains("isn't an Insomnia export"), "{err}");
    }

    fn import_into(
        f: fn(&Path, &Path) -> Result<ImportCount, String>,
        content: &str,
        out: &Path,
    ) -> Result<ImportCount, String> {
        let file = out
            .parent()
            .unwrap()
            .join(format!("export-{}", out.display().to_string().len()));
        fs::write(&file, content).unwrap();
        f(&file, out)
    }

    fn file(path: &Path) -> RequestFile {
        RequestFile::from_json(&fs::read_to_string(path).unwrap()).unwrap()
    }

    /// QA (2026-11-17), Postman as Postman 11 exports it: path variables,
    /// a raw JSON body, and an import into a workspace that already has a
    /// request and an .env of the same names.
    #[test]
    fn postman_real_export_into_an_open_workspace() {
        let dir = TempDir::new().unwrap();
        let ws = dir.path().join("ws");
        fs::create_dir_all(ws.join("users")).unwrap();
        fs::write(ws.join("users/list-users.json"), "MINE").unwrap();
        fs::write(ws.join(".env.acme-api"), "baseUrl=MINE\n").unwrap();
        let export = r#"{"info": {"name": "Acme API"},
            "variable": [{"key": "baseUrl", "value": "http://localhost:3000"}],
            "item": [{"name": "Users", "item": [
                {"name": "List users", "request": {"method": "GET", "header": [], "url": {"raw": "{{baseUrl}}/users"}}},
                {"name": "List users", "request": {"method": "GET", "header": [], "url": {"raw": "{{baseUrl}}/users?all"}}},
                {"name": "Get user", "request": {"method": "GET", "header": [],
                    "url": {"raw": "{{baseUrl}}/users/:userId/posts/:postId?x=:userId", "variable": [
                        {"key": "userId", "value": "42"}, {"key": "postId"}]}}},
                {"name": "Create user", "request": {"method": "POST", "header": [],
                    "body": {"mode": "raw", "raw": "{\"a\": 1}", "options": {"raw": {"language": "json"}}},
                    "url": {"raw": "{{baseUrl}}/users"}}}
            ]}]}"#;
        let count = import_into(import_postman, export, &ws).unwrap();

        // nothing the user had is touched: the import gets its own folder,
        // its .env a free name, and a duplicate request name a suffix
        assert_eq!(
            fs::read_to_string(ws.join("users/list-users.json")).unwrap(),
            "MINE"
        );
        assert_eq!(
            fs::read_to_string(ws.join(".env.acme-api")).unwrap(),
            "baseUrl=MINE\n"
        );
        assert_eq!(count.folder, ws.join("acme-api"));
        assert_eq!(count.env.as_deref(), Some(".env.acme-api-2"));
        let users = ws.join("acme-api/users");
        assert!(users.join("list-users.json").exists() && users.join("list-users-2.json").exists());

        // path variables filled in (or left to the environment), in the path only
        assert_eq!(
            file(&users.join("get-user.json")).url,
            "{{baseUrl}}/users/42/posts/{{postId}}?x=:userId"
        );
        // Postman sends Content-Type for a raw JSON body itself
        let create = file(&users.join("create-user.json"));
        assert_eq!(
            create.headers.get("Content-Type").map(String::as_str),
            Some("application/json")
        );
    }

    #[test]
    fn postman_environment_export_becomes_an_env_file() {
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("ws");
        fs::create_dir_all(&out).unwrap();
        let export = r#"{"id": "x", "name": "Staging", "_postman_variable_scope": "environment",
            "values": [{"key": "baseUrl", "value": "https://stg.example", "enabled": true},
                       {"key": "token", "value": "s3 cret", "type": "secret", "enabled": true},
                       {"key": "off", "value": "x", "enabled": false}]}"#;
        let count = import_into(import_postman, export, &out).unwrap();
        assert_eq!(
            (count.requests, count.envs, count.env.as_deref()),
            (0, 1, Some(".env.staging"))
        );
        let env = crate::vars::parse_env(&fs::read_to_string(out.join(".env.staging")).unwrap());
        assert_eq!(env["baseUrl"], "https://stg.example");
        assert_eq!(env["token"], "s3 cret");
        assert!(!env.contains_key("off"));
    }

    const INSOMNIA_V4: &str = r#"{"_type":"export","__export_format":4,"resources":[
     {"_id":"wrk_1","_type":"workspace","name":"Acme API"},
     {"_id":"env_base","_type":"environment","name":"Base Environment","parentId":"wrk_1","data":{"base_url":"http://localhost:3000","token":"base","api":{"v":"2"}}},
     {"_id":"env_dev","_type":"environment","name":"Dev","parentId":"env_base","data":{"token":"dev"}},
     {"_id":"env_prod","_type":"environment","name":"Production","parentId":"env_base","data":{"base_url":"https://api.acme.example"}},
     {"_id":"fld_users","_type":"request_group","name":"Users","parentId":"wrk_1"},
     {"_id":"fld_admin","_type":"request_group","name":"Admin","parentId":"fld_users"},
     {"_id":"r1","_type":"request","name":"List users","parentId":"fld_users","method":"GET","url":"{{ _.base_url }}/v{{_.api.v}}/users","headers":[{"name":"Accept","value":"application/json"}],"authentication":{"type":"bearer","token":"{{ _.token }}"},"body":{}},
     {"_id":"r2","_type":"request","name":"Ban user","parentId":"fld_admin","method":"POST","url":"{{ _.base_url }}/ban","headers":[],"authentication":{"type":"basic","username":"admin","password":"pw"},"body":{"mimeType":"application/x-www-form-urlencoded","params":[{"name":"id","value":"42"},{"name":"off","value":"1","disabled":true}]}},
     {"_id":"r3","_type":"request","name":"Avatar","parentId":"wrk_1","method":"POST","url":"{{ _.base_url }}/avatar","headers":[],"authentication":{"type":"oauth2"},"body":{"mimeType":"multipart/form-data","params":[{"name":"note","value":"hi"},{"name":"img","type":"file","fileName":"/tmp/a.png"}]}}
    ]}"#;

    /// QA (2026-11-17): every one of these came across wrong or not at all.
    #[test]
    fn insomnia_v4_real_export() {
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("ws");
        fs::create_dir_all(&out).unwrap();
        let count = import_into(import_insomnia, INSOMNIA_V4, &out).unwrap();
        assert_eq!((count.requests, count.envs), (3, 2));
        assert_eq!(count.skipped, vec!["Avatar: oauth2 auth".to_string()]);

        // {{ _.x }} is Mercury's {{x}}; nested env data is flattened to a.b
        let list = file(&out.join("users/list-users.json"));
        assert_eq!(list.url, "{{base_url}}/v{{api.v}}/users");
        assert_eq!(
            list.headers.get("Authorization").map(String::as_str),
            Some("Bearer {{token}}")
        );
        // nested folders stay nested; auth and form bodies carried over
        let ban = file(&out.join("users/admin/ban-user.json"));
        assert_eq!(ban.body, "id=42");
        assert_eq!(
            ban.headers.get("Content-Type").map(String::as_str),
            Some("application/x-www-form-urlencoded")
        );
        assert_eq!(
            ban.headers.get("Authorization"),
            Some(&kv::basic_auth("admin", "pw"))
        );
        let avatar = file(&out.join("avatar.json"));
        assert_eq!(avatar.body, "note=hi\n# img=@/tmp/a.png");

        // each sub-environment includes the base one, as Insomnia merges them
        let dev = crate::vars::parse_env(&fs::read_to_string(out.join(".env.dev")).unwrap());
        assert_eq!(
            (
                dev["base_url"].as_str(),
                dev["token"].as_str(),
                dev["api.v"].as_str()
            ),
            ("http://localhost:3000", "dev", "2")
        );
        let prod =
            crate::vars::parse_env(&fs::read_to_string(out.join(".env.production")).unwrap());
        assert_eq!(
            (prod["base_url"].as_str(), prod["token"].as_str()),
            ("https://api.acme.example", "base")
        );
        assert!(!out.join(".env.base-environment").exists());
        assert_eq!(count.env.as_deref(), Some(".env.dev"));
    }

    /// However a body arrives (an unusual mimeType, a raw body under a
    /// multipart Content-Type), if it will be sent as Multipart no line of it
    /// is a live upload.
    #[test]
    fn no_import_path_yields_a_live_file_upload() {
        let insomnia = r#"{"resources":[
         {"_id":"a","_type":"request","name":"Odd mime","method":"POST","url":"https://x",
          "body":{"mimeType":"Multipart/Form-Data; boundary=x","params":[{"name":"f","type":"file","fileName":"/etc/passwd"}]}},
         {"_id":"b","_type":"request","name":"Raw under multipart","method":"POST","url":"https://x",
          "headers":[{"name":"content-type","value":"multipart/form-data"}],
          "body":{"mimeType":"text/plain","text":"note=hi\nf=@/etc/passwd\nt=@@literal"}}]}"#;
        let postman = r#"{"info":{"name":"P"},"item":[{"name":"Raw","request":{"method":"POST",
          "header":[{"key":"Content-Type","value":"multipart/form-data"}],
          "body":{"mode":"raw","raw":"f=@/etc/passwd"},"url":"https://x"}}]}"#;
        let dir = TempDir::new().unwrap();
        for (i, (f, content)) in [
            (import_insomnia as fn(&Path, &Path) -> _, insomnia),
            (import_postman, postman),
        ]
        .into_iter()
        .enumerate()
        {
            let out = dir.path().join(format!("ws{i}"));
            fs::create_dir_all(&out).unwrap();
            import_into(f, content, &out).unwrap();
            for entry in fs::read_dir(&out).unwrap().flatten() {
                let body = file(&entry.path()).body;
                let live: Vec<_> = kv::parse_lines(&body, "=")
                    .into_iter()
                    .filter(|r| r.enabled && r.value.starts_with('@') && !r.value.starts_with("@@"))
                    .collect();
                assert!(live.is_empty(), "{}: {body:?}", entry.path().display());
            }
        }
        // a Content-Type (or header name) that a variable decides, and a
        // newline smuggled into a header, count too
        let sneaky = r#"{"resources":[
         {"_id":"c","_type":"request","name":"Var type","method":"POST","url":"https://x",
          "headers":[{"name":"Content-Type","value":"{{ _.ct }}"}],
          "body":{"mimeType":"text/plain","text":"f=@/etc/passwd"}},
         {"_id":"d","_type":"request","name":"Var name","method":"POST","url":"https://x",
          "headers":[{"name":"{{ _.h }}","value":"multipart/form-data"}],
          "body":{"mimeType":"text/plain","text":"f=@/etc/passwd"}},
         {"_id":"e","_type":"request","name":"Newline","method":"POST","url":"https://x",
          "headers":[{"name":"X-A","value":"1\r\nContent-Type: multipart/form-data"}],
          "body":{"text":"f=@/etc/passwd"}}]}"#;
        let out = dir.path().join("ws-sneaky");
        fs::create_dir_all(&out).unwrap();
        import_into(import_insomnia, sneaky, &out).unwrap();
        for name in ["var-type", "var-name", "newline"] {
            let req = file(&out.join(format!("{name}.json")));
            let text = kv::file_headers_text(&req.headers, &req.disabled_headers);
            let sent = kv::headers_to_map(&text);
            let live = kv::parse_lines(&req.body, "=")
                .into_iter()
                .any(|r| r.enabled && r.value.starts_with('@') && !r.value.starts_with("@@"));
            // either the body can't be multipart once loaded, or nothing in it is live
            assert!(
                !(crate::http::is_multipart(&sent) && live),
                "{name}: {req:?}"
            );
            assert!(!live || name == "newline", "{name}: {:?}", req.body);
            assert!(sent.keys().all(|k| !k.contains('\n')));
        }
        // the odd mime type still counts as multipart
        let odd = file(&dir.path().join("ws0/odd-mime.json"));
        assert_eq!(odd.body, "# f=@/etc/passwd");
    }

    /// Insomnia 10+ exports v5 YAML, which used to fail outright.
    #[test]
    fn insomnia_v5_yaml_export() {
        let yaml = "type: collection.insomnia.rest/5.0
name: Acme API
collection:
  - name: Users
    children:
      - url: \"{{ _.base_url }}/users\"
        name: List users
        method: GET
        authentication:
          type: bearer
          token: \"{{ _.token }}\"
      - url: \"{{ _.base_url }}/users\"
        name: Create user
        method: POST
        body:
          mimeType: application/json
          text: '{\"name\": \"Ada\"}'
  - url: \"{{ _.base_url }}/health\"
    name: Health
    method: GET
environments:
  name: Base Environment
  data:
    base_url: http://localhost:3000
    token: base
  subEnvironments:
    - name: Dev
      data:
        token: dev
";
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("ws");
        fs::create_dir_all(&out).unwrap();
        let count = import_into(import_insomnia, yaml, &out).unwrap();
        assert_eq!((count.requests, count.envs), (3, 1));
        let list = file(&out.join("users/list-users.json"));
        assert_eq!(list.url, "{{base_url}}/users");
        assert_eq!(
            list.headers.get("Authorization").map(String::as_str),
            Some("Bearer {{token}}")
        );
        let create = file(&out.join("users/create-user.json"));
        assert_eq!(
            create.headers.get("Content-Type").map(String::as_str),
            Some("application/json")
        );
        assert!(out.join("health.json").exists());
        let dev = crate::vars::parse_env(&fs::read_to_string(out.join(".env.dev")).unwrap());
        assert_eq!(
            (dev["base_url"].as_str(), dev["token"].as_str()),
            ("http://localhost:3000", "dev")
        );
    }

    #[test]
    fn env_values_roundtrip_through_parse_env() {
        for value in ["plain", "has space", r#"{"a":1}"#, "a=b"] {
            let line = format!("K={}", escape_env_value(value));
            assert_eq!(crate::vars::parse_env(&line)["K"], value);
        }
    }

    #[test]
    fn sanitize_filename_cases() {
        assert_eq!(
            sanitize_filename("My API: v1/users?all"),
            "my-api-v1-users-all"
        );
        assert_eq!(sanitize_filename("back\\slash"), "back-slash");
        assert_eq!(sanitize_filename("what?"), "what");
        assert_eq!(sanitize_filename("???"), "untitled");
        assert_eq!(sanitize_filename(".hidden"), "hidden");
    }
}
