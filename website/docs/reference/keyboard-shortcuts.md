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
| `⌘ ⇧ C` | Copy as cURL |
| `⌘ ⇧ F` | Focus mode |
| `?` | Keyboard shortcuts |
| `Esc` | Cancel request / close dialog / clear filter |

Press `?` (when you aren't typing in a field), click **Shortcuts** in the status bar, or pick **Keyboard shortcuts** from the `⋯` menu to see this list in the app.

## Notes

- **`⌘ N`** clears the editor for a new, unsaved request. It saves the current request first if it has a file.
- **`⌘ S`** on an unsaved request asks for a name and saves it to the selected collection (the workspace root if none is selected). See [Saving](/docs/features/requests#saving).
- **`⌘ K`** opens the command palette: type to search every request in the workspace and every command Mercury has. `↑`/`↓` move, `⏎` opens, `Esc` closes.
- **`⌘ D`** switches between the light and dark theme. Mercury follows your system theme until you press it.
- **`⌘ E`** steps through the environments in the picker, then None, then starts over.
- **`⌘ Shift F`** hides the sidebar. Press it again to bring the sidebar back.
- **`Esc`** cancels a running request. If no request is running, it clears the sidebar filter. If a dialog or the palette is open, it closes it.

## Related

- [Quick Start](/docs/quickstart)
- [Requests](/docs/features/requests)
