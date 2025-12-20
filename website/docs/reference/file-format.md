---
title: .http File Format
sidebar_label: File Format
sidebar_position: 1
---

# .http File Format Reference

> Mercury uses the `.http` file format — a simple, human-readable text format for HTTP requests. This page documents the complete specification.

## Overview

A `.http` file represents a single HTTP request. The format is:

```http
METHOD URL
Header-Name: Header-Value
Header-Name: Header-Value

Optional body content
```

## Request Line

The first line contains the HTTP method and URL.

```http
GET https://api.example.com/users
```

### Supported Methods

| Method | Description |
|--------|-------------|
| `GET` | Retrieve resource |
| `POST` | Create resource |
| `PUT` | Replace resource |
| `PATCH` | Partial update |
| `DELETE` | Remove resource |
| `HEAD` | Get headers only |
| `OPTIONS` | Get allowed methods |

Methods are case-insensitive, but UPPERCASE is conventional.

### URL Format

URLs must include the protocol:

```http
# ✅ Correct
GET https://api.example.com/users

# ❌ Invalid (no protocol)
GET api.example.com/users
```


## Headers

Headers go on lines after the request line. Each header is `Name: Value`.

```http
GET https://api.example.com/users
Accept: application/json
Authorization: Bearer token123
X-Custom-Header: my-value
```

### Header Rules

- One header per line
- Headers end at the first blank line
- Names are case-insensitive (`Content-Type` = `content-type`)
- Whitespace around `:` is trimmed

### Common Headers

| Header | Purpose | Example |
|--------|---------|---------|
| `Authorization` | Authentication | `Bearer token123` |
| `Content-Type` | Body format | `application/json` |
| `Accept` | Expected response format | `application/json` |
| `User-Agent` | Client identification | `Mercury/1.0` |
| `Cache-Control` | Caching behavior | `no-cache` |

## Body

The body starts after a blank line following the headers.

```http
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

### JSON Body

```http
POST https://api.example.com/data
Content-Type: application/json

{
  "key": "value",
  "array": [1, 2, 3],
  "nested": {
    "field": true
  }
}
```

### Form Data

```http
POST https://api.example.com/login
Content-Type: application/x-www-form-urlencoded

username=john&password=secret
```

### Plain Text

```http
POST https://api.example.com/webhook
Content-Type: text/plain

This is a plain text message.
It can span multiple lines.
```

### No Body

GET, HEAD, and OPTIONS typically have no body. Just omit the blank line and body:

```http
GET https://api.example.com/users
Accept: application/json
```

## Variables

Use `{{variable}}` syntax for dynamic values.

```http
GET {{BASE_URL}}/users/{{USER_ID}}
Authorization: Bearer {{API_TOKEN}}
```

### Variable Substitution

Variables are replaced with values from `.env` files:

```bash
# .env
BASE_URL=https://api.example.com
USER_ID=12345
API_TOKEN=secret123
```

### Variable Locations

Variables work in:

| Location | Example |
|----------|---------|
| URL | `{{BASE_URL}}/endpoint` |
| Headers | `Authorization: Bearer {{TOKEN}}` |
| Body | `{"user": "{{USERNAME}}"}` |

### Variable Rules

- Variable names are case-sensitive
- Undefined variables are sent as literal text (`{{UNDEFINED}}`)
- No spaces inside braces: `{{VALID}}`, not `{{ INVALID }}`

## Comments

Lines starting with `#` are comments (in `.env` files):

```bash
# .env file
# This is a comment
BASE_URL=https://api.example.com  # This is NOT a comment (included in value)
```

:::warning
Comments are not supported in `.http` files. Every line is parsed as part of the request.
:::

## Complete Examples

### GET with Authentication

```http
GET https://api.example.com/users
Authorization: Bearer {{API_TOKEN}}
Accept: application/json
```

### POST with JSON

```http
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer {{API_TOKEN}}

{
  "name": "Jane Doe",
  "email": "jane@example.com",
  "role": "admin"
}
```

### PUT Update

```http
PUT https://api.example.com/users/{{USER_ID}}
Content-Type: application/json

{
  "name": "Updated Name"
}
```

### DELETE

```http
DELETE https://api.example.com/users/{{USER_ID}}
Authorization: Bearer {{API_TOKEN}}
```

### Multiple Headers

```http
GET https://api.example.com/data
Authorization: Bearer {{TOKEN}}
Accept: application/json
Accept-Language: en-US
X-Request-ID: {{REQUEST_ID}}
X-API-Version: 2
Cache-Control: no-cache
```

## File Organization

A typical workspace structure:

```
my-api-project/
├── .env                    # Default environment
├── .env.development        # Dev overrides
├── .env.production         # Prod overrides
├── users/
│   ├── get-user.http       # GET /users/:id
│   ├── list-users.http     # GET /users
│   └── create-user.http    # POST /users
└── products/
    ├── list.http
    └── create.http
```

## Related

- [Requests](/docs/features/requests) — Working with requests in Mercury
- [Environments](/docs/features/environments) — Setting up `.env` files
