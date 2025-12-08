# Mercury API Client

## Product Requirements Document (37signals Edition)

### The Big Idea

**Make API testing feel like using a text editor, not launching Photoshop.**

Every other API client wants to be a Swiss Army knife with 47 attachments. We're building a sharp knife that does one thing perfectly: send HTTP requests fast.

### What We're NOT Building

- Team collaboration features (use Git)
- Cloud sync (use Dropbox/iCloud)
- Mock servers (use a real server)
- API documentation generators (write docs separately)
- GraphQL support (that's a different tool)
- Automated testing suites (use your CI/CD)
- History/audit logs beyond what Git gives you
- User accounts or login systems

**Why?** Because every feature is a mortgage. You pay for it monthly in complexity, maintenance, and cognitive load.

---

## Core Philosophy

### 1. Files Over Databases

Everything is a `.http` file on disk. Period.

```
my-api/
  auth/
    login.http
    refresh.http
  users/
    create-user.http
    get-user.http
```

**Benefits:**
- Git works naturally
- Grep works
- Your text editor works
- No export/import mental model
- No lock-in
- No "sync conflicts"

### 2. Convention Over Configuration

The app reads your directory structure. That's it.

- Folders = Collections
- `.http` files = Requests
- `.env` files = Environments (standard dotenv format)
- No project files, no workspace files, no `.insomnia` folder

**Open a folder. Start working.**

### 3. Fast By Default

- Launch time: < 100ms
- Memory: < 30MB
- File parsing: Instant (we're reading text files, not decoding protobuf)
- Response streaming: Yes
- No splash screens, no loading spinners

---

## Feature Set (The Essential Few)

### What IS Included

#### 1. The Request Editor

- Method dropdown + URL bar
- Headers as `Key: Value` pairs
- Body editor with basic syntax coloring
- Send button
- That's it

#### 2. The Response Viewer

- Status code (big and obvious)
- Response time in milliseconds
- Body with auto-formatting (JSON, XML, HTML)
- Headers list
- Raw view option

#### 3. The Sidebar

- Shows your folder structure
- Click to open a request
- Search to filter (fuzzy search on filenames)
- Context menu: Duplicate, Rename, Delete
- Color-coded by HTTP method

#### 4. Environment Variables

Standard `{{variable}}` syntax in your `.http` files.

```
GET https://{{host}}/api/users
Authorization: Bearer {{token}}
```

Environment switcher in the header bar reads from `.env` files:
- `.env.local`
- `.env.dev`
- `.env.production`

**No GUI editor for .env files.** Just open them in your text editor. You're a developer.

#### 5. Import From Competitors

**One-time migration only:**
- Drop a Postman collection → Converts to folder structure + `.http` files
- Paste a cURL command → Creates a new `.http` file

**No export feature.** Your files are already portable. Just zip the folder.

#### 6. Copy as cURL

Right-click any request → "Copy as cURL"

For when you need to share with someone who doesn't use Mercury.

---

## What Makes This Different

### The 37signals Cuts

**Removed: Team Features**
- No user management
- No permissions
- No sharing
- No comments

Use Git for collaboration. It already works.

**Removed: Cloud Everything**
- No accounts
- No cloud sync
- No online dashboard

Put your project folder in Dropbox/iCloud/Git if you want sync.

**Removed: Advanced Features**
- No code generation
- No automation
- No test scripting
- No pre-request scripts

If you need that, you need a different tool (or actual automated tests in your codebase).

**Removed: The Kitchen Sink**
- No GraphQL queries
- No websockets
- No gRPC
- No SOAP

Build separate tools for separate protocols. Don't cram everything into one app.

---

## Technical Decisions

### Stack
- **Rust** for performance
- **egui** for UI (native, immediate mode, GPU-accelerated)
- **reqwest** for HTTP
- Standard filesystem APIs

### Why egui over Electron?

| Metric | Electron | egui |
|--------|----------|------|
| Launch time | 3-5 seconds | <100ms |
| Memory | 300-500MB | <30MB |
| Installer size | 150MB+ | 15MB |

### Platform Support
- macOS (11+)
- Windows (10+)
- Linux (Ubuntu, Fedora, Arch)

---

## The UI (Opinionated & Minimal)

```
┌────────────────────────────────────────────────────────────┐
│  Mercury  [Dev ▾]  [Search]                     ⚙ Settings │
├──────────┬─────────────────────────────────────────────────┤
│          │ GET ▾  https://api.example.com/users    [Send]  │
│ Projects ├─────────────────────────────────────────────────┤
│          │                                                  │
│ ▾ auth   │ Headers                                          │
│   login  │ Authorization: Bearer {{token}}                  │
│   logout │                                                  │
│          │ Body                                             │
│ ▾ users  │ (none)                                           │
│   list   │                                                  │
│   create │                                                  │
│   delete ├─────────────────────────────────────────────────┤
│          │ Response                                         │
│          │ 200 OK • 234ms • 2.1KB                           │
│          │                                                  │
│          │ {                                                │
│          │   "users": [...]                                 │
│          │ }                                                │
└──────────┴─────────────────────────────────────────────────┘
```

### Color Palette (Calm, Professional)
- Background: #1e1e1e (dark) or #ffffff (light)
- Accent: Single brand color only
- Syntax: Subtle, not rainbow vomit

### Keyboard Shortcuts (Just The Essentials)
- `Cmd/Ctrl + N` → New request
- `Cmd/Ctrl + Return` → Send
- `Cmd/Ctrl + P` → Quick open (fuzzy search)
- `Cmd/Ctrl + K` → Focus search
- `Cmd/Ctrl + ,` → Settings

No 30-item keyboard shortcut reference manual.

---

## Pricing & Distribution

**Free.**

Open source (MIT license).

Why? Because the API client market doesn't need another SaaS subscription. The world has enough of those.

We make money by building Basecamp and HEY, not by nickel-and-diming developers $12/month for basic tools.

---

## Development Phases

### Phase 1 (Week 1-2): The Core Loop
- Parse and execute `.http` files
- Display request/response
- Basic UI with egui

### Phase 2 (Week 3-4): The File System
- Sidebar with folder navigation
- Create/rename/delete operations
- Search/filter

### Phase 3 (Week 5): Migration Tools
- Postman collection importer
- cURL command parser

### Phase 4 (Week 6): Polish
- Custom styling
- Keyboard shortcuts
- Installer packaging

### Ship It (Week 7)
- Launch blog post
- GitHub release
- Done

---

## Success Metrics

**Not this:**
- Daily active users
- Engagement time
- Feature adoption rates

**This:**
- Installation time: < 1 minute
- First API call: < 30 seconds from launch
- Resource usage: Invisible in Activity Monitor
- Support requests: Minimal (because it's simple)

---

## The Pitch

> "Insomnia takes 4 seconds to launch and uses half a gigabyte of RAM to send HTTP requests.
> 
> Mercury launches instantly, uses 30MB, and your requests are just text files that work with Git.
> 
> Import your Insomnia collection in one click. You'll never go back."

---

**This is the product.**

Everything else is just clutter.