---
title: FAQ
sidebar_label: FAQ
sidebar_position: 99
---

# Frequently Asked Questions

Common questions and troubleshooting tips for Mercury.

## General

### What is Mercury?

Mercury is a fast, minimal API client for developers. It's a native desktop app written in Rust — no Electron, no web views. It launches in under 50ms and uses about 30MB of RAM.

### Is Mercury free?

Yes, completely free and open source. No accounts, no subscriptions, no limits.

### What platforms are supported?

- **macOS** — Intel and Apple Silicon (Universal binary)
- **Windows** — x64
- **Linux** — AppImage (most distributions)

### Where is my data stored?

Your data is stored in plain `.http` files in the folder you choose as your workspace. Mercury doesn't upload anything to the cloud.

---

## Migration

### How do I import from Postman?

Direct Postman import is coming soon. For now:

1. In Postman, right-click a request → **Copy as cURL**
2. In Mercury, paste with `⌘+V` / `Ctrl+V`
3. Mercury parses the cURL and fills in the request

### How do I import from Insomnia?

1. In Insomnia, go to **File → Export Data**
2. Export as JSON or YAML
3. In Mercury, click **Import** and select your file
4. Mercury creates `.http` files for each request

### Can I export my Mercury requests?

Yes! Since requests are plain `.http` files, you can:
- Copy them anywhere
- Commit to Git
- Share via email/Slack
- Convert to cURL with `⌘+Shift+C`

---

## Requests

### Why isn't my request sending?

Check these common issues:

1. **URL missing protocol** — Make sure URL starts with `http://` or `https://`
2. **Network issues** — Verify you have internet access
3. **Firewall blocking** — Some corporate firewalls block certain domains

### Why are my variables showing in the request?

Variables like `{{TOKEN}}` appear literally when:

1. The variable is not defined in your `.env` file
2. The `.env` file is not in your workspace root
3. There's a typo in the variable name (case-sensitive)

Look for **red indicators** next to undefined variables.

### How do I send form data?

Use `Content-Type: application/x-www-form-urlencoded`:

```http
POST https://api.example.com/login
Content-Type: application/x-www-form-urlencoded

username=john&password=secret
```

### How do I upload a file?

File uploads (multipart form data) are coming soon. For now, use a base64-encoded body or a separate tool for file uploads.

---

## Environment Variables

### Where do I put my .env file?

Put `.env` files in your workspace root (the folder you opened in Mercury):

```
my-workspace/          ← This folder
├── .env               ← Here
├── .env.development
└── requests/
    └── api.http
```

### Can I have multiple environments?

Yes! Create multiple `.env` files:

- `.env` — Default (always loaded)
- `.env.development` — Dev overrides
- `.env.production` — Prod overrides

Switch between them using the environment selector in the status bar.

### How do I keep secrets out of Git?

Add to your `.gitignore`:

```gitignore
.env
.env.*
!.env.example
```

Create `.env.example` with placeholder values for documentation.

---

## Performance

### Why is Mercury so fast?

Mercury is:

1. **Native** — Written in Rust, compiled to machine code
2. **No Electron** — No bundled browser engine
3. **Minimal** — Does one thing well, no bloat

### How much RAM does Mercury use?

About **30MB** on average, compared to:
- Postman: 300-800MB
- Insomnia: 200-500MB

### What's the binary size?

About **5MB**, compared to:
- Postman: ~500MB
- Insomnia: ~400MB

---

## Troubleshooting

### Mercury won't start

**Mac**: Right-click → Open → Open (bypasses Gatekeeper for unsigned apps)

**Linux**: Make the AppImage executable:
```bash
chmod +x Mercury-*.AppImage
```

**Windows**: If blocked by SmartScreen, click "More info" → "Run anyway"

### Changes in my editor don't appear

Mercury watches your workspace for changes. If sync isn't working:

1. Make sure the file is saved in your editor
2. Check if the file is in your workspace directory
3. Try reloading the workspace

### Request stuck in loading state

If the send button keeps spinning:

1. Press `Esc` to cancel
2. Check your internet connection
3. Verify the URL is reachable
4. Look for firewall or proxy issues

### Response body not displaying

Large responses (>1MB) or binary content show a placeholder instead of the raw content. You can:

1. Click **Save Response** to download the full response
2. View headers to confirm the response arrived

---

## Features

### Does Mercury support GraphQL?

Basic support — you can send GraphQL queries as POST requests with JSON body:

```http
POST https://api.example.com/graphql
Content-Type: application/json

{
  "query": "{ users { id name } }"
}
```

Dedicated GraphQL features (explorer, schema) are coming soon.

### Does Mercury support WebSockets?

Not yet. WebSocket support is on the roadmap.

### Does Mercury support gRPC?

Not yet. gRPC support is on the roadmap.

### Can I sync requests across devices?

Mercury doesn't have built-in sync, but since requests are plain files, you can use:
- Git
- Dropbox / Google Drive / iCloud
- Any file sync tool

---

## Contributing

### Is Mercury open source?

Yes! MIT licensed. View the source at [github.com/Harry-kp/mercury](https://github.com/Harry-kp/mercury).

### How can I contribute?

- **Report bugs** — Open an issue on GitHub
- **Request features** — Open a feature request
- **Submit PRs** — Bug fixes and features welcome

### Where do I report bugs?

[GitHub Issues](https://github.com/Harry-kp/mercury/issues) — please include:
- Mercury version
- OS version
- Steps to reproduce
- Expected vs actual behavior

---

## Still have questions?

Open an issue on [GitHub](https://github.com/Harry-kp/mercury/issues) and we'll help you out!
