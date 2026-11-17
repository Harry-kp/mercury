# Changelog

All notable changes to Mercury will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Save a response value for the next request.** Right-click a field in a JSON response and choose **Save as `{{token}}` in .env.dev**: the line is written into the environment file and used from the next send. **Copy value** is there too.
- **`.env` is shared by every environment**, as in dotenv: with `.env.dev` selected, `.env` is read first and `.env.dev` overrides it.
- **Trust a self-signed certificate where it fails.** The SSL/TLS error has **Trust host and retry**, which skips the check for that host until Mercury quits. That connection keeps its own cookies and follows redirects only on the same host. No setting, and nothing is remembered.
- **Hover a `{{variable}}` in the URL bar to see its value** and the environment it comes from, even while you type. Before, only the Headers and Body chips showed values, so you opened the `.env` file to check.
- **Import several exports at once.** The Postman and Insomnia pickers take more than one file, so a collection and its environment exports come in together.

### Fixed
- A `.env` value containing a line break (from an import) is written as `\n` and read back intact, instead of splitting into a broken line.
- **Sidebar filter:** a folder that matches by name shows its requests (it showed up empty), `json` no longer matches every request, and a filter that matches nothing says so instead of leaving the sidebar blank.
- Filtering a folder with a symlink back up the tree no longer loads ~100 nested copies of it and widens the sidebar. Very deep trees stop indenting before their rows run out of room.
- Clicking a request no longer rescans the whole workspace (re-reading every visible request file).

### Changed
- **History keeps response bodies up to 1 MB.** It rewrote up to 50 × 10 MB on every send; a bigger body is still shown in full when it arrives.
- **No request timeout.** Slow endpoints (reports, LLM APIs) used to fail at 30 seconds with no way around it. Now only connecting is limited (30 s); `Esc` cancels anything else.
- **The `⋯` beside Send is gone.** Copy as cURL and Revert to saved are on the open request's `⋯` in the sidebar (and in `⌘ K`); Format is in the Body tab, where it already was.
- The sidebar sorts names ignoring case, the way a file manager does: `alpha` no longer comes after `Zeta`.
- The install instructions include `brew trust --cask harry-kp/tap/mercury`: Homebrew 7 refuses casks from untrusted taps, so the one-line install failed.

### Fixed
- **Sidebar filter:** a folder that matches by name shows its requests (it showed up empty), `json` no longer matches every request, and a filter that matches nothing says so instead of leaving the sidebar blank.
- Filtering a folder with a symlink back up the tree no longer loads ~100 nested copies of it and widens the sidebar. Very deep trees stop indenting before their rows run out of room.
- Clicking a request no longer rescans the whole workspace (re-reading every visible request file).

## [0.4.3] - 2026-11-17

The first published build of the 0.4 line: everything below, and everything listed under 0.4.0.

### Added
- **Create and edit environments in Mercury.** The environment menu has **New environment…** (seeded with the variables the open request uses) and **Edit .env.dev…**, a table of variables with **Bulk edit** for the raw file. Before, the menu only said to add `.env` files to the workspace folder.
- Hover the URL bar to see it with every variable filled in, and a variable chip to see its value.
- **Postman environment exports** import as `.env` files, the same way as collections.
- **Insomnia 10+ exports (v5 YAML)** import. They used to fail with "missing field `resources`".
- Postman's `{{$guid}}`, `{{$timestamp}}`, `{{$isoTimestamp}}` and `{{$randomInt}}` work, with a fresh value at every send.

