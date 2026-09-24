---
title: Working with Requests
sidebar_label: Requests
sidebar_position: 1
---

# Working with Requests

Each saved request is a `.json` file in your workspace. The [File Format](/docs/reference/file-format) page describes the file itself. This page covers the editor and the response panel.

## The URL bar

- **Method.** Click the method name to choose one of `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`, `CONNECT` or `TRACE`.
- **URL.** Type the full URL, including `http://` or `https://`. `⌘ L` moves focus to it.
- **cURL.** Paste a command that starts with `curl ` and it replaces the method, URL, headers and body. See [Import from cURL](/docs/features/import-export#import-from-curl).
- **Send / Stop.** `⌘ Enter` or the button sends the request. While it's running, the button becomes Stop, and `Esc` cancels too.

If the URL, headers or body use a `{{variable}}` that the selected environment doesn't define, the URL bar gets an amber border. Hover over it to see which variables are missing.

## Tabs

The editor has four tabs: **Body**, **Params**, **Headers** and **Auth**. The Params and Headers tabs show how many entries are enabled, for example `Headers (2)`. The Auth tab is labeled with the current auth type.

### Body

A plain text editor with JSON highlighting. The format button at the top right pretty-prints the body if it's valid JSON. Mercury doesn't set `Content-Type` for you, so add it on the Headers tab.

### Params

This is a table of the URL's query parameters, and it stays in sync with the URL:

- Editing the URL updates the table.
- Editing the table rewrites the URL's query string. Keys and values are percent-encoded, and `{{variables}}` are left as they are.
- Unchecking a row removes that parameter from the URL. Because the URL holds the params, a disabled row is gone the next time the URL is re-read (when you edit the URL or reopen the request).

**Bulk Edit** switches to raw text with one `key=value` per line. A leading `#` disables a line.

### Headers

Headers use the same table and **Bulk Edit** toggle. In text form each line is `Key: Value`:

```
Content-Type: application/json
Accept: application/json
# X-Debug: 1
```

- A line starting with `#` is disabled. It isn't sent and isn't written to the request file, so it survives only until you reopen the file.
- Lines without a `:` are ignored.
- Header names must be unique. If you repeat a name, only one of the values is kept.

Below the Params and Headers tabs, chips list each `{{variable}}` in use, marked ✓ (defined in the selected environment) or ✗ (not defined).

### Auth

The Auth tab edits the `Authorization` header. See [Authentication](/docs/features/auth).

## Variables

`{{NAME}}` is replaced with a value from the selected environment in the URL, headers and body when you send the request or copy it as cURL. Unknown variables are sent as the literal text `{{NAME}}`. See [Environments](/docs/features/environments).

## Saving

- `⌘ S` saves the open request. If the request isn't saved yet, Mercury asks for a name and creates `<name>.json` in the workspace root. If no workspace is open, it asks you to open a folder first.
- Right-click a folder and choose **New Request** to save the current editor contents into that folder.
- Once a request has a file, Mercury saves it automatically every 5 seconds while there are unsaved changes, when you switch to another request, and when you quit. A dot after the name in the breadcrumb means there are unsaved changes.
- `⌘ N` starts a new, empty, unsaved request.

If a request file changes on disk (for example in another editor) and you have no unsaved edits, Mercury reloads it. If you have unsaved edits, yours are kept (with a warning) and saved over the file. If the file is deleted, Mercury clears the editor.

## The response panel

The top row shows the status (colored by class), the response time and the size. The time is green under 200 ms, amber up to 1 s, and red above that.

Below that:

- **Headers (N)** shows the response headers.
- **Cookies (N)** appears when the response sets cookies and lists them as `name=value`.
- **Raw** shows text bodies exactly as received, without formatting. `⌘ R` toggles it.
- **Save** appears for images, binary bodies and large text, and writes the body to a file.
- **History** opens the [history list](/docs/features/history).

How the body is displayed depends on its type:

| Response | Display |
|----------|---------|
| JSON | Pretty-printed and highlighted |
| XML (including SVG) | Indented and highlighted |
| HTML | Highlighted |
| Other text | Plain text |
| Empty or `204` | "The server returned an empty response" |
| Image, PDF, audio, video, archive, octet-stream | Content type and size, with **Save** |
| Text over 100 KB | Content type and size, with **Save** |
| `Content-Length` over 10 MB | **Response Too Large** (not downloaded) |

If the server doesn't send a `Content-Type`, Mercury guesses the type from the body. Text bodies have a copy button. See [Performance](/docs/performance) for why the size limits exist.

If a request fails, the panel shows the reason: timed out, SSL/TLS error, connection failed, or invalid URL.

## Defaults

These values are fixed in the app and can't be changed:

| Setting | Value |
|---------|-------|
| Timeout | 30 seconds |
| Redirects | Followed (up to 10) |
| Cookies | Kept for the session (see below) |

## Cookies

Mercury uses one HTTP client for the whole session, and that client has a cookie store:

- When a response sets a cookie, Mercury stores it and sends it on later requests to the same site. A login followed by an authenticated request works without copying cookies by hand.
- Cookies live only in memory. Quitting Mercury clears them, and there's no other way to clear them.
- To send a particular cookie yourself, add a `Cookie` header.

## Sidebar actions

Right-click a request in the sidebar:

| Action | What it does |
|--------|--------------|
| **Duplicate** | Copies `name.json` to `name_copy1.json` (or the next free number) |
| **Rename** | Renames the file; `.json` is added if you leave it off. |
| **Delete** | Deletes the file permanently, after you confirm |
| **Copy Path** | Copies the file's full path |

## Related

- [Collections](/docs/features/collections)
- [Environments](/docs/features/environments)
- [Authentication](/docs/features/auth)
- [File Format](/docs/reference/file-format)
