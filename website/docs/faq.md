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

No. Redirect handling and the response size limit are fixed, and there's no timeout (`Esc` cancels). See [Defaults](/docs/features/requests#defaults).

## Migration

### How do I import from Postman or Insomnia?

Use **Import from Postman…** or **Import from Insomnia…** in the workspace menu (the folder chip in the top bar). See [Import & Export](/docs/features/import-export) for what gets imported and where the files go.

### How do I get requests out of Mercury?

Request files are plain JSON, so you can copy them, commit them or send them to someone. To share one request, press `⌘ Shift C` to copy it as a cURL command.

## Requests

### My request doesn't send

- The URL needs to include `http://` or `https://`.
- The failed-request panel names the cause (nothing listening, host not found, untrusted certificate, can't connect within 30s) and shows the full error below it. For anything network-related, check the network, VPN or proxy.
- Press `Esc` to cancel a request that is stuck.

### My `{{variable}}` is sent literally

The selected environment doesn't define it. Check the environment picker at the top right, and check that the file is in the workspace root. Only one environment is active at a time. See [Environments](/docs/features/environments#troubleshooting).

### How do I send form data?

Pick **Form** in the Body tab and fill in the table. Mercury sets `Content-Type: application/x-www-form-urlencoded`. See [Body types](/docs/features/requests).

### Can I upload files (multipart)?

Yes. Pick **Multipart** in the Body tab and use **Attach file…**, or write `name=@/path/to/file`.

### Does Mercury support GraphQL?

You can send GraphQL as a normal `POST` with a JSON body and `Content-Type: application/json`:

```json
{ "query": "{ users { id name } }" }
```

There's no schema explorer.

### WebSockets, gRPC?

Not supported. Mercury only sends HTTP requests.

### My dev server uses a self-signed certificate

The request fails with an SSL/TLS error and a **Trust host and retry** button. It skips the certificate check for that host until Mercury quits. A certificate from a CA your system trusts (your company's, or one made with `mkcert`) just works.

### Are cookies kept?

Yes, for the current session only. See [Cookies](/docs/features/requests#cookies).

## Troubleshooting

### Mercury won't start

- **macOS:** if Gatekeeper blocks Mercury, open **System Settings → Privacy & Security** and click **Open Anyway**.
- **Windows:** at the SmartScreen prompt, click **More info**, then **Run anyway**.

### Edits from my editor don't show up

Mercury reloads the open request when its file changes, **unless you have unsaved edits in Mercury** (shown by a dot after the name in the breadcrumb). In that case Mercury keeps its own version. For edits to `.env` values, select the environment again in the picker.

### A large response isn't displayed

Text bodies of any size are shown inline and scroll smoothly. Responses that declare a `Content-Length` over 10 MB aren't downloaded. See [Performance](/docs/performance#response-size-limits).

## Contributing

Bug reports and pull requests are welcome on [GitHub](https://github.com/Harry-kp/mercury/issues). **Report an issue** in the `⋯` menu opens the issue tracker. When you report a bug, include your Mercury version, your OS, the steps to reproduce, and what you expected to happen.
