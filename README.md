<p align="center">
  <img src="assets/icons/icon.png" alt="Mercury" width="80" height="80">
</p>

<h1 align="center">Mercury</h1>

<p align="center">
  <strong>API Testing for Purists.</strong><br>
  Instant startup. 60fps UI. $0 forever.
</p>

<p align="center">
  <a href="https://harry-kp.github.io/mercury/docs/getting-started">Documentation</a> •
  <a href="https://github.com/Harry-kp/mercury/releases">Download</a> •
  <a href="https://github.com/users/Harry-kp/projects/5">Roadmap</a> •
  <a href="#your-requests-are-files">Requests are files</a> •
  <a href="#why-mercury">Why Mercury</a> •
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <a href="https://github.com/Harry-kp/mercury/releases"><img src="https://img.shields.io/github/v/release/Harry-kp/mercury?style=flat-square&color=a78bfa" alt="Release"></a>
  <img src="https://img.shields.io/badge/homebrew-cask-orange?style=flat-square&logo=homebrew" alt="Homebrew Cask">
  <a href="https://github.com/Harry-kp/mercury/actions"><img src="https://img.shields.io/github/actions/workflow/status/Harry-kp/mercury/ci.yml?branch=master&style=flat-square&label=build" alt="Build Status"></a>
  <a href="https://github.com/Harry-kp/mercury/blob/master/LICENSE"><img src="https://img.shields.io/github/license/Harry-kp/mercury?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue?style=flat-square" alt="Platform">
</p>

<p align="center">
  <a href="https://github.com/Harry-kp/mercury/stargazers"><img src="https://img.shields.io/github/stars/Harry-kp/mercury?style=social" alt="GitHub stars"></a>
  <a href="https://github.com/Harry-kp/mercury/issues"><img src="https://img.shields.io/github/issues/Harry-kp/mercury?style=flat-square" alt="Issues"></a>
  <a href="https://github.com/Harry-kp/mercury/pulls"><img src="https://img.shields.io/github/issues-pr/Harry-kp/mercury?style=flat-square" alt="Pull Requests"></a>
  <a href="https://github.com/Harry-kp/mercury/discussions"><img src="https://img.shields.io/github/discussions/Harry-kp/mercury?style=flat-square" alt="Discussions"></a>
</p>

<p align="center">
  <img src="assets/media/demo.gif" alt="Mercury: open a collection, send a request, find anything in the response" width="100%">
</p>

<p align="center">
  <sub><a href="assets/media/demo.mp4">Watch in higher quality (MP4, 32s)</a></sub>
</p>

## Quick start

```bash
brew install --cask harry-kp/tap/mercury
```

Launch from **Applications**, or run `mercury`. Universal build — Apple Silicon and Intel.

> The first launch says the developer cannot be verified: right-click Mercury in Applications and choose **Open**. Mercury is unsigned because a Developer ID costs $99/year.

<details>
<summary><strong>Windows, Linux, or without Homebrew</strong></summary>

