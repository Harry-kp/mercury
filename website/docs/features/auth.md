---
title: Authentication
sidebar_label: Authentication
sidebar_position: 4
---

# Authentication

> Mercury supports common authentication methods. Add credentials without manually encoding headers.

## Overview

Mercury provides a dedicated **Auth tab** for managing authentication. This is more convenient than manually adding `Authorization` headers.

Supported methods:
- **None** — No authentication
- **Basic Auth** — Username and password
- **Bearer Token** — Token-based auth (OAuth, JWT)
- **Custom** — Any custom header format

## Basic Authentication

Basic Auth sends credentials encoded as Base64.

### Using the Auth Tab

1. Open a request
2. Click the **Auth** tab
3. Select **Basic**
4. Enter **Username** and **Password**
5. Mercury generates the header automatically

![Basic Auth tab - Replace with: Screenshot showing Basic Auth form with username/password fields](/img/screenshots/placeholder.png)

### Manual Header

You can also add the header directly:

```http
GET https://api.example.com/protected
Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
```

The value after "Basic " is `username:password` encoded in Base64.

:::tip
Using the Auth tab is easier — Mercury handles the encoding for you.
:::

## Bearer Token

Bearer tokens are commonly used for:
- OAuth 2.0 access tokens
- JWT (JSON Web Tokens)
- API keys in header format

### Using the Auth Tab

1. Open a request
2. Click the **Auth** tab
3. Select **Bearer**
4. Enter your **Token**

Mercury adds: `Authorization: Bearer your-token`

![Bearer Auth tab - Replace with: Screenshot showing Bearer token input field](/img/screenshots/placeholder.png)

### Manual Header

```http
GET https://api.example.com/protected
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Using Environment Variables

Store tokens in your `.env` file:

```bash
# .env
API_TOKEN=your-secret-token
```

Use the variable in your request:

```http
GET https://api.example.com/protected
Authorization: Bearer {{API_TOKEN}}
```

This keeps secrets out of your request files.

## Custom Authentication

For non-standard auth schemes, use Custom mode or add headers directly.

### API Key in Header

```http
GET https://api.example.com/data
X-API-Key: your-api-key
```

### API Key in Query String

```http
GET https://api.example.com/data?api_key={{API_KEY}}
```

### Digest Authentication

Add the header manually:

```http
GET https://api.example.com/protected
Authorization: Digest username="admin", realm="example", ...
```

## Auth Inheritance

The Auth tab settings apply only to the current request. Each `.http` file manages its own authentication.

:::tip Shared Auth
For requests that share the same auth, add the header in each file or use a variable:

```http
Authorization: Bearer {{SHARED_TOKEN}}
```
:::

## Security Best Practices

### 1. Use Environment Variables

Never hardcode secrets in `.http` files:

```http
# ❌ Bad
Authorization: Bearer abc123secret

# ✅ Good
Authorization: Bearer {{API_TOKEN}}
```

### 2. Gitignore Secrets

Add to `.gitignore`:

```gitignore
.env
.env.*
!.env.example
```

### 3. Rotate Tokens Regularly

Update your `.env` file when tokens expire or need rotation.

### 4. Use Different Tokens Per Environment

```bash
# .env.development
API_TOKEN=dev-token-safe-for-testing

# .env.production
API_TOKEN=prod-token-real-data
```

## Troubleshooting

### 401 Unauthorized

- Check if the token/credentials are correct
- Verify the auth method matches what the API expects
- Check if the token has expired
- Ensure variables are defined (look for red indicators)

### Credentials Not Sent

- Make sure you saved the request after adding auth
- Check if the Auth tab shows the correct method selected
- Verify there are no conflicting `Authorization` headers

## Related Features

- [Environments](/docs/features/environments) — Store tokens in environment variables
- [Requests](/docs/features/requests) — Adding headers manually
