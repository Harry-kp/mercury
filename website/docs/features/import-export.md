---
title: Import & Export
sidebar_label: Import & Export
sidebar_position: 6
---

# Import & Export

Mercury imports Postman and Insomnia exports and cURL commands, and it can copy any request as a cURL command.

## Where imports go

Choose **Open → Import Postman...** or **Open → Import Insomnia...**. The empty sidebar also has **Import from ...** links. Pick the export file, and then:

- if a workspace is open, the requests are written **into that workspace**
- if no workspace is open, Mercury asks for a folder to write to, then opens it as the workspace

The status bar reports how many requests and environments were imported.

File and folder names are sanitized: they're lowercased, and spaces and characters such as `/ \ : * ? " < > |` become `-`. For example, "Get User" becomes `get-user.json`. Existing files with the same name are **overwritten**.

## Postman

Export the collection from Postman as **Collection v2.1** (JSON).

| Postman | Mercury |
|---------|---------|
| Request | `<name>.json` |
| Folder (any depth) | Folder, with the same nesting |
| Collection variables | `.env.<collection-name>` in the import folder |
| Headers | Headers (disabled ones are skipped) |
| Raw body | Body |
| URL | The raw URL. Without one, the URL is rebuilt from its parts, skipping disabled query params. |

Postman `{{variables}}` stay as they are in URLs, headers and bodies, so they work with the generated `.env` file. Auth settings, form-data and other non-raw bodies, scripts and tests aren't imported. Add auth on the [Auth tab](/docs/features/auth) afterwards.

Example: importing a collection named "My API":

```
workspace/
├── .env.my-api
├── auth/
│   └── login.json
├── users/
│   └── admin/
│       └── list-admins.json
└── health-check.json
```

## Insomnia

Export from Insomnia as JSON or YAML.

| Insomnia | Mercury |
|----------|---------|
| Request inside a request group | `<group-name>/<name>.json` |
| Request not in a group | `imported/<name>.json` |
| Environment (non-empty) | `.env.<environment-name>` |
| Headers | Headers (disabled ones are skipped) |
| Body text | Body |

Each request goes into a folder named after its **direct parent** group. Nested groups aren't recreated as nested folders. Auth settings aren't imported.

```
workspace/
├── .env.base-environment
├── users/
│   ├── get-user.json
│   └── list-users.json
└── imported/
    └── health.json
```

## Import from cURL

Paste a command that starts with `curl ` into the **URL bar**. Mercury parses it and replaces the method, URL, headers and body. This works with commands copied from browser DevTools or API docs, including multi-line commands that use `\` continuations.

```bash
curl -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John"}'
```

| Flag | Effect |
|------|--------|
| `-X`, `--request` | Method |
| `-H`, `--header` | Header |
| `-d`, `--data`, `--data-raw`, `--data-binary` | Body. Changes a `GET` to `POST`. |
| `--json` | Body plus `Content-Type: application/json`. Changes a `GET` to `POST`. |
| `-u`, `--user` | `Authorization: Basic ...` |
| `-A`, `--user-agent` | `User-Agent` header |
| `-b`, `--cookie` | `Cookie` header |
| `-I`, `--head` | `HEAD` |
| `-G`, `--get` | `GET` |
| `-o`, `-x`, `-c`, `-m`, `-w`, `-e`, `--output`, `--proxy`, `--cookie-jar`, `--connect-timeout`, `--max-time`, `--write-out`, `--cacert`, `--cert`, `--key`, `--referer` | Ignored, along with their argument |
| Other flags (`-s`, `-L`, `-k`, `-v`, `--compressed`, ...) | Ignored |

The first argument that isn't a flag is used as the URL. If there isn't one, you get an error.

## Copy as cURL

Press `⌘ Shift C` to copy the current request to the clipboard as a cURL command:

```bash
curl -X GET 'https://api.example.com/users' \
  -H 'Accept: application/json' \
  -H 'Authorization: Bearer abc123'
```

- Variables from the selected environment are substituted in. Undefined ones stay as `{{NAME}}`.
- Disabled (`#`) headers are left out.
- Arguments are single-quoted, so you can paste the command into a POSIX shell as is.

## Sharing request files

Requests are plain `.json` files, so you can also share them by copying them, committing them to Git, or sending them to someone. See [File Format](/docs/reference/file-format).

## Related

- [Collections](/docs/features/collections)
- [Environments](/docs/features/environments)
- [File Format](/docs/reference/file-format)
