---
title: Working with Requests
sidebar_label: Requests
sidebar_position: 1
---

# Working with Requests

> Requests are the core of Mercury. Learn how to create, edit, and manage your HTTP requests using the simple `.http` file format.

## What is a Request?

In Mercury, every request is a plain text `.http` file. This means:
- Version control friendly (Git works perfectly)
- Editable in any text editor
- Portable across any system
- No proprietary formats

## Creating Requests

### From the UI

1. Right-click in the sidebar → **New Request**
2. Or press `⌘+N` (Mac) / `Ctrl+N` (Windows/Linux)
3. Enter a name (e.g., `get-users`)

Mercury creates a `.http` file in your workspace.

### From Your Editor

Create any file with `.http` extension:

```http
GET https://api.example.com/users
```

Mercury detects it automatically thanks to live file watching.

## The .http File Format

Every `.http` file follows this structure:

```http
METHOD URL
Header-Name: Header-Value
Header-Name: Header-Value

Body content here
```

### Example: Simple GET Request

```http
GET https://api.example.com/users
```

### Example: POST with Headers and Body

```http
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer {{token}}

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

![Request editor - Replace with: Screenshot showing a POST request with headers and JSON body](/img/screenshots/placeholder.png)

## HTTP Methods

Mercury supports all standard HTTP methods:

| Method | Description | Color |
|--------|-------------|-------|
| <span class="method-badge method-get">GET</span> | Retrieve data | Green |
| <span class="method-badge method-post">POST</span> | Create resource | Blue |
| <span class="method-badge method-put">PUT</span> | Replace resource | Orange |
| <span class="method-badge method-patch">PATCH</span> | Partial update | Yellow |
| <span class="method-badge method-delete">DELETE</span> | Remove resource | Red |
| HEAD | Get headers only | Cyan |
| OPTIONS | Get allowed methods | Brown |

Click the method badge in the URL bar to change methods.

## Adding Headers

Headers go on lines between the URL and the body:

```http
GET https://api.example.com/data
Accept: application/json
X-API-Version: 2
Cache-Control: no-cache
```

### Common Headers

| Header | Purpose |
|--------|---------|
| `Authorization` | Auth credentials (Basic, Bearer) |
| `Content-Type` | Body format (`application/json`) |
| `Accept` | Expected response format |
| `User-Agent` | Client identification |

:::tip Auto-Headers
Mercury automatically adds `Content-Type: application/json` when your body starts with `{` or `[`.
:::

## Request Body

Add a blank line after headers, then your body content:

### JSON Body

```http
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "Mercury",
  "type": "API Client",
  "features": ["fast", "minimal", "free"]
}
```

### Form Data (URL-encoded)

```http
POST https://api.example.com/login
Content-Type: application/x-www-form-urlencoded

username=john&password=secret
```

### Plain Text

```http
POST https://api.example.com/webhook
Content-Type: text/plain

This is a plain text message
```

## Using Variables

Reference environment variables with `{{variable}}` syntax:

```http
GET {{BASE_URL}}/users/{{USER_ID}}
Authorization: Bearer {{API_TOKEN}}
```

Mercury shows indicators for each variable:
- **Green** — Variable is defined in your `.env` file
- **Red** — Variable is undefined (will be sent as literal text)

See [Environment Variables](/docs/features/environments) for more details.

## Sending Requests

**Keyboard**: Press `⌘+Enter` (Mac) or `Ctrl+Enter` (Windows/Linux)

**Mouse**: Click the **Send** button in the URL bar

The button animates while the request is in progress.

![Sending request - Replace with: Screenshot showing animated send button during request](/img/screenshots/placeholder.png)

## Request Actions

Right-click on a request in the sidebar for actions:

| Action | Description |
|--------|-------------|
| **Duplicate** | Create a copy of the request |
| **Rename** | Change the filename |
| **Delete** | Move to trash |
| **Copy as cURL** | Copy as cURL command |

## Related Features

- [Collections](/docs/features/collections) — Organize requests in folders
- [Environments](/docs/features/environments) — Use variables for different environments
- [Authentication](/docs/features/auth) — Add Basic Auth or Bearer tokens
- [File Format Reference](/docs/reference/file-format) — Complete `.http` specification
