---
title: JSON File Format
sidebar_label: File Format
sidebar_position: 1
---

# Request File Format

Each request is one `.json` file in the workspace that holds a single JSON object.

```json
{
  "method": "POST",
  "url": "{{BASE_URL}}/users",
  "headers": {
    "Authorization": "Bearer {{API_TOKEN}}",
    "Content-Type": "application/json"
  },
  "body": "{\"name\": \"Jane\"}"
}
```

## Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `method` | string | Yes | One of `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`, `CONNECT`, `TRACE`. Must be uppercase. |
| `url` | string | Yes | The full URL, including `http://` or `https://`. May contain `{{variables}}`. |
| `headers` | object | No | Header name → value, both strings. Missing means no headers. |
| `body` | string | No | The raw request body. Missing means no body. |

The smallest valid file:

```json
{ "method": "GET", "url": "https://httpbin.org/get" }
```

If a file can't be parsed as a request, it still appears in the sidebar, without a method, and opening it shows an error.

## How Mercury writes files

- The JSON is pretty-printed with 2-space indentation.
- **Headers are sorted by name**, so saving the same request always produces the same file and Git diffs stay small.
- **Empty `headers` and `body` are omitted.**
- **Disabled headers (lines starting with `#` in the editor) aren't saved.** The file only has the headers that are sent.
- Header names are unique keys. A second header with the same name replaces the first.
- The query string is part of `url`. There's no separate params field.
- Auth is the `Authorization` entry in `headers`. There's no separate auth field.

## Body

The body is always a string, whatever its content type. A JSON body is stored as an escaped JSON string:

```json
{
  "method": "POST",
  "url": "https://api.example.com/login",
  "headers": { "Content-Type": "application/x-www-form-urlencoded" },
  "body": "username=john&password=secret"
}
```

Mercury doesn't add a `Content-Type` header, so include one yourself.

## Variables

`{{NAME}}` (or `{{ NAME }}`) in `url`, header values or `body` is replaced with a value from the selected environment when you send the request. Variables that aren't defined are sent as the literal text. See [Environments](/docs/features/environments).

## Editing by hand

You can create or edit request files in any editor. Mercury watches the workspace and reloads the open request when its file changes, unless you have unsaved edits in Mercury.

## Other files Mercury uses

These live outside the workspace, in `~/.mercury/` (or `$MERCURY_HOME`):

| File | Contents |
|------|----------|
| `state.json` | The last workspace, the editor contents, the selected tab and environment. Restored on launch. |
| `recent.json` | Unsaved requests you've sent (up to 50) |
| `history.json` | Sent requests and their responses (the newest 50, up to 7 days old) |

## Related

- [Requests](/docs/features/requests)
- [Environments](/docs/features/environments)