### Fixed
- **Security:** new files (requests, environments, duplicates, imports) are created atomically and never through a symlink. A folder from someone else with a dangling link such as `.env.staging -> ~/.zshenv` could make Mercury create the file it pointed to.
- **Security:** an environment file that is a symlink is ignored. A folder from someone else could link `.env.dev` to `~/.aws/credentials`, whose `key = value` lines read as variables a request could send, or to a file that editing the environment would overwrite.
- **A URL with an undefined variable is no longer sent.** It failed as something else ("invalid international domain name"), or reached the server as `%7B%7BUSER_ID%7D%7D`. The response panel now says which variable is missing and where to define it, and the message goes away when you switch environment.
- **Importing into an open workspace could overwrite your files.** A request or `.env` with the same name as one in the collection was replaced without a word. Imports now go into their own folder in the workspace, and a taken name gets a suffix instead.
- **Insomnia imports didn't work.** Every request failed with "invalid international domain name", because `{{ _.base_url }}` wasn't converted to `{{base_url}}`. Sub-environments (Dev, Production) now include the base environment's values, as in Insomnia; Bearer, Basic and API key auth, form and multipart bodies, and nested folders now come across; and requests outside any folder stay at the top instead of `imported/`.
- Postman requests with **path variables** (`/users/:userId`) were sent with the `:userId` in them; they're filled in now. A raw JSON (or XML, text) body gets the `Content-Type` Postman would have sent.
- After an import, the imported collection's first request is open and its environment selected. An import error says what file Mercury expected, instead of a parser message.

## [0.4.2] - 2026-11-17

