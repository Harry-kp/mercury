---
title: Authentication
sidebar_label: Authentication
sidebar_position: 4
---

# Authentication

The **Auth** tab edits the request's `Authorization` header. Mercury doesn't store auth anywhere else. Whatever you enter on the Auth tab is written to that header, and editing the header on the Headers tab updates the Auth tab.

## Choosing a type

Click the ⏷ next to the tab names and pick a type. The tab label shows the current type.

| Type | Header it writes |
|------|------------------|
| **None** | Removes the `Authorization` header |
| **Basic** | `Authorization: Basic <base64 of username:password>` |
| **Bearer** | `Authorization: Bearer <token>` |
| **Custom** | `Authorization: <whatever you type>` |

The type is detected from the header. A value that starts with `Basic` is Basic, a value that starts with `Bearer` is Bearer, any other value is Custom, and no header means None. If you paste `Authorization: Bearer abc` on the Headers tab, the Auth tab switches to Bearer and shows `abc`.

## Basic

Enter a **Username** and **Password**. Mercury encodes them into the header as you type. A preview of the header, with a copy button, appears below the fields.

## Bearer

Paste a token (JWT, OAuth access token, API token). The header preview appears below the field.

## Custom

Type the full header value, for example `ApiKey abc123` or `Digest username="admin", realm="example", ...`.

For API keys sent in another header (such as `X-API-Key`) or in the query string, use the **Headers** or **Params** tab instead.

## Using variables

Auth values are ordinary header text, so `{{variables}}` work:

```
Authorization: Bearer {{API_TOKEN}}
```

For Basic, the username and password are base64-encoded **before** variable substitution. Variables inside the username or password fields are therefore not replaced. To use variables with Basic, add `Authorization: Basic {{BASIC_CREDENTIALS}}` on the Headers tab and put the base64 value in your `.env`. The Auth tab will then show Basic with empty fields. Don't type in them, because that replaces the header.

## Details

- Only the first enabled `Authorization` line is used. A disabled `# Authorization: ...` line is left alone and not sent.
- Auth applies to one request. Nothing is inherited from folders. To share a token, put it in an environment variable and reference it in each request.
- Because auth is a header, it's saved in the request file as plain text. Use a variable for secrets and keep your `.env` files out of Git. See [Environments](/docs/features/environments#keeping-secrets-out-of-git).

## Troubleshooting 401s

- Check that the environment that defines the token is selected. An undefined `{{API_TOKEN}}` is sent literally, and the URL bar shows an amber border.
- Check the Headers tab for a disabled or duplicate `Authorization` line.
- Check whether the token has expired.

## Related

- [Environments](/docs/features/environments)
- [Requests](/docs/features/requests)
