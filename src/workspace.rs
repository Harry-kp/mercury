//! The workspace is a plain folder: sub-folders are collections, `*.json`
//! files are requests, `.env*` files in the root are environments.
//! Everything here is filesystem-only (no UI state).

use crate::model::{CollectionItem, RequestFile};
use notify_debouncer_mini::{new_debouncer, notify, DebounceEventResult, Debouncer};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub type Watcher = Debouncer<notify::RecommendedWatcher>;

fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Folders first, then requests, each sorted by name. Hidden entries are
/// skipped. Only folders in `expanded` are scanned recursively (lazy loading).
pub fn scan(dir: &Path, expanded: &HashSet<PathBuf>) -> Vec<CollectionItem> {
    scan_dir(dir, expanded, true)
}

/// [`scan`], optionally without opening each request to read its method.
fn scan_dir(dir: &Path, expanded: &HashSet<PathBuf>, methods: bool) -> Vec<CollectionItem> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    // the type read_dir already has, not a stat per entry; links followed
    let mut paths: Vec<(PathBuf, bool)> = entries
        .flatten()
        .map(|e| {
            let path = e.path();
            let dir = e
                .file_type()
                .is_ok_and(|t| t.is_dir() || (t.is_symlink() && path.is_dir()));
            (path, dir)
        })
        .collect();
    paths.sort();

    let mut folders = Vec::new();
    let mut requests = Vec::new();
    for (path, is_dir) in paths {
        let name = file_name(&path);
        if name.starts_with('.') {
            continue;
        }
        if is_dir {
            let children = expanded
                .contains(&path)
                .then(|| scan_dir(&path, expanded, methods));
            folders.push(CollectionItem::Folder {
                name,
                path,
                children,
            });
        } else if path.extension().is_some_and(|e| e == "json") {
            let method = if methods { request_method(&path) } else { None };
            requests.push(CollectionItem::Request { name, path, method });
        }
    }
    folders.extend(requests);
    folders
}

/// The method of the request file at `path`, if it parses.
pub fn request_method(path: &Path) -> Option<crate::model::HttpMethod> {
    let content = fs::read_to_string(path).ok()?;
    RequestFile::from_json(&content).ok().map(|r| r.method)
}

/// Every request in the workspace, depth-first — what the command palette
/// searches. Unlike [`scan`] this does not stop at collapsed folders, and it
/// leaves `method` as `None`: reading every file took ~50 ms at 2,000
/// requests. Use [`request_method`] for the few on screen.
pub fn all_requests(root: &Path) -> Vec<CollectionItem> {
    /// Deep enough for any real collection; stops runaway symlink loops.
    const MAX_DEPTH: usize = 16;

    fn walk(dir: &Path, depth: usize, out: &mut Vec<CollectionItem>) {
        if depth == 0 {
            return;
        }
        for item in scan_dir(dir, &HashSet::new(), false) {
            match item {
                CollectionItem::Folder { path, .. } => walk(&path, depth - 1, out),
                request => out.push(request),
            }
        }
    }

    let mut out = Vec::new();
    walk(root, MAX_DEPTH, &mut out);
    out
}

/// `.env*` files in the workspace root, sorted.
/// Symlinks are left out: a folder from someone else could link `.env.dev`
/// to `~/.aws/credentials` (whose `key = value` lines parse as variables a
/// request could then send), or to a file that editing it would overwrite.
pub fn env_files(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        // the entry's own type: not followed through a link
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| file_name(&e.path()))
        .filter(|n| n.starts_with(".env"))
        .collect();
    names.sort();
    names
}

/// A name the user typed is a single file name, never a path: `../x` would
/// write outside the folder they picked, and a leading `.` hides the result
/// from the tree (`scan` skips dot entries).
fn safe_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    // the path check comes first: "../x" is a path, not a dot-file
    if name.contains(['/', '\\']) || name.contains("..") {
        return Err("Name cannot contain a path".into());
    }
    if name.starts_with('.') {
        return Err("Name cannot start with a dot".into());
    }
    Ok(name)
}

fn ensure_free(path: &Path) -> Result<(), String> {
    if path.exists() {
        Err(format!("'{}' already exists", file_name(path)))
    } else {
        Ok(())
    }
}

/// Create `<parent>/<name>.json` with `content`; returns the new path.
pub fn create_request(parent: &Path, name: &str, content: &str) -> Result<PathBuf, String> {
    let name = safe_name(name)?;
    let file = if name.ends_with(".json") {
        name.to_string()
    } else {
        format!("{name}.json")
    };
    let path = parent.join(file);
    ensure_free(&path)?;
    fs::write(&path, content).map_err(|e| format!("Could not create '{name}': {e}"))?;
    Ok(path)
}

