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
| **Top bar** | The workspace menu, the breadcrumb (`workspace / folder / METHOD name`), the environment picker, the theme toggle and the `⋯` menu |
| **Sidebar** (left) | A filter box, **Recent** unsaved requests and the workspace folder tree |
| **Editor** (center) | Method, URL bar, and the **Body**, **Params**, **Headers** and **Auth** tabs |
| **Response** (right) | Status, time and size, then the **Body**, **Headers** and **Cookies** tabs — or the history list |
| **Status bar** | The workspace path, and links to the command palette (`⌘ K`) and the shortcut list |

## 1. Send a request

You don't need a workspace to send a request.

1. Click the URL bar (or press `⌘ L`) and type `httpbin.org/get`. A URL without a scheme gets one: `http` for `localhost`, `https` for anything else.
2. Press `⌘ Enter`, or click **Send**.

The response panel shows the status, response time and size, followed by the body. JSON is pretty-printed and highlighted. While a request is running, press `Esc` or click the stop button to cancel it.

Requests you send without saving appear under **Recent** in the sidebar. Click one to load it again.

:::tip
You can paste a cURL command into the URL bar. Mercury fills in the method, URL, headers and body from it. See [Import & Export](/docs/features/import-export#import-from-curl).
:::

On Windows and Linux, `⌘` means `Ctrl`.

## 2. Open a workspace

A workspace is any folder. Mercury shows its subfolders and `.json` request files in the sidebar.

1. Press `⌘ O`, or choose **Open folder…** from the workspace menu in the top bar.
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

## 6. Find anything with `⌘ K`

`⌘ K` opens the command palette. Type part of a request name (or its folder) to open it without hunting through the tree, or type what you want to do — "curl", "format", "import" — to run it. `↑`/`↓` move, `⏎` opens, `Esc` closes.

## Picking up where you left off

Mercury reopens the workspace, the environment and the request file you had open. Unsaved edits are written to that file before it quits, so a relaunch puts you back where you were.

## Light and dark

Mercury follows your system appearance. `⌘ D`, or the sun/moon button in the top bar, pins it to light or dark instead; the choice is remembered between launches.

## Next

- [History](/docs/features/history): reopen past requests together with their responses
- [Authentication](/docs/features/auth): Basic, Bearer and custom `Authorization` headers
- [Import & Export](/docs/features/import-export): Postman, Insomnia and cURL
- [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts)
