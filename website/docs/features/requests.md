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
- **cURL.** Paste a command that starts with `curl ` and it becomes the request: method, URL, headers and body. If a saved request is open, the cURL opens as a new unsaved request and the file is left alone. See [Import from cURL](/docs/features/import-export#import-from-curl).
- **Send / Stop.** `⌘ Enter` or the button sends the request. While it's running, the button becomes Stop, and `Esc` cancels too.

If the URL, headers or body use a `{{variable}}` that the selected environment doesn't define, the URL bar gets an amber border. Hover over it to see which variables are missing.

## Tabs

The editor has four tabs: **Body**, **Params**, **Headers** and **Auth**.

The body type sits at the right of the tab row and writes the `Content-Type` header for you:

| Type | Header | Editor |
|------|--------|--------|
| **JSON** | `application/json` | Highlighted, with a format button |
| **Form** | `application/x-www-form-urlencoded` | A table; values are percent-encoded into `a=1&b=2` |
| **Multipart** | `multipart/form-data` | A table of fields. A value starting with `@` is a file path, sent as an upload (`@@` for text that starts with `@`); **Attach file…** picks one. Variables are filled in field by field, so a variable's value is always text, never a file or an extra field, and file paths are used exactly as written (no variables). The boundary is added when the request is sent. |
| **Text** | `text/plain` | Plain |
| **none** | none | Plain |

If you type a body and leave the type as **none**, Mercury says so under the editor — most APIs reject a body without a `Content-Type`, and the error they send back rarely says that.

The header stays the source of truth, so editing `Content-Type` by hand in the Headers tab changes the body type too. A content type Mercury doesn't recognise (`application/xml`, say) shows the plain editor and is left alone unless you pick a different type. The Params and Headers tabs show how many entries are enabled as a small count next to the name. A request opens on the tab that holds its content: **Body** for anything with a body (and POST, PUT, PATCH), **Params** otherwise. At narrow window sizes the body type moves to its own line under the tabs.

### Body

A plain text editor with JSON highlighting. The format button at the top right pretty-prints the body if it's valid JSON. Mercury doesn't set `Content-Type` for you, so add it on the Headers tab.

### Params

This is a table of the URL's query parameters, and it stays in sync with the URL:

- Editing the URL updates the table.
- Editing the table rewrites the URL's query string. Keys and values are percent-encoded, and `{{variables}}` are left as they are.
- Unchecking a row removes that parameter from the URL. Because the URL holds the params, a disabled row is gone the next time the URL is re-read (when you edit the URL or reopen the request).

**Bulk edit** switches to raw text with one `key=value` per line. A leading `#` disables a line.

### Headers

Headers use the same table and **Bulk edit** toggle. In text form each line is `Key: Value`:

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

`{{NAME}}` is replaced with a value from the selected environment in the URL, headers and body when you send the request or copy it as cURL. Unknown variables are named in a line under the URL bar ("`{{NAME}}` isn't defined in .env.dev") and, if you send anyway, go out as the literal text `{{NAME}}`. See [Environments](/docs/features/environments).

## Saving

- `⌘ S`, or **Save…** next to the breadcrumb, saves the request. If it isn't saved yet, Mercury asks for a name and creates `<name>.json` in the folder of the request you opened last (or the folder you clicked last, whichever came later), else in the workspace root. If no workspace is open, it asks you to open a folder first.
- Right-click a folder and choose **New Request** to save the current editor contents into that folder.
- Once a request has a file, Mercury saves it automatically every 5 seconds while there are unsaved changes, when you switch to another request, and when you quit. A dot after the name in the breadcrumb, and **Unsaved changes** in the status bar, mean the latest edits aren't on disk yet; the status bar says **Saved** once they are.
- **Revert to saved** (in the open request's `⋯` in the sidebar, or `⌘ K`) puts the file back the way it was when you opened it, undoing edits that autosave has already written.
- `⌘ N` starts a new, empty, unsaved request.

If a response arrives after you have opened a different request, it is recorded in [history](/docs/features/history) rather than shown under the request you are now looking at.

If a request file changes on disk (for example in another editor) and you have no unsaved edits, Mercury reloads it. If you have unsaved edits, yours are kept (with a warning) and saved over the file. If the file is deleted, Mercury clears the editor.

## The response panel

The top row shows the status (colored by class), the response time and the size. The time is green under 200 ms, amber up to 1 s, and red above that.

Below that are three tabs:

- **Body** is the response body (see the table below).
- **Headers** lists every response header.
- **Cookies** appears with a count when the response sets cookies, and lists them as `name=value`.

On the Body tab, the **Pretty / Raw** switch chooses between the formatted body and the exact bytes (`⌘ R`), and the copy icon copies the whole body. The search icon (or `⌘ F`) opens a find bar over the body: every match is highlighted, the bar counts them, `⏎` steps to the next and `Esc` closes it. You can also select part of the body and copy that. For images and other binary content, the panel offers a **Save response…** button instead. Text bodies of any size up to the 10 MB download limit are shown; long lines of raw text wrap to the panel. In the top row, the download icon saves the body to a file and the clock icon opens the [history list](/docs/features/history). The clock is there before anything has been sent, too.

**Copy as cURL** is `⌘ Shift C`, `⌘ K`, or the open request's `⋯` in the sidebar. **Format** is in the Body tab.

How the body is displayed depends on its type:

| Response | Display |
|----------|---------|
| JSON | Pretty-printed and highlighted |
| XML (including SVG) | Indented and highlighted |
| HTML | Highlighted, wrapped to the panel |
| Other text | Plain text, wrapped to the panel |
| Empty or `204` | "The server returned an empty body" |
| Image, PDF, audio, video, archive, octet-stream | Content type and size, with **Save** (reopened from history, there are no bytes to save: send again) |
| Over 10 MB | **Response too large** (not downloaded; without a `Content-Length`, reading stops at the limit) |

If the server doesn't send a `Content-Type`, Mercury guesses the type from the body. Text bodies have a copy button. See [Performance](/docs/performance) for why the size limits exist.

If a request fails, the panel says what happened and what to try (for example "Nothing is listening on localhost:3000. Is the server running?"), followed by the full underlying error.

## Defaults

These values are fixed in the app and can't be changed:

| Setting | Value |
|---------|-------|
| Timeout | None: a slow endpoint can take as long as it needs; `Esc` cancels. Connecting gives up after 30 seconds. |
| Redirects | Followed (up to 10) |
| Cookies | Kept for the session (see below) |
| Certificates | Checked against your system's trusted roots (so company and `mkcert` CAs work). A self-signed certificate fails with **Trust host and retry**, which skips the check for that host until Mercury quits. |

## Cookies

Mercury uses one HTTP client for the whole session, and that client has a cookie store:

- When a response sets a cookie, Mercury stores it and sends it on later requests to the same site. A login followed by an authenticated request works without copying cookies by hand.
- Cookies live only in memory. Quitting Mercury clears them, and there's no other way to clear them.
- To send a particular cookie yourself, add a `Cookie` header.

## Sidebar actions

Hover a request in the sidebar and click `⋯` (it stays visible on the open request), or right-click it:

| Action | What it does |
|--------|--------------|
| **Copy as cURL** | The open request, as a `curl` command (`⌘ Shift C`) |
| **Duplicate** | Copies `name.json` to `name_copy1.json` (or the next free number) |
| **Revert to saved** | The open request only: undoes edits autosave has written since you opened it |
| **Rename** | Renames the file; `.json` is added if you leave it off. |
| **Delete** | Deletes the file permanently, after you confirm |
| **Copy Path** | Copies the file's full path |

## Related

- [Collections](/docs/features/collections)
- [Environments](/docs/features/environments)
- [Authentication](/docs/features/auth)
- [File Format](/docs/reference/file-format)
