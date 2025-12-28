---
title: JSON File Format
sidebar_label: File Format
sidebar_position: 1
---

# JSON File Format Reference

> Mercury stores requests as JSON files — a structured, machine-readable format with excellent tooling support. This page documents the complete specification.

## Overview

A `.json` request file contains a single HTTP request as a JSON object:

```json
{
  "method": "GET",
  "url": "https://api.example.com/users",
  "headers": {},
  "body": ""
}
```

## Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `method` | string | Yes | HTTP method (GET, POST, PUT, etc.) |
| `url` | string | Yes | Full URL including protocol |
| `headers` | object | Yes | Key-value pairs of HTTP headers |
| `body` | string | Yes | Request body (empty string if none) |

## Method

The `method` field specifies the HTTP verb.

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

Methods should be UPPERCASE.

## URL

URLs must include the protocol:

```json
{
  "method": "GET",
  "url": "https://api.example.com/users"
}
```

:::warning
URLs without protocol (e.g., `api.example.com/users`) are invalid.
:::

## Headers

Headers are stored as a JSON object with key-value pairs:

```json
{
  "method": "GET",
  "url": "https://api.example.com/users",
  "headers": {
    "Accept": "application/json",
    "Authorization": "Bearer token123",
    "X-Custom-Header": "my-value"
  },
  "body": ""
}
```

### Common Headers

| Header | Purpose | Example |
|--------|---------|---------|
| `Authorization` | Authentication | `Bearer token123` |
| `Content-Type` | Body format | `application/json` |
| `Accept` | Expected response format | `application/json` |
| `User-Agent` | Client identification | `Mercury/1.0` |
| `Cache-Control` | Caching behavior | `no-cache` |

## Body

The `body` field contains the request payload as a string.

### JSON Body

For JSON payloads, escape the JSON within the string:

```json
{
  "method": "POST",
  "url": "https://api.example.com/users",
  "headers": {
    "Content-Type": "application/json"
  },
  "body": "{\"name\": \"John Doe\", \"email\": \"john@example.com\"}"
}
```

### Form Data

```json
{
  "method": "POST",
  "url": "https://api.example.com/login",
  "headers": {
    "Content-Type": "application/x-www-form-urlencoded"
  },
  "body": "username=john&password=secret"
}
```

### No Body

For GET, HEAD, and OPTIONS, use an empty string:

```json
{
  "method": "GET",
  "url": "https://api.example.com/users",
  "headers": {
    "Accept": "application/json"
  },
  "body": ""
}
```

## Variables

Use `{{variable}}` syntax for dynamic values. Variables work in URL, headers, and body:

```json
{
  "method": "GET",
  "url": "{{BASE_URL}}/users/{{USER_ID}}",
  "headers": {
    "Authorization": "Bearer {{API_TOKEN}}"
  },
  "body": ""
}
```

### Variable Substitution

Variables are replaced with values from `.env` files:

```bash
# .env
BASE_URL=https://api.example.com
USER_ID=12345
API_TOKEN=secret123
```

### Variable Rules

- Variable names are case-sensitive
- Undefined variables are sent as literal text (`{{UNDEFINED}}`)
- No spaces inside braces: `{{VALID}}`, not `{{ INVALID }}`

## Complete Examples

### GET with Authentication

```json
{
  "method": "GET",
  "url": "https://api.example.com/users",
  "headers": {
    "Authorization": "Bearer {{API_TOKEN}}",
    "Accept": "application/json"
  },
  "body": ""
}
```

### POST with JSON

```json
{
  "method": "POST",
  "url": "https://api.example.com/users",
  "headers": {
    "Content-Type": "application/json",
    "Authorization": "Bearer {{API_TOKEN}}"
  },
  "body": "{\"name\": \"Jane Doe\", \"email\": \"jane@example.com\", \"role\": \"admin\"}"
}
```

### PUT Update

```json
{
  "method": "PUT",
  "url": "https://api.example.com/users/{{USER_ID}}",
  "headers": {
    "Content-Type": "application/json"
  },
  "body": "{\"name\": \"Updated Name\"}"
}
```

### DELETE

```json
{
  "method": "DELETE",
  "url": "https://api.example.com/users/{{USER_ID}}",
  "headers": {
    "Authorization": "Bearer {{API_TOKEN}}"
  },
  "body": ""
}
```

### Multiple Headers

```json
{
  "method": "GET",
  "url": "https://api.example.com/data",
  "headers": {
    "Authorization": "Bearer {{TOKEN}}",
    "Accept": "application/json",
    "Accept-Language": "en-US",
    "X-Request-ID": "{{REQUEST_ID}}",
    "X-API-Version": "2",
    "Cache-Control": "no-cache"
  },
  "body": ""
}
```

## File Organization

A typical workspace structure:

```
my-api-project/
├── .env                    # Default environment
├── .env.development        # Dev overrides
├── .env.production         # Prod overrides
├── users/
│   ├── get-user.json       # GET /users/:id
│   ├── list-users.json     # GET /users
│   └── create-user.json    # POST /users
└── products/
    ├── list.json
    └── create.json
```

## Related

- [Requests](/docs/features/requests) — Working with requests in Mercury
- [Environments](/docs/features/environments) — Setting up `.env` files