Not published: GitHub had no Apple Silicon runner capacity for its macOS build. (0.4.0 went out without installers because a CI output was dropped, and was withdrawn; 0.4.1's Apple Silicon build hit a retired runner image.) Its changes are listed under 0.4.0 and ship in 0.4.3.

## [0.4.0] - 2026-11-17

### Added
- **Multipart bodies and file uploads.** Pick **Multipart** as the body type: a table of fields where a value starting with `@` is a file (**Attach file…** picks one). Pasted `curl -F` commands and Postman form-data bodies now import as Multipart (an imported collection's file fields arrive switched off, a variable's value is always sent as text, and file paths take no variables, so nothing can upload a local file you haven't seen), and Copy as cURL writes them back (`-F` for files, `--form-string` for text).
- **Failed sends are in History**, marked **Failed**: timeouts, refused connections and bad URLs used to leave no trace, though they're the attempts you most want to look at again. Opening one shows the error with **Retry**. Older history files load as before.
- `{{variables}}` in the URL bar are coloured by whether the selected environment defines them, as in Bruno and Postman.
- **Send a sample request** under **Try this**, before you've sent anything: one click opens a new request to a free public JSON API and sends it, so a first run shows a real response without needing an API of your own (Bruno ships a sample collection for the same reason).
- **Revert to saved** (URL-bar `⋯`, `⌘ K`) puts the open file back the way it was when you opened it. Autosave writes every edit within seconds, so this was the missing undo.
- **Rename, Duplicate and Delete for the open request in `⌘ K`**, and a `⋯` on sidebar rows (hover, or the open request) with the same menu as right-click, which nothing on screen advertised.
- **Save…** next to the breadcrumb for a request that isn't a file yet. `⌘ S` was the only way, and it was mentioned only in the first-run tips.
- History entries record the **environment** they were sent with, and the response panel shows it, as **from .env.dev** once you've switched. Reopening an entry opens the **file** it was sent from instead of a detached copy. Older history entries load as before.

### Changed
- **A tidier sidebar.** Everything in it now shares one left edge with room to breathe (headings, rows and text used to start at three different offsets, hard against the panel). With no folder open it shows a proper empty state, like Bruno's and Insomnia's: what Mercury needs, an **Open folder…** button, and Postman and Insomnia imports as rows instead of bare links.
- **Large responses are shown, not refused.** Text bodies over 100 KB used to get a "Large response" placeholder and a Save button. Any text body up to the 10 MB download limit now opens inline, syntax-coloured, and scrolls at full speed: only the lines on screen are laid out (a 6.6 MB JSON costs about 0.1 ms per frame). The pretty-printed form is made on the network thread, and find caches its matches per query.
- **`⌘ K` opens instantly in big workspaces.** It used to open and parse every request file to show its method (about 50 ms at 2,000 requests) and lay out every matching row on every keystroke (5 ms a frame). Now it lists names only, reads a method when its row scrolls into view, and draws only the rows on screen (0.2 ms a frame).
- CI checks that the shipped binary stays within a size budget, runs against the committed `Cargo.lock`, audits dependency changes before they merge, and fails if the shortcut tables in the README or docs drift from the app.
- **Smaller download: 8.7 MB → 7.6 MB.** TLS now uses rustls on the `ring` crypto provider instead of aws-lc (same verification against the system's trusted certificates), and release builds use full link-time optimisation.
- **Sending no longer stutters with a large history.** Each response used to rewrite the whole history file (up to 50 full responses) on the UI thread, about 60 ms per send at 50 MB. It's written in the background now.
- **On macOS the top bar is the title bar.** The grey system title bar is gone; the window buttons sit in Mercury's own top bar, which drags the window and zooms it on double-click.
- **A new look: Graphite Violet.** The neutrals follow the app icon's graphite instead of blue-grey, and the violet accent stays. Syntax colours are calmer (lavender keys, light strings, pink numbers) and the light theme follows the same direction. The layout is flatter: the request editor and the response share one surface, only the URL bar keeps a border, code sits on an inset fill without an outline, and the open request in the sidebar is marked with an accent bar.
- With nothing typed, `⌘ K` lists the requests you opened most recently, then the commands. It used to list every request first, which pushed the commands out of reach in a real workspace.
- **Focus mode** is now called **Hide / show sidebar**, which is what it does (same `⌘ ⇧ F`).
- The history panel says it keeps entries for 7 days, headers and tokens included. Only the docs said so.
- `⌘ S` with no folder open no longer shows a success-ticked notification beside the folder picker; the picker's title says what it's for.
- The sidebar's **Recent** section is now **Unsaved**: it lists requests you sent without saving. Every send is in History.
- New and saved requests land in the folder of the request you opened last (or the folder you clicked last, whichever came later). It used to be whichever folder was clicked last, however long ago.
- The status bar shows whether the open request is saved instead of repeating the workspace path (now the folder chip's tooltip). `⌘ S` on a saved file no longer adds a "Saved" notification, and `⌘ N` no longer announces "New request".
- The sidebar header reads **Requests**, and sidebar menus are menu-width instead of stretching across the editor.
- **A request opens on the tab that holds its content.** Requests with a body (and POST, PUT, PATCH) open on Body, everything else on Params. The tab used to be a single global value, so every GET opened on an empty Body editor, or on whatever the previous request had shown.
- **The auth type is picked inside the Auth tab**, from a None / Basic / Bearer / Custom row. The Auth tab no longer renames itself to "Bearer" or "Basic", and the unlabeled ⏷ next to the tabs is gone.
- **Undefined variables are named under the URL bar** ("`{{USER_ID}}` isn't defined in .env.dev"). Before, the URL bar turned amber and said why only on hover, and Send went ahead with the literal text.
- Find in response, Copy as cURL and History have visible buttons: a search icon in the response toolbar, a `⋯` menu at the end of the URL bar, and a clock icon in the response panel in every state. Each was reachable only by shortcut or the palette.
- The body type reads **Body type: none / JSON / Form / Text**.

### Fixed
- **Unchecking a header deleted it** once the request was saved: request files kept only the headers that are sent. Disabled headers are now saved in a new `disabled_headers` field and come back unchecked. Older Mercury versions ignore the field.
- `⌘ K` opened with its first row scrolled up behind the search box.
- **Find** showed every match the same, so "1 of 2" didn't say which one you were on; the current match is now highlighted in the accent colour. The find field also looks like Mercury's other search fields instead of a bare text box.
- Opening a request from `⌘ K`, history or a relaunch now reveals it in the sidebar; it used to stay hidden inside collapsed folders.
- **Request failed** shows the explanation ("Nothing is listening on … Is the server running?") as plain text, with the network library's message quiet underneath; both used to be pink monospace.
- The **Try this** tips only appear on the first run, not under every unsent request in a workspace, and the environment menu says **No environment** like the chip, not **None**.
- The **⌘ ⏎ to send** hint in the empty response panel was left-aligned under a centred title.
- **About one history entry in eight wouldn't open** once it had been written to disk: its timestamp, which identifies it, was read back from the file a hair off. Timestamps now survive the file exactly.
- **Pasted cURL commands that came out wrong:** repeated `-d` kept only the last one (curl joins them with `&`); `--data-urlencode` was ignored; `-XPOST` and other values glued to their flag were ignored; `-G` sent the data as a POST body instead of the query string; bash's `$'…'` quoting (Chrome's Copy as cURL for bodies with quotes or newlines) kept the `$` and the backslashes; and `-F` took the form field as the URL (it now imports as a Multipart body).
- `.env` files with `export KEY=value` lines or a byte-order mark (some Windows editors add one) lost those variables.
- **Security:** updated the TLS stack (`rustls` 0.23.46, `rustls-webpki` 0.103.17, `webpki-root-certs` 1.0.12) for six advisories, including a ServerHello downgrade (RUSTSEC-2026-0349) and acceptance of distrusted intermediates (RUSTSEC-2026-0382).
- **The Pretty view re-sorted JSON keys and rewrote numbers.** It re-serialised the body, so keys came out alphabetical, integers too large for 64 bits turned into floats, `1.10` became `1.1`, and duplicate keys disappeared. **Format body** did the same to the request you were about to send. Both now only re-indent: the text is exactly what was sent.
- **`⌘ ⇧ C` copied the selected text instead of the request** whenever any text was selected: the selection answered the same copy event after Mercury and won the clipboard. `⌘ ⇧ C` now always copies the request as cURL; plain `⌘ C` still copies a selection.
- Clicking a sidebar row selected its name as text. Only text you would copy is selectable now: the response body, response headers and cookies, and error messages.
- An unsaved GET restored on relaunch opened on the Body tab instead of Params.
- **`⌘ ⇧ C` (Copy as cURL) did nothing outside the tests.** The window layer turns any `⌘ C`, with or without `⇧`, into a copy event and never reports the key, so the shortcut never fired. It listens for that event now.
- **`⌘ F` could crash Mercury under VoiceOver or another accessibility tool.** It put focus on the find field a frame before the field existed, and the accessibility layer rejects a focused element that isn't on screen. Focus requests from shortcuts and the palette now wait for the next frame, and `⌘ F` does nothing when there's no response to search.
- `⌘ K` found nothing for two-word searches such as "list ord": the space had to match a character in the name. Each word now matches on its own.
- Switching the auth type and back (Bearer → Basic → Bearer) threw away the token you had entered. Each type's value is kept while the request is open.
- The Basic auth Username and Password fields had no name for screen readers.
- A blank name in the New request / New folder / Rename dialog did nothing when confirmed; it now says "Type a name first". A name like `../x` is reported as a path rather than "cannot start with a dot".
- On wide windows the response panel now starts wider (up to 720 px instead of 480 px).
- **Every text field reached screen readers as just "text field"**, and header and param checkboxes had no name. Fields are named ("URL", "Filter requests", "Request body", "Key 2"…) and checkboxes say which row they enable. The folder and environment chips draw a focus ring, and Tab now walks the top bar in the order it is drawn.
- At the minimum window size the request and response tab rows were drawn on top of their own controls ("Auth" over the body type; "Cookies" under copy and Pretty/Raw). The controls now move to a second line when there's no room.
- **Pasting a cURL into a saved request overwrote the file.** The cURL replaced the method, headers and body, autosave wrote that over the request file within seconds, and `⌘ Z` only brought back the URL. A cURL pasted over a saved request now opens as a new unsaved request. Typing `curl …` by hand no longer re-parses on every keystroke.
- **Clearing History or Recent was one click with no way back.** The trash icon sits next to the panel's close button. Clearing is still instant, and the notification now has an **Undo** link.
- **`⌘ E` could land on production silently.** Environments cycle in file-name order, so dev → prod was one press away, and the only sign was a chip in the corner. Every switch is now announced, and with a `prod` environment selected, the Send button turns red and reads **Send · prod**.
- **Deleting a folder didn't say what it would take with it.** The confirmation now counts the requests inside and any other files (fixtures, READMEs), which are deleted too.
- **Network errors said what failed but not why.** "error sending request for url (…)" now leads with the cause and a next step, e.g. "Nothing is listening on localhost:3000. Is the server running?", followed by the full error.

## [0.3.1] - 2026-09-29

### Fixed
- **0.3.0 shipped without a macOS `.app`.** `cargo-bundle` 0.12 moved the macOS manifest keys into their own table and rejects unknown ones, so the bundle job failed after every other artifact had already published — and the `.app` is what the Homebrew cask installs. The key has moved and `cargo-bundle` is pinned, so a release no longer depends on what was published upstream that morning.

## [0.3.0] - 2026-09-29

### Added
- **Find in response (`⌘ F`).** Every match in the body is highlighted, the bar counts them, `⏎` steps to the next and `Esc` closes it. A 28 KB response previously had to be read by scrolling.
- A body with no `Content-Type` now says so under the editor, with one click to set JSON, Form or Text. The picker alone was in the corner of the tab row, which is not where anyone looks while typing a body.
- **A body type next to the Body tab** — JSON, Form, Text or none — which writes the `Content-Type` header for you. A JSON body used to be sent with no `Content-Type` at all, so a first POST failed against most APIs. Picking **Form** edits `a=1&b=2` as a table and percent-encodes the values, which is what an OAuth token request needs.
- Mercury reopens the request file you had open when you quit, instead of restoring its contents as an "Untitled" unsaved request.
- **A light theme.** Mercury follows your system appearance and `⌘ D` overrides it. The choice is remembered.
- **A command palette (`⌘ K`)** over every request in the workspace and every command Mercury has. Arrow keys move, `⏎` opens, `Esc` closes.
- The response panel has **Body / Headers / Cookies** tabs instead of checkboxes.
- The request editor's `⋯` menu, the workspace menu and the environment picker replace the old Help/Open menus.

### Fixed
- **A Postman collection with a valueless query parameter failed to import at all.** `?archived` — a parameter Postman exports with no `value` field — aborted the whole import with an error that named the URL parser rather than the parameter. Imports now survive every shape a real export contains.
- **Imported requests arrived without their auth or their body.** Collection-, folder- and request-level Bearer, Basic and API-key auth were dropped, so every imported request came back 401; URL-encoded bodies were dropped, so a login POST sent nothing under a `Content-Type` that promised a form. Both are carried over now, and whatever Mercury genuinely cannot express — OAuth 2, `form-data`, GraphQL — is **named in the notification** instead of vanishing.
- **Importing a collection left the previous request on screen.** The editor still held whatever scratch request was open, often from a folder that was no longer even open, and the collection you just imported was nowhere to be seen. Mercury now opens the first request it imported.
- **An HTML or plain-text response was one endless horizontal line.** Only JSON and XML get re-formatted, so everything else kept the server's single line and had to be read by scrolling sideways through the whole document — which is exactly what you get from a proxy's error page. Unformatted bodies now wrap; formatted JSON still doesn't, so its indentation survives.
- **`⌘ S` on an unsaved request asked you to "Create" a "New request".** It now says Save request / Save.
- **"Imported 3 requests, 1 environments."**
- **Every send raised a toast repeating the status already shown at the top of the response.** The toast is now kept for the cases where the status isn't visible — a response that arrived for a different request, or history covering the pane — and a new send clears the old one, so a stale "404 Not Found" can no longer sit over a connection failure.
- **The URL bar was outlined in amber for an undefined variable that wasn't in the URL.** A `{{token}}` in the headers marked the URL bar, sending you to look in the wrong place, and the explanation was hover-only. The tab that holds the variable now carries the warning.
- **Tab moved the keyboard focus but nothing showed where it had gone.** Every button, tab, list row and the method picker is drawn by hand, and egui only marks focus on the widgets it draws itself. They now paint a focus ring, so the app can be driven from the keyboard without guessing.
- **Sidebar rows, the method picker and the status-bar links were invisible to screen readers.** They reached the accessibility tree as anonymous boxes, so VoiceOver announced nothing for the request list — the app's main navigation. Each one now carries its name.
- **Text that was too faint to read, in both themes.** Hints, counters, timestamps and JSON punctuation sat as low as 2.4:1 against their background; the light theme was the worse of the two. Every text colour now clears WCAG AA (4.5:1) on every surface it lands on, and a test keeps it that way.
- **The delete button's label was white on pink in the dark theme** — 2.7:1, the least readable text in the app.
- **The sidebar grew 2px wider every frame, and sprang back after you resized it.** A text field ended up two pixels wider than the space it was given — a frame occupies `content + margin + 2 × stroke width`, and only the margin was being subtracted. Inside a panel that feeds back into the panel width, so it crept outward forever. The response panel did the same whenever history was open.
- **The macOS app was Apple Silicon only, despite being published as "universal".** Intel Macs could not run the Homebrew cask at all. The bundle is now a real universal binary, and the release fails if it ever isn't.
- **macOS called the app "damaged".** The bundle claimed a sealed signature it did not have, so Gatekeeper refused it outright instead of offering the usual unidentified-developer prompt. It is now ad-hoc signed and sealed, so right-click → Open works.
- **Windows opened a console window behind the GUI.**
- **Compressed responses rendered as mojibake.** Any server with gzip on — and every cURL pasted from a browser, which always asks for gzip — produced garbage. Mercury now decodes gzip, brotli and deflate.
- **`localhost:3000/api` failed with "Invalid URL".** A URL without a scheme gets one: `http` for loopback, `https` otherwise.
- **A request or folder named `../x` was created outside the folder you picked.** Names are now a single file name; `/`, `\`, `..` and a leading `.` are rejected.
- A response that arrived after you opened a different request was shown as *that* request's response. It now goes to history only, and says so.
- Deleting the workspace folder on disk left Mercury claiming it was still open, with buttons that could only fail.
- A 4xx or 5xx was announced with a green tick.
- The response body could not be selected in Pretty mode — only copied whole.
- Binary and large responses told you to "Use Save" without showing one; they now offer a **Save response…** button, and the useless copy icon is gone for bodies that are not text.
- Recent grew without limit in the sidebar while the file kept only 50, and 50 entries pushed the collection off the bottom. It is capped, scrolls, and has a clear button.
- Recent and history rows truncated URLs at a fixed character count instead of the width actually available.
- Failing to open a request printed the whole absolute path across three lines.
- Clicking a folder while filtering silently flipped its expanded state.
- The sidebar filter only searched folders you had already expanded, so a request two folders deep could not be found.
- Pressing Enter in the New request / New folder / Rename dialog did nothing, and the name field did not take focus — you had to click it and then click Create.
- History paired a response with whatever was in the editor when it arrived, not with the request that was actually sent.
- Response headers were capped at ~170px, hiding all but the first few. Headers and Cookies now fill the panel.
- A failed request reported the same error twice, once in the panel and once as a notification.
- The notification after a request read "200 200 OK".
- Formatted JSON wrapped long values onto an unindented next line; it now keeps its structure and scrolls.
- The response panel grew wider when it showed an empty state or a long error, shifting the whole layout.
- At the minimum window size the editor was squeezed to about 50px wide, leaving no room for the URL.
- Icon buttons, tabs and badges had no accessible names, so screen readers announced nothing.
- Sub-millisecond responses showed "0 ms".
- Crash when a URL or error message contained non-ASCII characters.
- Query params with UTF-8 (`%C3%A9`) were decoded incorrectly.
- Copy as cURL included disabled (`#`) headers and didn't escape quotes.
- Pasting a cURL command changed escaped quotes inside single-quoted bodies.
- Typing `?` in a text field opened the shortcuts dialog.
- The shortcuts dialog listed shortcuts that don't exist (⌘I, "Clear Console").
- The "Request completed" message never faded.
- Auto-save didn't run until the first manual ⌘S.
- An open request didn't refresh after being edited outside Mercury.
- Custom auth values couldn't contain trailing spaces while typing.
- The Auth tab could show the previous request's credentials.
- CONNECT/TRACE requests came back as GET after a restart.
- `.env` files in subfolders showed up in the picker but never loaded (only root `.env*` files count now).
- Import failures were silent. Insomnia request names with `/` could write outside the target folder. Postman `{{vars}}` in URL paths were percent-encoded.
- Trailing spaces were stripped from header values.
- Renaming a request without typing `.json` made it disappear from the sidebar.
- Clicking a Recent item could drop the open file's last few seconds of edits.
- **Save** was offered for binary responses restored from history, and wrote a placeholder instead of the file.
- Imported environment keys like `api-key` were renamed to `api_key`, so `{{api-key}}` stopped resolving. Imported values containing quotes were corrupted.
- Editing the selected `.env` file had no effect until you re-selected it.

### Changed
- **The interface was rebuilt.** New palettes (light and dark), a real type scale, hand-drawn vector icons in place of emoji, list rows with hover and selection, a prominent Send button, a filter box in the sidebar, floating notifications, and dialogs on a dimmed backdrop.
- Mercury ships Inter and JetBrains Mono (subset, ~200 KB) so text looks the same on every platform, and no longer scales the whole UI by 1.25.
- `⌘ K` opens the command palette; the sidebar filter box is always visible instead.
- The status bar shows the workspace path relative to `~`.
- Side panel widths are derived from the window size; the minimum window is 900x620 and the default 1280x840.
- **Pretty / Raw** is a two-state switch instead of a link that showed the current mode.
- `⌘ S` on an unsaved request saves into the collection you have selected, not always the workspace root.
- Request files write headers in sorted order, for stable git diffs.
- Error messages include the actual cause (file, reason) instead of generic advice.
- Removed the "About Mercury" menu item, which did nothing.
- Postman import is offered next to Insomnia in the empty sidebar.

### Technical
- `ui/theme.rs` holds both palettes behind one `Theme` shape; `ui/icon.rs` draws every icon from a 24x24 geometry table.
- `AppState` gained a `theme` field (defaults to following the system).
- Flattened the codebase into domain modules, with one implementation per concept (~30% less code).
- Added a headless UI smoke test (egui_kittest).
- Added CLAUDE.md and a `/fix-issue` Claude Code skill. CI now runs on every PR.

## [0.2.0] - 2025-12-20

### Added
- **Centralized Error Handling**: Robust error management with `thiserror` and user-friendly "fading toast" notifications.
- **Request Cancellation**: Dedicated button to abort active or stalled requests instantly.
- **Enhanced cURL Support**: Expanded parser with support for `--user`, `--proxy`, `--cookie`, `--head`, and more.
- **Advanced HTTP Methods**: Support for `CONNECT` and `TRACE` methods in the request editor and dropdown.
- **JSON Beautification**: Added a sparkle icon (✨) for one-click pretty-printing of request bodies.
- **Standardized UI**: Unified iconography across the application (Chevrons, Cross, Checkmarks, and Tooltips).
- **Dot Indicator**: Visual cue for unsaved changes with a descriptive tooltip.

### Changed
- **Unified Navigation**: Replaced folder icons with modern chevrons for consistent collection browsing.
- **Deduplicated UI Components**: Consolidated modal and context menu logic for better consistency.
- **Optimized Rendering**: Reduced memory allocations and improved UI snappiness by minimizing object clones.

### Fixed
- **Icon Rendering**: Resolved "square box" display issues for several emojis (EDIT, WARNING) across different systems.
- **Installation UX**: Refined the one-line installer scripts for smoother onboarding on macOS and Linux.

### Technical
- **Modular Architecture**: Complete codebase restructuring for improved maintainability.
- **Automation**: Integrated `cargo fmt` and `cargo audit` into the development workflow.
- **Testing**: Expanded unit test coverage.

## [0.1.0-beta] - 2025-12-10

### Added
- HTTP methods: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT, TRACE
- Plain-text request files
- Environment variables with `.env` files
- Two-way file sync with external editors
- Request history timeline
- cURL import/export
- Keyboard shortcuts for fast workflow
- Focus mode
- Request search
- Syntax highlighting
- Dark theme
- Folder-based organization

### Technical
- Built with Rust and egui
- Native performance, single small binary
- 100% local storage
- Cross-platform: macOS, Linux, Windows
- **Easy Installation**: One-line installer for macOS (`curl | bash`)
- **Ad-hoc Signing**: Improved Apple Silicon support with hardened runtime entitlements
- No telemetry, no accounts
