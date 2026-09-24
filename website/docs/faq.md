---
title: FAQ
sidebar_label: FAQ
sidebar_position: 99
---

# FAQ

## General

### What is Mercury?

Mercury is a desktop API client written in Rust with egui. It's a single native binary with no Electron or web view, and it stores requests as plain `.json` files in a folder you choose.

### Is it free?

Yes. Mercury is MIT-licensed and open source, with no account or subscription. The source is at [github.com/Harry-kp/mercury](https://github.com/Harry-kp/mercury).

### Which platforms are supported?

Releases are built for macOS (Apple Silicon and Intel), Linux (x86_64 and ARM64) and Windows (x64). See [Installation](/docs/getting-started#installation).

### Where is my data stored?

- **Requests and environments** are in your workspace folder, as `.json` and `.env*` files.
- **App data** is in `~/.mercury/`: `state.json` (the last session), `recent.json` (unsaved requests you've sent) and `history.json` (request history). Set the `MERCURY_HOME` environment variable to use a different directory.

Mercury doesn't upload anything. Network requests go only to the APIs you call and, if you use the **Help** menu, to GitHub links opened in your browser.

### Are there settings?

No. The timeout (30 s), redirect handling and response size limits are fixed. See [Defaults](/docs/features/requests#defaults).

## Migration

### How do I import from Postman or Insomnia?

Use **Open → Import Postman...** or **Open → Import Insomnia...**. See [Import & Export](/docs/features/import-export) for what gets imported and where the files go.

### How do I get requests out of Mercury?

Request files are plain JSON, so you can copy them, commit them or send them to someone. To share one request, press `⌘ Shift C` to copy it as a cURL command.

## Requests

### My request doesn't send

- The URL needs to include `http://` or `https://`.
- If a status bar message says "Request timed out after 30s", "Connection failed" or "SSL/TLS error", check the network, VPN or proxy.
- Press `Esc` to cancel a request that is stuck.

### My `{{variable}}` is sent literally

The selected environment doesn't define it. Check the environment picker at the top right, and check that the file is in the workspace root. Only one environment is active at a time. See [Environments](/docs/features/environments#troubleshooting).

### How do I send form data?

Set the header and write the body yourself:

```
Content-Type: application/x-www-form-urlencoded
```
```
username=john&password=secret
```

### Can I upload files (multipart)?

No. Multipart bodies aren't supported.

### Does Mercury support GraphQL?

You can send GraphQL as a normal `POST` with a JSON body and `Content-Type: application/json`:

```json
{ "query": "{ users { id name } }" }
```

There's no schema explorer.

### WebSockets, gRPC?

Not supported. Mercury only sends HTTP requests.

### Are cookies kept?

Yes, for the current session only. See [Cookies](/docs/features/requests#cookies).

## Troubleshooting

### Mercury won't start

- **macOS:** if Gatekeeper blocks Mercury, open **System Settings → Privacy & Security** and click **Allow Anyway**.
- **Windows:** at the SmartScreen prompt, click **More info**, then **Run anyway**.

### Edits from my editor don't show up

Mercury reloads the open request when its file changes, **unless you have unsaved edits in Mercury** (shown by a dot after the name in the breadcrumb). In that case Mercury keeps its own version. For edits to `.env` values, select the environment again in the picker.

### A large response isn't displayed

Text bodies over 100 KB aren't shown inline. Click **Save** and open the file in an editor. Responses that declare a `Content-Length` over 10 MB aren't downloaded. See [Performance](/docs/performance#response-size-limits).

## Contributing

Bug reports and pull requests are welcome on [GitHub](https://github.com/Harry-kp/mercury/issues). The **Help → Report Issue** menu item opens the issue tracker. When you report a bug, include your Mercury version, your OS, the steps to reproduce, and what you expected to happen.
