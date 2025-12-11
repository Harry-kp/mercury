---
title: Collections & Folders
sidebar_label: Collections
sidebar_position: 2
---

# Collections & Folders

> Organize your API requests into logical groups using folders. Mercury's file-based approach means your collection structure mirrors your file system.

## What is a Collection?

In Mercury, a **collection** is simply a folder containing `.http` files. There's no special format or database — just your file system.

```
my-api-project/         ← Workspace root (collection)
├── .env                ← Environment variables
├── auth/               ← Folder for auth-related requests
│   ├── login.http
│   └── register.http
├── users/              ← Folder for user endpoints
│   ├── get-user.http
│   ├── list-users.http
│   └── update-user.http
└── products/
    ├── list.http
    └── create.http
```

## Opening a Workspace

1. Launch Mercury
2. Click **Open Folder** or press `⌘+O`
3. Select any folder — this becomes your workspace

Mercury scans recursively for all `.http` files and displays them in the sidebar.

![Workspace sidebar - Replace with: Screenshot showing sidebar with folder tree and .http files](/img/screenshots/placeholder.png)

## Creating Folders

### From Mercury

1. Right-click in the sidebar
2. Select **New Folder**
3. Enter a name

### From Your File System

Just create a folder in your workspace directory. Mercury detects it immediately.

```bash
mkdir users
touch users/get-user.http
```

## Creating Requests in Folders

1. Right-click on a folder in the sidebar
2. Select **New Request**
3. Enter the request name

The `.http` file is created inside that folder.

## Folder Structure Best Practices

### By Resource Type

```
api-tests/
├── users/
├── products/
├── orders/
└── auth/
```

### By Environment

```
api-tests/
├── development/
├── staging/
└── production/
```

### By Feature

```
api-tests/
├── authentication/
├── checkout-flow/
├── search/
└── admin/
```

## Expanding and Collapsing

- Click the **arrow** next to a folder to expand/collapse
- Your expansion state persists between sessions

:::tip Keyboard Navigation
Use arrow keys to navigate the sidebar tree when focused.
:::

## Moving Requests

### Drag and Drop

Drag a request to a different folder to move it.

### Rename Path

Rename the file in your file system:
```bash
mv users/old-request.http products/new-request.http
```

Mercury updates automatically.

## Renaming

1. Right-click on a folder or request
2. Select **Rename**
3. Enter the new name

For folders, all child paths update automatically.

## Deleting

1. Right-click on a folder or request
2. Select **Delete**

:::warning
Deleting a folder removes all requests inside it. This action cannot be undone from within Mercury (but you can recover with `git checkout` if using version control).
:::

## Live File Sync

Mercury watches your workspace in real-time:

- **Add a file** → Appears in sidebar instantly
- **Delete a file** → Disappears from sidebar
- **Move a file** → Tree updates automatically
- **Edit in VS Code** → Changes sync to Mercury

No import/export ever needed!

![Live sync - Replace with: Screenshot or GIF showing file edited in VS Code updating in Mercury](/img/screenshots/placeholder.png)

## Git Integration

Since collections are just folders and files, Git works perfectly:

```bash
cd my-api-project
git init
git add .
git commit -m "Initial API collection"
```

Share collections with your team:
```bash
git clone https://github.com/your-team/api-collection.git
```

Open the cloned folder in Mercury — done!

## Searching Requests

Press `⌘+K` (Mac) or `Ctrl+K` (Windows/Linux) to open quick search:

- Search by request name
- Search by URL
- Jump directly to any request

## Related Features

- [Requests](/docs/features/requests) — Working with `.http` files
- [Environments](/docs/features/environments) — Manage environment variables
- [Import/Export](/docs/features/import-export) — Import from Postman or Insomnia
