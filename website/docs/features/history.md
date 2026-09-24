---
title: Request History
sidebar_label: History
sidebar_position: 5
---

# Request History

Every request that gets a response is saved to history together with that response. Requests that fail (timeouts, connection errors) aren't recorded.

## Opening history

Press `⌘ H`, or click **History** in the response panel. The history list takes the place of the response panel. Press `⌘ H` again, or click ×, to close it.

Each entry shows:

- the method and URL
- when it was sent ("Just now", "5 min ago", "Yesterday", ...)
- the status code
- the response time

Newest entries are at the top. The **Search history...** box filters entries by URL.

## Restoring an entry

Click an entry to load its method, URL, headers and body into the editor and show the stored response. The list then closes. The restored request isn't linked to a file, so press `⌘ S` to save it as a new request.

Bodies of images and binary responses aren't stored in history. A restored binary response shows its type and size, but you can't save its contents.

## Storage and limits

| | |
|---|---|
| File | `~/.mercury/history.json` (one list for all workspaces) |
| Entries kept | The newest 50 |
| Age | Entries older than 7 days are dropped |
| Clearing | The trash icon in the history header deletes the file immediately, with no confirmation |

History stores full request headers and response bodies, including any tokens they contain, in plain text. Clear it if that's a concern.

History is read from disk only when you first open the list or send a request, and only a summary of each entry is kept in memory. See [Performance](/docs/performance#lazy-loading).

To keep Mercury's data somewhere other than `~/.mercury/`, set the `MERCURY_HOME` environment variable before launching. See the [FAQ](/docs/faq#where-is-my-data-stored).

## Related

- [Requests](/docs/features/requests)
- [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts)
