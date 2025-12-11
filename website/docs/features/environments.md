---
title: Environment Variables
sidebar_label: Environments
sidebar_position: 3
---

# Environment Variables

> Use variables to manage different environments (development, staging, production) and keep secrets out of your request files.

## What are Environment Variables?

Environment variables let you:
- Store base URLs that change per environment
- Keep API keys and secrets separate from requests
- Easily switch between development and production
- Share requests without exposing credentials

## Creating Environment Files

Create `.env` files in your workspace root:

```bash
# .env (default, always loaded)
BASE_URL=https://api.example.com
TIMEOUT=5000

# .env.development
BASE_URL=http://localhost:3000
API_KEY=dev-key-123

# .env.production
BASE_URL=https://api.production.com
API_KEY=prod-key-secret
```

Mercury loads `.env` by default and lets you switch between other environment files.

## File Format

Environment files use simple `KEY=VALUE` format:

```bash
# This is a comment
BASE_URL=https://api.example.com

# Quotes are optional but stripped
API_KEY="my-secret-key"
ANOTHER_KEY='single-quotes-work-too'

# No spaces around equals sign
USER_ID=12345
```

## Using Variables in Requests

Reference variables with double curly braces `{{variable}}`:

```http
GET {{BASE_URL}}/users/{{USER_ID}}
Authorization: Bearer {{API_TOKEN}}
Content-Type: application/json
```

Variables work everywhere:
- **URL** — `{{BASE_URL}}/endpoint`
- **Headers** — `Authorization: Bearer {{TOKEN}}`
- **Body** — `{"user": "{{USERNAME}}"}`

![Variable substitution - Replace with: Screenshot showing request with variables and their resolved values](/img/screenshots/placeholder.png)

## Variable Indicators

Mercury shows the status of each variable:

| Indicator | Meaning |
|-----------|---------|
| 🟢 Green | Variable is defined in current environment |
| 🔴 Red | Variable is undefined (will be sent as literal `{{name}}`) |

Hover over a variable to see its current value.

![Variable indicators - Replace with: Screenshot showing green/red variable indicators in request editor](/img/screenshots/placeholder.png)

## Switching Environments

Click the environment selector in the status bar to switch between:
- `.env` (default)
- `.env.development`
- `.env.production`
- `.env.staging`
- (any `.env.*` file in your workspace)

:::tip Quick Switch
Use the keyboard shortcut shown in the environment selector for faster switching.
:::

## Environment Hierarchy

Mercury loads environment files in this order:

1. `.env` — Always loaded first (base values)
2. `.env.{selected}` — Overrides values from base

This means you can have defaults in `.env` and only override what changes per environment.

### Example

```bash
# .env (base)
BASE_URL=https://api.example.com
TIMEOUT=5000
DEBUG=false

# .env.development (overrides)
BASE_URL=http://localhost:3000
DEBUG=true
```

With **development** selected:
- `BASE_URL` = `http://localhost:3000` (overridden)
- `TIMEOUT` = `5000` (from base)
- `DEBUG` = `true` (overridden)

## Secrets Management

:::warning Never Commit Secrets
Add `.env*` to your `.gitignore` to prevent committing secrets:

```gitignore
# .gitignore
.env
.env.*
!.env.example
```
:::

Create a template for your team:

```bash
# .env.example (safe to commit)
BASE_URL=https://api.example.com
API_KEY=your-api-key-here
```

Team members copy and fill in their own values:
```bash
cp .env.example .env
```

## Common Patterns

### Per-Environment Base URLs

```bash
# .env.development
BASE_URL=http://localhost:3000

# .env.staging
BASE_URL=https://staging.api.example.com

# .env.production
BASE_URL=https://api.example.com
```

### Auth Tokens

```bash
# .env
AUTH_TOKEN=your-token-here

# Use in request
Authorization: Bearer {{AUTH_TOKEN}}
```

### Dynamic IDs

```bash
# .env
USER_ID=12345
ORDER_ID=67890

# Use in request
GET {{BASE_URL}}/users/{{USER_ID}}/orders/{{ORDER_ID}}
```

## Troubleshooting

### Variable Not Substituted?

1. Check for typos in the variable name
2. Ensure the `.env` file is in workspace root
3. Verify the correct environment is selected
4. Look for red indicators — they mean undefined

### Changes Not Reflecting?

Mercury watches `.env` files automatically. If changes don't appear:
1. Save the `.env` file
2. Check for syntax errors (no spaces around `=`)

## Related Features

- [Requests](/docs/features/requests) — Using variables in `.http` files
- [File Format](/docs/reference/file-format) — Complete variable syntax reference
