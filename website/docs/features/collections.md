---
title: Collections & Folders
sidebar_label: Collections
sidebar_position: 2
---

# Collections & Folders

Mercury has no collection format of its own. A **workspace** is a folder on disk, its **subfolders are collections**, and the **`*.json` files are requests**.

```
my-api/                 ← workspace (the folder you open)
├── .env                ← environments: .env* files in the root only
├── .env.production
├── auth/               ← collection
│   ├── login.json
│   └── register.json
├── users/
│   ├── admin/          ← collections can nest
│   │   └── list-admins.json
│   └── get-user.json
└── health.json
```

## Opening a workspace

Press `⌘ O`, choose **Open → Open Folder...**, or click **Open a folder** in the empty sidebar. Mercury remembers the workspace and reopens it on the next launch.

## The sidebar tree

- Folders come first, then requests, each sorted by name.
- Only `.json` files are listed. Other files are ignored.
- Hidden files and folders (names starting with `.`) are skipped.
- Each request shows its method. If a `.json` file isn't a valid request, it's listed without a method and shows an error when you open it.
- Mercury reads a folder the first time you expand it. Clicking a folder expands or collapses it.

Unsaved requests you've sent appear above the tree under **Recent** (up to 50). Click one to load it, or click × to remove it. When you save a request, it's removed from Recent.

## Searching

The search box in the top bar (`⌘ K`) filters the tree by file and folder name. The search is case-insensitive, and matching folders expand while you search. `Esc` clears the search.

## Creating, renaming and deleting

Right-click a **folder**:

| Action | What it does |
|--------|--------------|
| **New Request** | Saves the current editor contents as `<name>.json` in this folder |
| **New Folder** | Creates a subfolder |
| **Rename** | Renames the folder in place |
| **Delete** | Deletes the folder and everything in it, after you confirm |
| **Copy Path** | Copies the folder's full path |

Requests have **Duplicate**, **Rename**, **Delete** and **Copy Path**. See [Sidebar actions](/docs/features/requests#sidebar-actions).

To create a folder directly in the workspace root, use your file manager or a terminal (`mkdir users`). Mercury picks up the change.

:::warning
Deleting is permanent. Nothing goes to the trash. If the workspace is in Git, you can restore files with `git checkout`.
:::

## Changes made outside Mercury

Mercury watches the workspace folder. When files are added, removed or renamed, whether by an editor, `git pull` or a script, the tree and the environment list refresh within about half a second. If the open request's file changes and you have no unsaved edits, Mercury reloads it. If you do have unsaved edits, Mercury keeps yours, warns you, and saves over the file.

## Sharing with Git

A workspace is just files, so version control works as usual:

```bash
cd my-api
git init
git add .
git commit -m "API requests"
```

Mercury writes headers sorted by name, so diffs stay small. Keep secrets out of the repository. See [Environments](/docs/features/environments#keeping-secrets-out-of-git).

## Related

- [Requests](/docs/features/requests)
- [Environments](/docs/features/environments)
- [Import & Export](/docs/features/import-export)