/// The path of environment `name`, refusing a symlink (see [`env_files`]);
/// checked at every read and write, not only when listing.
pub fn env_path(root: &Path, name: &str) -> Result<std::path::PathBuf, String> {
    if !name.starts_with(".env") || name.contains(['/', '\\']) || name.contains("..") {
        return Err(format!("{name} isn't an environment file name"));
    }
    let path = root.join(name);
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.file_type().is_symlink() => Err(format!(
            "{name} is a link to another file, so Mercury won't use it"
        )),
        _ => Ok(path),
    }
}

/// A new `.env.<name>` in the workspace root, holding `content`. Accepts
/// "staging", ".env.staging" or "env.staging"; never overwrites. Returns the
/// file name.
pub fn create_env(root: &Path, name: &str, content: &str) -> Result<String, String> {
    let name = name.trim();
    let name = name
        .strip_prefix(".env.")
        .or_else(|| name.strip_prefix("env."))
        .unwrap_or(name);
    let file = format!(".env.{}", safe_name(name)?);
    let path = root.join(&file);
    if path.exists() {
        return Err(format!("{file} already exists"));
    }
    fs::write(&path, content).map_err(|e| format!("Could not create {file}: {e}"))?;
    Ok(file)
}

pub fn create_folder(parent: &Path, name: &str) -> Result<(), String> {
    let path = parent.join(safe_name(name)?);
    ensure_free(&path)?;
    fs::create_dir(&path).map_err(|e| format!("Could not create folder '{name}': {e}"))
}

/// Rename in place (same parent); returns the new path.
pub fn rename(path: &Path, new_name: &str) -> Result<PathBuf, String> {
    let mut name = safe_name(new_name)?.to_string();
    // a request renamed to "foo" must stay "foo.json" or it leaves the tree
    if path.extension().is_some_and(|e| e == "json") && !name.ends_with(".json") {
        name.push_str(".json");
    }
    let new_path = path.with_file_name(name);
    ensure_free(&new_path)?;
    fs::rename(path, &new_path).map_err(|e| format!("Could not rename: {e}"))?;
    Ok(new_path)
}

/// What deleting `dir` takes with it: `(requests, other files)`, counted
/// recursively, hidden files included. A folder delete is permanent, so the
/// confirmation has to say more than the folder's name.
pub fn count_contents(dir: &Path) -> (usize, usize) {
    let mut counts = (0, 0);
    for path in fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
    {
        if path.is_dir() {
            let (r, o) = count_contents(&path);
            counts = (counts.0 + r, counts.1 + o);
        } else if path.extension().is_some_and(|e| e == "json") {
            counts.0 += 1;
        } else {
            counts.1 += 1;
        }
    }
    counts
}

pub fn delete(path: &Path) -> Result<(), String> {
    let result = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    result.map_err(|e| format!("Could not delete '{}': {e}", file_name(path)))
}

/// Copy `name.json` to the first free `name_copyN.json`.
pub fn duplicate(path: &Path) -> Result<PathBuf, String> {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    let new_path = (1..)
        .map(|n| path.with_file_name(format!("{stem}_copy{n}.{ext}")))
        .find(|p| !p.exists())
        .expect("an unused name exists");
    fs::copy(path, &new_path).map_err(|e| format!("Could not duplicate: {e}"))?;
    Ok(new_path)
}

/// Watch `root` recursively; `on_change` gets `Ok` after a debounced batch of
/// changes or `Err` on watcher failure. Drop the returned value to stop.
pub fn watch(
    root: &Path,
    on_change: impl Fn(Result<(), String>) + Send + 'static,
) -> Result<Watcher, String> {
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |result: DebounceEventResult| match result {
            Ok(events) if !events.is_empty() => on_change(Ok(())),
            Ok(_) => {}
            Err(e) => on_change(Err(format!("File watcher error: {e}"))),
        },
    )
    .map_err(|e| format!("Failed to create file watcher: {e}"))?;
    debouncer
        .watcher()
        .watch(root, notify::RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch workspace: {e}"))?;
    Ok(debouncer)
}

#[cfg(test)]
mod tests {
    /// `../x` is a path; saying "cannot start with a dot" points at the
    /// wrong rule.
    #[test]
    fn a_path_is_reported_as_a_path() {
        assert_eq!(safe_name("../x").unwrap_err(), "Name cannot contain a path");
        assert_eq!(
            safe_name(".env").unwrap_err(),
            "Name cannot start with a dot"
        );
    }

