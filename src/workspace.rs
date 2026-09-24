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
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();

    let mut folders = Vec::new();
    let mut requests = Vec::new();
    for path in paths {
        let name = file_name(&path);
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            let children = expanded.contains(&path).then(|| scan(&path, expanded));
            folders.push(CollectionItem::Folder {
                name,
                path,
                children,
            });
        } else if path.extension().is_some_and(|e| e == "json") {
            let method = fs::read_to_string(&path)
                .ok()
                .and_then(|c| RequestFile::from_json(&c).ok())
                .map(|r| r.method);
            requests.push(CollectionItem::Request { name, path, method });
        }
    }
    folders.extend(requests);
    folders
}

/// `.env*` files in the workspace root, sorted.
pub fn env_files(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| file_name(&e.path()))
        .filter(|n| n.starts_with(".env"))
        .collect();
    names.sort();
    names
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
    let name = name.trim();
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

pub fn create_folder(parent: &Path, name: &str) -> Result<(), String> {
    let path = parent.join(name.trim());
    ensure_free(&path)?;
    fs::create_dir(&path).map_err(|e| format!("Could not create folder '{name}': {e}"))
}

/// Rename in place (same parent); returns the new path.
pub fn rename(path: &Path, new_name: &str) -> Result<PathBuf, String> {
    let mut name = new_name.trim().to_string();
    // a request renamed to "foo" must stay "foo.json" or it leaves the tree
    if path.extension().is_some_and(|e| e == "json") && !name.ends_with(".json") {
        name.push_str(".json");
    }
    let new_path = path.with_file_name(name);
    ensure_free(&new_path)?;
    fs::rename(path, &new_path).map_err(|e| format!("Could not rename: {e}"))?;
    Ok(new_path)
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
