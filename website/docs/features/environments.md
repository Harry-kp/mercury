---
title: Environment Variables
sidebar_label: Environments
sidebar_position: 3
---

# Environment Variables

Environments are `.env` files in the workspace root. Use them for base URLs, tokens and IDs that change between setups, and to keep secrets out of request files.

## Environment files

Every file in the **workspace root** whose name starts with `.env` is an environment: `.env`, `.env.dev`, `.env.production`, `.env.local` and so on. `.env` files in subfolders are ignored.

```bash
# .env.dev
BASE_URL=http://localhost:3000
API_TOKEN=dev-token

# .env.production
BASE_URL=https://api.example.com
API_TOKEN="prod token"
```

## Choosing an environment

**Only one environment is active at a time.** Files aren't layered or merged. A plain `.env` is not loaded on top of the selected one.

- Pick the environment in the menu at the **top right** of the window. **None** turns variables off.
- `⌘ E` switches to the next environment. After the last one it goes to None, then starts over.
- When you open a workspace, Mercury selects the first file (alphabetically) whose name contains `.dev`, or else the first file. When Mercury reopens the workspace at launch, it restores the environment you last had selected.
- Environment names that contain `prod` are shown in red, and names that contain `stag` in amber, so it's harder to send to production by mistake.

## File format

```bash
# Full-line comments start with #
BASE_URL=https://api.example.com
USER_ID = 12345
QUOTED="two words"
SINGLE='also fine'
EQUALS=a=b
```

- One `KEY=VALUE` per line. The line is split at the first `=`.
- Spaces around the key and value are trimmed.
- A value wrapped in matching `"` or `'` has the quotes removed. No other escaping is done.
- Blank lines and lines starting with `#` are ignored. Lines without `=` are ignored.
- Keys are case-sensitive.
- Values are used as they are. A value that contains `{{OTHER}}` is not expanded.

## Using variables

Write `{{NAME}}` in the URL, a header (including the Auth tab) or the body:

```
GET {{BASE_URL}}/users/{{USER_ID}}
Authorization: Bearer {{API_TOKEN}}
```

- Variables are substituted when you send a request or copy it as cURL. Request files always keep the `{{NAME}}` placeholders.
- Spaces inside the braces are allowed: `{{ NAME }}` is the same as `{{NAME}}`.
- A variable the active environment doesn't define is **left as-is** and sent as the literal text `{{NAME}}`.

## Seeing what's missing

- If any variable in the request is undefined, the URL bar gets an **amber border**. Hover over it to see the list.
- The **Params** and **Headers** tabs show a chip for each variable they use: ✓ if it's defined, ✗ if it isn't.

## Editing environment files

Edit `.env` files in any editor. When you add or remove an env file, the picker updates right away. After changing values in an existing file, select the environment again in the picker to reload them.

## Keeping secrets out of Git

```gitignore
.env*
!.env.example
```

Commit a `.env.example` with placeholder values so that others can copy it:

```bash
cp .env.example .env.dev
```

## Troubleshooting

If a variable isn't substituted:

1. Check that the right environment is selected at the top right.
2. Check that the file is in the workspace **root** and its name starts with `.env`.
3. Check the spelling. Names are case-sensitive.
4. If you just edited the file, select the environment again to reload it.

## Related

- [Requests](/docs/features/requests)
- [Authentication](/docs/features/auth)
- [Import & Export](/docs/features/import-export): Postman and Insomnia variables become `.env.<name>` files
