---
title: Quick Start
sidebar_label: Quick Start
sidebar_position: 2
---

# Quick Start

This page walks through sending, saving and organizing requests. It assumes Mercury is [installed](/docs/getting-started).

## The layout

| Area | What it shows |
|------|---------------|
| **Top bar** | Breadcrumb (`workspace / folder / METHOD name`), the search box, the **Help** and **Open** menus, and the environment picker |
| **Sidebar** (left) | **Recent** unsaved requests and the workspace folder tree |
| **Editor** (center) | Method, URL bar, and the **Body**, **Params**, **Headers** and **Auth** tabs |
| **Response** (right) | Status, time, size, headers, cookies and body, or the history list |
| **Status bar** | Notifications, the workspace name and a **? Shortcuts** link |

## 1. Send a request

You don't need a workspace to send a request.

1. Click the URL bar (or press `⌘ L`) and type `https://httpbin.org/get`.
2. Press `⌘ Enter`, or click the send button.

The response panel shows the status, response time and size, followed by the body. JSON is pretty-printed and highlighted. While a request is running, press `Esc` or click the stop button to cancel it.

Requests you send without saving appear under **Recent** in the sidebar. Click one to load it again.

:::tip
You can paste a cURL command into the URL bar. Mercury fills in the method, URL, headers and body from it. See [Import & Export](/docs/features/import-export#import-from-curl).
:::

On Windows and Linux, `⌘` means `Ctrl`.

## 2. Open a workspace

A workspace is any folder. Mercury shows its subfolders and `.json` request files in the sidebar.

1. Press `⌘ O`, or choose **Open → Open Folder...**
2. Pick or create a folder.

Mercury watches the folder, so files you add or edit in another editor show up in the sidebar.

## 3. Save the request

Press `⌘ S`. Mercury asks for a name and writes `<name>.json` to the workspace root:

```json
{
  "method": "GET",
  "url": "https://httpbin.org/get"
}
```

After that, Mercury saves changes to the open file automatically. See [Saving](/docs/features/requests#saving).

To save into a particular folder, right-click the folder in the sidebar and choose **New Request**. This saves the current editor contents into that folder under the name you give it.

## 4. Add an environment

Create a `.env` file in the workspace root:

```bash
BASE_URL=https://httpbin.org
TOKEN=secret-token
```

Choose it in the environment picker at the top right, then use the variables anywhere in the request:

```
{{BASE_URL}}/bearer
```

If the selected environment is missing a variable you use, the URL bar border turns amber. Hover over the URL bar to see which variables are missing. See [Environments](/docs/features/environments).

## 5. Organize with folders

Subfolders of the workspace are collections:

```
my-api/
├── .env
├── users/
│   ├── list-users.json
│   └── create-user.json
└── health.json
```

Right-click a folder for **New Request**, **New Folder**, **Rename**, **Delete** and **Copy Path**. See [Collections](/docs/features/collections).

## Next

- [History](/docs/features/history): reopen past requests together with their responses
- [Authentication](/docs/features/auth): Basic, Bearer and custom `Authorization` headers
- [Import & Export](/docs/features/import-export): Postman, Insomnia and cURL
- [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts)
