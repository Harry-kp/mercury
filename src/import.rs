//! Import Postman (v2.1 JSON) and Insomnia (JSON/YAML) exports into a
//! workspace folder as Mercury request files + `.env.<name>` files.

use crate::kv::{self, url_encode, KeyValue};
use crate::model::{HttpMethod, RequestFile};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

/// (requests written, env files written)
/// What an import produced, and what it could not bring across.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportCount {
    pub requests: usize,
    pub envs: usize,
    /// One line per request whose auth or body Mercury cannot express.
    pub skipped: Vec<String>,
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
    let request = RequestFile {
        method: HttpMethod::parse(method).unwrap_or_default(),
        url,
        headers,
        body,
    };
    let path = dir.join(format!("{}.json", sanitize_filename(name)));
    write_file(&path, request.to_json())
}

fn write_env<'a>(
    dir: &Path,
    name: &str,
    vars: impl IntoIterator<Item = (&'a String, &'a Value)>,
) -> Result<(), String> {
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
    write_file(
        &dir.join(format!(".env.{}", sanitize_filename(name))),
        content,
    )
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
    graphql: Option<PostmanGraphql>,
}

#[derive(Deserialize)]
struct PostmanGraphql {
    #[serde(default)]
    query: String,
    #[serde(default)]
    variables: Option<Value>,
}

/// A Postman body in Mercury's terms, plus the `Content-Type` it implies when
/// the request did not set one. `formdata` and `file` need an attached file,
/// which a request file cannot reference — those come back `None` so the
/// import can say what it could not bring across, instead of writing an empty
/// body under a `Content-Type` that promises one.
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
        Some("graphql") => {
            let g = body.graphql.as_ref()?;
            let payload = serde_json::json!({
                "query": g.query,
                "variables": g.variables.clone().unwrap_or(Value::Object(Default::default())),
            });
            Some((
                serde_json::to_string_pretty(&payload).ok()?,
                Some("application/json"),
            ))
        }
        Some("formdata") | Some("file") => None,
        // "raw", or a body with no mode at all
        _ => Some((body.raw.clone().unwrap_or_default(), None)),
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
fn postman_url(url: &PostmanUrl) -> String {
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

/// Auth and bodies Mercury cannot express, named so the import can say what
/// it left behind instead of writing a request that quietly fails.
#[derive(Default)]
struct Skipped(Vec<String>);

impl Skipped {
    fn note(&mut self, request: &str, what: &str) {
        self.0.push(format!("{request}: {what}"));
    }
}

fn postman_item(
    item: &PostmanItem,
    dir: &Path,
    inherited: Option<&PostmanAuth>,
    skipped: &mut Skipped,
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
            Some(None) => skipped.note(&item.name, &format!("{} auth", auth_kind(auth))),
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
                skipped.note(&item.name, &format!("{} body", mode.unwrap_or_default()));
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

fn auth_kind(auth: Option<&PostmanAuth>) -> &str {
    auth.map_or("", |a| a.kind.as_str())
}

pub fn import_postman(path: &Path, out: &Path) -> Result<ImportCount, String> {
    let collection: PostmanCollection =
        serde_json::from_str(&read(path)?).map_err(|e| format!("Postman import failed: {e}"))?;

    let mut envs = 0;
    if !collection.variable.is_empty() {
        let vars = collection.variable.iter().map(|v| (&v.key, &v.value));
        write_env(out, &collection.info.name, vars)?;
        envs = 1;
    }
    let mut skipped = Skipped::default();
    let requests = collection
        .item
        .iter()
        .map(|item| postman_item(item, out, collection.auth.as_ref(), &mut skipped))
        .sum::<Result<usize, String>>()?;
    Ok(ImportCount {
        requests,
        envs,
        skipped: skipped.0,
    })
}

// ---------------------------------------------------------------------------
// Insomnia
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct InsomniaExport {
    resources: Vec<InsomniaResource>,
}

#[derive(Deserialize)]
#[serde(tag = "_type")]
enum InsomniaResource {
    #[serde(rename = "request")]
    Request {
        name: String,
        method: String,
        url: String,
        #[serde(default)]
        headers: Vec<InsomniaHeader>,
        #[serde(default)]
        body: Option<InsomniaBody>,
        #[serde(rename = "parentId")]
        parent_id: Option<String>,
    },
    #[serde(rename = "request_group")]
    Group {
        #[serde(rename = "_id")]
        id: String,
        name: String,
    },
    #[serde(rename = "environment")]
    Environment {
        name: String,
        data: HashMap<String, Value>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct InsomniaHeader {
    name: String,
    value: String,
    #[serde(default)]
    disabled: bool,
}

#[derive(Deserialize)]
struct InsomniaBody {
    text: Option<String>,
}

/// Requests go into one folder per request group (flat), or `imported/`.
pub fn import_insomnia(path: &Path, out: &Path) -> Result<ImportCount, String> {
    let content = read(path)?;
    let export: InsomniaExport = serde_json::from_str(&content).or_else(|json_err| {
        serde_yaml::from_str(&content).map_err(|yaml_err| {
            format!(
                "Insomnia import failed: Failed to parse as JSON ({json_err}) or YAML ({yaml_err})"
            )
        })
    })?;

    let groups: HashMap<&str, &str> = export
        .resources
        .iter()
        .filter_map(|r| match r {
            InsomniaResource::Group { id, name } => Some((id.as_str(), name.as_str())),
            _ => None,
        })
        .collect();

    let (mut requests, mut envs) = (0, 0);
    for resource in &export.resources {
        match resource {
            InsomniaResource::Environment { name, data } if !data.is_empty() => {
                let mut vars: Vec<_> = data.iter().collect();
                vars.sort_by_key(|(k, _)| *k);
                write_env(out, name, vars)?;
                envs += 1;
            }
            InsomniaResource::Request {
                name,
                method,
                url,
                headers,
                body,
                parent_id,
            } => {
                let group = parent_id
                    .as_deref()
                    .and_then(|id| groups.get(id))
                    .map_or("imported".to_string(), |g| sanitize_filename(g));
                let folder = out.join(group);
                create_dir(&folder)?;
                let headers = headers
                    .iter()
                    .filter(|h| !h.disabled)
                    .map(|h| (h.name.clone(), h.value.clone()))
                    .collect();
                let body = body
                    .as_ref()
                    .and_then(|b| b.text.clone())
                    .unwrap_or_default();
                write_request(&folder, name, method, url.clone(), headers, body)?;
                requests += 1;
            }
            _ => {}
        }
    }
    Ok(ImportCount {
        requests,
        envs,
        skipped: Vec::new(),
    })
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
                "body": {"mode": "formdata", "formdata": [{"key": "file", "type": "file"}]},
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
        assert_eq!(
            count.skipped,
            vec![
                "Upload: oauth2 auth".to_string(),
                "Upload: formdata body".to_string()
            ]
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
        };
        assert_eq!(postman_url(&url), "https://example.com/api");
    }

    #[test]
    fn postman_invalid_json() {
        let (_dir, result) = import(import_postman, "NOT JSON");
        assert!(result.unwrap_err().contains("Postman import failed"));
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
        assert!(dir.path().join("out/imported/root-one.json").exists());

        let env = read_out(&dir, ".env.base-env");
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
        assert!(err.contains("Failed to parse as JSON") && err.contains("or YAML"));
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
