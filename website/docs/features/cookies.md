---
title: Cookie Management
sidebar_label: Cookies
sidebar_position: 7
---

# Automatic Cookie Management

> Cookies are handled automatically — no configuration needed.

## How It Works

Mercury maintains a **session-wide cookie jar**:

1. When a server responds with `Set-Cookie`, the cookie is stored
2. Subsequent requests to the same domain **automatically include** the cookie
3. Cookies are scoped by domain (no cross-domain leaks)
4. All cookies are cleared when you restart Mercury

## Testing Auth Flows

This makes login flows work naturally:

```http
# 1. Login - server returns Set-Cookie: session=abc123
POST https://api.example.com/login
Content-Type: application/json

{"username": "user", "password": "pass"}
```

```http
# 2. Dashboard - cookie is sent automatically!
GET https://api.example.com/dashboard
# Cookie: session=abc123 (auto-sent)
```

## Viewing Cookies

In the response panel, a **Cookies (N)** checkbox appears when cookies are received.

Click it to expand and see:
- Cookie name (in purple)
- Cookie value (in gray)
- Copy button to copy all cookies

## Clearing Cookies

**Restart Mercury** to clear all stored cookies.

This is intentional — cookies live only in memory for the current session.

:::tip Why no persistence?
Mercury follows the principle of "manual over automatic." Session cookies clearing on restart:
- Prevents stale sessions
- Matches expected dev workflow (fresh start = fresh auth)
- Avoids hidden state that could cause confusion
:::

## Supported Use Cases

- ✅ Login flows (POST /login → GET /protected)
- ✅ OAuth token exchange
- ✅ Session-based APIs
- ✅ CSRF cookie handling

## Related Features

- [Environments](/docs/features/environments) — Managing API keys and tokens
- [Requests](/docs/features/requests) — Sending HTTP requests