    #[test]
    fn count_contents_sees_nested_requests_and_other_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("a/b")).unwrap();
        for f in [
            "x.json",
            "a/y.json",
            "a/b/z.json",
            "a/README.md",
            "a/b/.fixture",
        ] {
            fs::write(root.join(f), "{}").unwrap();
        }
        assert_eq!(count_contents(root), (3, 2));
        assert_eq!(count_contents(&root.join("a/b")), (1, 1));
    }

    #[cfg(unix)]
    #[test]
    fn environment_files_are_never_symlinks() {
        let dir = TempDir::new().unwrap();
        let secret = dir.path().join("credentials");
        fs::write(&secret, "aws_secret_access_key = abc\n").unwrap();
        fs::write(dir.path().join(".env.dev"), "A=1").unwrap();
        std::os::unix::fs::symlink(&secret, dir.path().join(".env.prod")).unwrap();
        assert_eq!(env_files(dir.path()), vec![".env.dev"]);
        assert!(env_path(dir.path(), ".env.prod")
            .unwrap_err()
            .contains("link"));
        assert!(env_path(dir.path(), ".env.dev").is_ok());
    }

    #[test]
    fn create_env_takes_any_spelling_and_never_overwrites() {
        let dir = TempDir::new().unwrap();
        assert_eq!(
            create_env(dir.path(), " staging ", "A=1\n").unwrap(),
            ".env.staging"
        );
        assert_eq!(
            fs::read_to_string(dir.path().join(".env.staging")).unwrap(),
            "A=1\n"
        );
        assert_eq!(
            create_env(dir.path(), ".env.prod", "").unwrap(),
            ".env.prod"
        );
        assert_eq!(create_env(dir.path(), "env.qa", "").unwrap(), ".env.qa");
        assert!(create_env(dir.path(), "staging", "B=2")
            .unwrap_err()
            .contains("already exists"));
        assert_eq!(
            fs::read_to_string(dir.path().join(".env.staging")).unwrap(),
            "A=1\n"
        );
        assert!(create_env(dir.path(), "../x", "").is_err());
        assert!(create_env(dir.path(), "", "").is_err());
    }

    #[test]
    fn names_the_user_types_cannot_escape_their_folder() {
        let dir = tempfile::TempDir::new().unwrap();
        let inside = dir.path().join("collection");
        fs::create_dir(&inside).unwrap();

        for bad in ["../escaped", "a/b", "a\\b", "..", "", "   ", ".hidden"] {
            assert!(
                create_request(&inside, bad, "{}").is_err(),
                "create_request accepted {bad:?}"
            );
            assert!(
                create_folder(&inside, bad).is_err(),
                "create_folder accepted {bad:?}"
            );
        }
        assert!(!dir.path().join("escaped.json").exists());

        let file = create_request(&inside, "ok", "{}").unwrap();
        assert!(rename(&file, "../escaped").is_err());
        assert!(!dir.path().join("escaped.json").exists());
        assert!(rename(&file, "renamed").is_ok());
    }

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn scan_orders_folders_first_and_loads_lazily() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::create_dir(root.join("users")).unwrap();
        fs::write(root.join("users/get.json"), r#"{"method":"GET","url":"u"}"#).unwrap();
        fs::write(root.join("a.json"), r#"{"method":"POST","url":"u"}"#).unwrap();
        fs::write(root.join("broken.json"), "nope").unwrap();
        fs::write(root.join("notes.txt"), "").unwrap();
        fs::write(root.join(".env"), "").unwrap();

        let tree = scan(root, &HashSet::new());
        assert_eq!(tree.len(), 3);
        assert!(matches!(
            &tree[0],
            CollectionItem::Folder { children: None, .. }
        ));
        assert!(matches!(
            &tree[1],
            CollectionItem::Request {
                method: Some(_),
                ..
            }
        ));
        assert!(matches!(
            &tree[2],
            CollectionItem::Request { method: None, .. }
        ));

        let expanded = HashSet::from([root.join("users")]);
        let CollectionItem::Folder { children, .. } = &scan(root, &expanded)[0] else {
            panic!("folder first")
        };
        assert_eq!(children.as_ref().unwrap().len(), 1);

        // a linked folder is still a folder (types come from read_dir now,
        // which doesn't follow links on its own)
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("users"), root.join("linked")).unwrap();
            assert!(matches!(
                &scan(root, &HashSet::new())[0],
                CollectionItem::Folder { name, .. } if name == "linked"
            ));
            assert_eq!(all_requests(root).len(), 4, "a.json, broken, both get.json");
        }
    }

    #[test]
    fn env_files_only_from_root() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".env.prod"), "").unwrap();
        fs::write(dir.path().join(".env"), "").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/.env.nested"), "").unwrap();
        assert_eq!(env_files(dir.path()), vec![".env", ".env.prod"]);
    }

    #[test]
    fn file_operations() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let path = create_request(root, "login", "{}").unwrap();
        assert_eq!(path, root.join("login.json"));
        assert!(create_request(root, "login.json", "{}").is_err());

        assert_eq!(duplicate(&path).unwrap(), root.join("login_copy1.json"));
        assert_eq!(duplicate(&path).unwrap(), root.join("login_copy2.json"));

        let renamed = rename(&path, "signin").unwrap();
        assert_eq!(renamed, root.join("signin.json"));
        assert!(renamed.exists() && !path.exists());
        assert!(rename(&renamed, "login_copy1.json").is_err());

        create_folder(root, "auth").unwrap();
        assert!(create_folder(root, "auth").is_err());
        delete(&root.join("auth")).unwrap();
        delete(&renamed).unwrap();
        assert!(!root.join("auth").exists() && !renamed.exists());
    }
}
