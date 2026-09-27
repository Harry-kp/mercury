---
title: Keyboard Shortcuts
sidebar_label: Keyboard Shortcuts
sidebar_position: 2
---

# Keyboard Shortcuts

On macOS `⌘` is Command. On Windows and Linux it's `Ctrl`.

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
| `⌘ ⇧ F` | Focus mode |
| `?` | Keyboard shortcuts |
| `Esc` | Close find / cancel request / close dialog / clear filter |

Press `?` (when you aren't typing in a field), click **Shortcuts** in the status bar, or pick **Keyboard shortcuts** from the `⋯` menu to see this list in the app.

## Notes

- **`⌘ N`** clears the editor for a new, unsaved request. It saves the current request first if it has a file.
- **`⌘ S`** on an unsaved request asks for a name and saves it to the selected collection (the workspace root if none is selected). See [Saving](/docs/features/requests#saving).
- **`⌘ K`** opens the command palette: type to search every request in the workspace and every command Mercury has. `↑`/`↓` move, `⏎` opens, `Esc` closes.
- **`⌘ D`** switches between the light and dark theme. Mercury follows your system theme until you press it.
- **`⌘ E`** steps through the environments in the picker, then None, then starts over.
- **`⌘ F`** opens the find bar over the response body: every match is highlighted, `⏎` steps to the next one, and `Esc` closes it.
- **`⌘ Shift F`** hides the sidebar. Press it again to bring the sidebar back.
- **`Tab`** moves between the controls on screen and `Space` or `⏎` activates the one you land on — the focus ring shows where you are. Everything Mercury can do is reachable without a mouse.
- **`Esc`** closes the find bar first, since that is what you are looking at when you press it. Otherwise it cancels a running request, and failing that clears the sidebar filter. A dialog or the palette closes before any of it.

## Related

- [Quick Start](/docs/quickstart)
- [Requests](/docs/features/requests)
