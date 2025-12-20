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
| <span class="method-badge method-get">GET</span> | Retrieve data | Blue |
| <span class="method-badge method-post">POST</span> | Create resource | Green |
| <span class="method-badge method-put">PUT</span> | Replace resource | Orange |
| <span class="method-badge method-patch">PATCH</span> | Partial update | Purple |
| <span class="method-badge method-delete">DELETE</span> | Remove resource | Red |
| HEAD | Get headers only | Cyan |
| OPTIONS | Get allowed methods | Brown |
| CONNECT | Tunnel connection | Teal |
| TRACE | Debug request path | Gray |

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

## Query Parameters

Mercury provides a dedicated **Params** tab for managing URL query parameters:

```http
GET https://api.example.com/search?q=mercury&page=1&limit=10
```

Instead of editing the URL directly, use the **Params** tab to:

- **Add key-value pairs** — Click in the empty row and type
- **Toggle parameters on/off** — Checkbox enables/disables without deleting
- **See all parameters at a glance** — Clean table view
- **Edit values easily** — No more navigating the URL string

### Two-Way Sync

The Params table stays in sync with the URL bar:
- Edit the URL → Params table updates automatically
- Edit in Params tab → URL updates automatically
- Paste a cURL command → Params are parsed and shown

### Variables in Parameters

Use `{{variable}}` syntax in parameter values:

| Key | Value |
|-----|-------|
| `api_key` | `{{API_KEY}}` |
| `page` | `1` |

Variable indicators show which variables are defined in your environment.

:::tip Toggle vs Delete
Use the checkbox to temporarily disable a parameter without losing it. This is useful for testing different API configurations.
:::

## Sending Requests

**Keyboard**: Press `⌘+Enter` (Mac) or `Ctrl+Enter` (Windows/Linux)

**Mouse**: Click the **Send** button in the URL bar

The button animates while the request is in progress.

### Cancelling a Request

Click the **Stop** button (which replaces **Send**) or press `Esc` to cancel a running request.
The request is immediately stopped, and the UI is unblocked.

![Sending request - Replace with: Screenshot showing animated send/stop button](/img/screenshots/placeholder.png)

## Defaults

Mercury uses sensible defaults so you can focus on your API, not configuration:

| Setting | Default | Behavior |
|---------|---------|----------|
| **Timeout** | 30 seconds | Requests fail after 30s of no response |
| **Redirects** | Followed | HTTP redirects followed automatically (up to 10) |

:::tip URL Validation
Mercury validates URLs before sending:
- Empty URLs show an error
- Unresolved `{{variables}}` show a warning
:::

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