**macOS / Linux**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Harry-kp/mercury/releases/latest/download/mercury-installer.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://github.com/Harry-kp/mercury/releases/latest/download/mercury-installer.ps1 | iex
```

Then run `mercury`. The installer puts the binary in `~/.cargo/bin` — if the shell can't find it, restart your terminal.

On Windows, SmartScreen says "Windows protected your PC": click **More info** → **Run anyway**.

On Linux, to get it in your app launcher:

```bash
cat > ~/.local/share/applications/mercury.desktop << 'EOF'
[Desktop Entry]
Name=Mercury
Exec=$HOME/.cargo/bin/mercury
Type=Application
Categories=Development;
EOF
```

**Manual download** — [every platform, from Releases](https://github.com/Harry-kp/mercury/releases).

**From source** — `git clone`, then `cargo build --release`.

</details>

<details>
<summary><strong>macOS says Mercury is damaged, or won't open</strong></summary>

Right-click in Applications → **Open** is the normal path. If macOS refuses outright:

```bash
xattr -d com.apple.quarantine /Applications/Mercury.app
```

For the shell installer's binary rather than the app bundle:

```bash
xattr -d com.apple.quarantine ~/.cargo/bin/mercury
```

The Homebrew cask ships a signed, sealed universal `.app`. Don't hand-build a `Mercury.app` around the bare binary — an unsigned bundle is what makes macOS call it damaged instead of merely unverified.

</details>

---

## Your requests are files

A workspace is just a folder. Subfolders are collections, and every `*.json` file is one request:

```
api/
├── .env.dev
├── .env.production
├── users/
│   ├── list-users.json
│   └── create-user.json
└── health.json
```

```json
{
  "method": "POST",
  "url": "{{BASE_URL}}/users",
  "headers": {
    "Authorization": "Bearer {{TOKEN}}",
    "Content-Type": "application/json"
  },
  "body": "{\"name\": \"Ada\"}"
}
```

Grep it, diff it, commit it, edit it in VS Code — Mercury picks up outside edits live. `headers`, `body` and `disabled_headers` (headers you unchecked) are optional, and headers are written in sorted order so git diffs stay clean.

**Environments** are `.env` files in the workspace root. Pick one from the top-right menu or cycle with `⌘ E`; `{{NAME}}` is substituted into the URL, headers and body when you send. Undefined variables are named under the URL bar and flagged on the tab that holds them. Production shows in red, staging in amber, and every switch is announced; with prod selected, Send reads **Send · prod**.

```bash
# .env.dev
BASE_URL=http://localhost:3000
TOKEN="dev token"
```

---

## Why Mercury

- **Native, not Electron.** Rust + egui draw directly on the GPU. One ~8 MB binary, no runtime, no splash screen: open in a third of a second, idle at under 100 MB.
- **Local only.** No account, no cloud, no telemetry. Your secrets stay on your disk.
- **Keyboard first.** `⌘ K` opens a palette over every request and command, `⌘ F` finds anything in a response, `?` lists the rest.
- **No setup tax.** Type `localhost:3000/api`, pick JSON or Form, hit send. Mercury fills in the scheme, the `Content-Type` and the decoding so the first request works.
- **Light or dark.** Follows your system theme, or `⌘ D` pins one. Vector icons and bundled fonts, so it looks the same on every OS.

### How it compares

| App | Launch to window (warm) | First launch | Idle memory (all processes) | Size on disk |
|---|---|---|---|---|
| **Mercury** 0.3.1 | **0.3 s** | **0.7 s** | **88 MB** | **22 MB** |
| Postman 12.37 | 0.4 s ¹ | 2.9 s | 753 MB | 471 MB |
| Bruno 4.3 | 1.9 s | 4.4 s | 555 MB | 501 MB |
| Insomnia 13.4 | 2.4 s | 7.2 s | 828 MB | 1.2 GB |

Measured on an M-series MacBook (macOS 15), each app freshly installed, nothing open in it, median of three warm launches. Memory is every process the app starts (Electron apps run several), 5 seconds after the window appears. ¹ Postman's first window is a loading screen; its UI arrives later. Reproduce with `swift scripts/bench-launch.swift /Applications/*.app`.

### What's in it

- **Collections** — create, rename, duplicate and delete from the sidebar; outside edits sync live and changes auto-save.
- **Auth** — Basic, Bearer or a custom `Authorization` value, always in sync with the Headers tab.
- **Query params** — a table that stays in sync with the URL, both ways.
- **cURL** — paste a `curl …` command into the URL bar to import it; `⌘ ⇧ C` copies the request back out.
- **Import** — Postman v2.1 collections and environments, Insomnia (v4 and the v5 YAML of Insomnia 10+), with their variables, auth and bodies. Imports never overwrite your files.
- **Responses** — pretty-printed and highlighted JSON, XML and HTML; headers and cookies; `⌘ F` to search; save images, binaries or large bodies to a file.
- **History** — your last 50 requests from the past 7 days, with full responses. **Unsaved** in the sidebar keeps requests you've sent without saving. Each history entry records its environment and reopens on its file.
- **Cookies** — kept for the session, so login flows just work.

---

## Shortcuts

`⌘` is `Ctrl` on Windows and Linux. Press `?` in the app for this list.

| Shortcut | Action |
|----------|--------|
| `⌘ ⏎` | Send request |
| `⌘ K` | Command palette |
| `⌘ N` | New request |
| `⌘ S` | Save request |
| `⌘ O` | Open folder |
| `⌘ L` | Focus URL bar |
| `⌘ E` | Next environment |
| `⌘ H` | Toggle history |
| `⌘ R` | Toggle raw response |
| `⌘ D` | Toggle light / dark |
| `⌘ F` | Find in response |
| `⌘ ⇧ C` | Copy as cURL |
| `⌘ ⇧ F` | Hide / show sidebar |
| `?` | Keyboard shortcuts |
| `Esc` | Close find / cancel request / close dialog / clear filter |

`Tab` moves between controls and `Space` activates the one you land on, so everything is reachable without a mouse.

---

## Defaults

There's no settings screen. These are fixed:

| | |
|---|---|
| Timeout | 30 seconds |
| Redirects | Followed (up to 10) |
| Largest response downloaded | 10 MB |
| Largest body shown inline | Any size up to the download limit, syntax-coloured |
| App data | `~/.mercury/` (session, recent, history). Set `MERCURY_HOME` to use another folder. |

---

## What Mercury is NOT

We deliberately don't build cloud sync, team collaboration, AI assistants, plugins, user accounts or analytics. They aren't missing features; we chose to leave them out.

---

## Contributing

PRs are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md). The repo is set up for Claude Code: [CLAUDE.md](CLAUDE.md) holds the conventions, and `/fix-issue <number>` runs the whole fix-to-merge loop.

```bash
cargo run      # run the app
cargo test     # unit tests + a headless UI smoke test
```

## License

MIT. Do whatever you want.

---

<p align="center">
  Built with obsessive minimalism.<br>
  <a href="https://github.com/Harry-kp">@Harry-kp</a>
</p>
