//! Import Postman (v2.1 JSON) and Insomnia (JSON/YAML) exports into a
//! workspace folder as Mercury request files + `.env.<name>` files.

use crate::kv::url_encode;
use crate::model::{HttpMethod, RequestFile};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

/// (requests written, env files written)
pub type ImportCount = (usize, usize);

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

fn sanitize_env_key(key: &str) -> String {
    let key: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if key.starts_with(|c: char| c.is_ascii_digit()) {
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
}

#[derive(Deserialize)]
struct PostmanRequest {
    method: String,
    #[serde(default)]
    header: Vec<PostmanKeyValue>,
    url: PostmanUrl,
    #[serde(default)]
    body: Option<PostmanBody>,
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
    key: String,
    value: String,
    #[serde(default)]
    disabled: bool,
}

#[derive(Deserialize)]
struct PostmanBody {
    #[serde(default)]
    raw: Option<String>,
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
                .map(|q| format!("{}={}", url_encode(&q.key), url_encode(&q.value)))
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

fn postman_item(item: &PostmanItem, dir: &Path) -> Result<usize, String> {
    if let Some(req) = &item.request {
        let headers = req
            .header
            .iter()
            .filter(|h| !h.disabled)
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect();
        let body = req
            .body
            .as_ref()
            .and_then(|b| b.raw.clone())
            .unwrap_or_default();
        write_request(
            dir,
            &item.name,
            &req.method,
            postman_url(&req.url),
            headers,
            body,
        )?;
        return Ok(1);
    }
    if item.item.is_empty() {
        return Ok(0);
    }
    let folder = dir.join(sanitize_filename(&item.name));
    create_dir(&folder)?;
    item.item
        .iter()
        .map(|child| postman_item(child, &folder))
        .sum()
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
    let requests = collection
        .item
        .iter()
        .map(|item| postman_item(item, out))
        .sum::<Result<usize, String>>()?;
    Ok((requests, envs))
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
    Ok((requests, envs))
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
        assert_eq!(result.unwrap(), (3, 1));

        let login = RequestFile::from_json(&read_out(&dir, "auth/login.json")).unwrap();
        assert_eq!(login.method, HttpMethod::POST);
        assert_eq!(login.body, "{\"user\":\"test\"}");
        assert!(login.headers.contains_key("Content-Type") && !login.headers.contains_key("Off"));

        let list = RequestFile::from_json(&read_out(&dir, "users/v1/list.json")).unwrap();
        assert_eq!(list.url, "https://{{host}}/v1/users/{{id}}?page=1");

        let env = read_out(&dir, ".env.comprehensive-api");
        assert!(env.contains("host=api.test.com"));
        assert!(env.contains("_1_bad_key=\"has space\""));
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
        assert_eq!(result.unwrap(), (2, 1));

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
        assert_eq!(import(import_insomnia, json).1.unwrap(), (1, 0));
        let err = import(import_insomnia, "NOT JSON OR YAML: [")
            .1
            .unwrap_err();
        assert!(err.contains("Failed to parse as JSON") && err.contains("or YAML"));
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
