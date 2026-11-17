---
title: Import & Export
sidebar_label: Import & Export
sidebar_position: 6
---

# Import & Export

Mercury imports Postman and Insomnia exports and cURL commands, and it can copy any request as a cURL command.

## Where imports go

Choose **Import from Postman…** or **Import from Insomnia…** from the workspace menu in the top bar. With no folder open, the sidebar also lists **Postman collection** and **Insomnia export** under **Import**. Pick the export file (or several at once: a Postman collection and its environment exports), and then:

- if a workspace is open, the requests are written into it, in a **new folder named after the collection** (`acme-api/`), so they can't mix with or replace your own
- if no workspace is open, Mercury asks for a folder to write to and opens it as the workspace; an empty folder gets the requests directly

Environments go in the workspace root as `.env.<name>`, where Mercury looks for them. When the import is done, its first request is open and its environment selected, and a notification says what was imported and where.

File and folder names are sanitized: they're lowercased, and spaces and characters such as `/ \ : * ? " < > |` become `-`. For example, "Get User" becomes `get-user.json`. **Nothing is ever overwritten**: a name that's taken gets a suffix (`.env.acme-api-2`, `list-users-2.json`).

## Postman

Export the collection from Postman as **Collection v2.1** (JSON). Your environments are separate exports in Postman (**Environments → ⋯ → Export**): select them together with the collection, or import each one later, and each becomes `.env.<name>`. Disabled variables are left out.

| Postman | Mercury |
|---------|---------|
| Request | `<name>.json` |
| Folder (any depth) | Folder, with the same nesting |
| Collection variables | `.env.<collection-name>` in the import folder |
| Headers | Headers (disabled ones are skipped) |
| Raw body | Body |
| URL-encoded body | A [Form body](/docs/features/requests#body), with the matching `Content-Type` |
| Form-data body | A [Multipart body](/docs/features/requests#body). File fields arrive switched off: tick one once you've checked which file it uploads |
| Bearer / Basic / API key auth | The `Authorization` (or API key) header, or a query parameter |
| URL | The raw URL. Without one, the URL is rebuilt from its parts, skipping disabled query params. |

Auth set on the collection or a folder applies to every request under it, and a request's own auth wins over it.

Postman `{{variables}}` stay as they are in URLs, headers and bodies, so they work with the generated `.env` file. Path variables are filled in (`/users/:userId` with value 42 becomes `/users/42`), or become `{{userId}}` when the collection has no value for them. A raw body's language (JSON, XML, text) sets the `Content-Type` Postman would have sent. Postman's built-in `{{$guid}}`, `{{$timestamp}}`, `{{$isoTimestamp}}` and `{{$randomInt}}` work as they are. Scripts and tests aren't imported.

What Mercury can't express is **named in the notification** rather than dropped quietly: OAuth 2 and other token-exchange auth, binary-file bodies, and GraphQL bodies. Anything listed there needs finishing by hand on the [Auth tab](/docs/features/auth) or the Body tab.

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

In Insomnia, use **Export** on a collection (v5 YAML, Insomnia 10 and later) or **Export Data** (v4 JSON or YAML, older versions). Both work.

| Insomnia | Mercury |
|----------|---------|
| Folder (any depth) | Folder, with the same nesting |
| Request outside any folder | `<name>.json` at the top |
| Sub-environment (Dev, Production…) | `.env.<name>`, **including the base environment's values** (Insomnia merges them the same way) |
| Base environment with no sub-environments | `.env.<collection-name>` |
| `{{ _.base_url }}` | `{{base_url}}`; nested values (`_.api.url`) become `api.url` |
| Bearer, API key and Basic auth | The `Authorization` (or API key) header, or a query parameter |
| JSON / text body | Body, with its `Content-Type` |
| Form URL-encoded / Multipart body | A Form / Multipart body. File fields arrive switched off |
| Headers | Headers (disabled ones are skipped) |

Named in the notification, because Mercury can't express them: OAuth 2 and other token-exchange auth, Basic auth whose username or password is a variable (the header is base64, which would hide it), GraphQL bodies, and template tags such as `{% response %}` or `{% uuid %}`.

```
workspace/
├── .env.dev
├── .env.production
├── users/
│   ├── admin/
│   │   └── ban-user.json
│   ├── get-user.json
│   └── list-users.json
└── health.json
```

## Import from cURL

Paste a command that starts with `curl ` into the **URL bar**. Mercury parses it into the method, URL, headers and body. Pasted over a saved request, it opens as a new unsaved request instead, so the file is never overwritten. Typing `curl …` by hand is left alone; only a paste converts. This works with commands copied from browser DevTools or API docs, including multi-line commands that use `\` continuations.

```bash
curl -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John"}'
```

| Flag | Effect |
|------|--------|
| `-X`, `--request` | Method |
| `-H`, `--header` | Header |
| `-d`, `--data`, `--data-raw`, `--data-binary` | Body. Changes a `GET` to `POST`. Repeated flags are joined with `&`, as curl does. Without a `Content-Type` header, adds `application/x-www-form-urlencoded`, which is what curl sends. |
| `--data-urlencode` | Body, percent-encoded: in `name=value` only the value is encoded. |
| `--json` | Body plus `Content-Type: application/json`. Changes a `GET` to `POST`. |
| `-u`, `--user` | `Authorization: Basic ...` |
| `-A`, `--user-agent` | `User-Agent` header |
| `-b`, `--cookie` | `Cookie` header |
| `-I`, `--head` | `HEAD` |
| `-G`, `--get` | `GET`, with any `-d` data appended to the URL as the query string |
| `-F`, `--form`, `--form-string` | A [Multipart body](/docs/features/requests#body), one field per `-F` (`name=@path` is a file; curl's `;type=` is dropped). Changes a `GET` to `POST`. Copy as cURL writes Multipart bodies back as `-F`. |
| `-o`, `-x`, `-c`, `-m`, `-w`, `-e`, `--output`, `--proxy`, `--cookie-jar`, `--connect-timeout`, `--max-time`, `--write-out`, `--cacert`, `--cert`, `--key`, `--referer` | Ignored, along with their argument |
| Other flags (`-s`, `-L`, `-k`, `-v`, `--compressed`, ...) | Ignored |

The first argument that isn't a flag is used as the URL. If there isn't one, you get an error. A value glued to a short flag (`-XPOST`, `-HAccept:text/plain`), short flags sharing one dash (`-sX POST`, `-sI`) and bash's `$'…'` quoting, which Chrome uses when a body contains a quote or a newline, all work.

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
