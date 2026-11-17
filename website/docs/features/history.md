---
title: Request History
sidebar_label: History
sidebar_position: 5
---

# Request History

Every request you send is saved to history together with its response. A request that gets no response (a timeout, a refused connection, a bad URL) is saved too, marked **Failed**; opening it shows the error again, with **Retry**.

## Opening history

Press `⌘ H`, or click the clock icon at the top of the response panel (it's there in every state). The history list takes the place of the response panel. Press `⌘ H` again, or click ×, to close it.

Each entry shows:

- the method and URL
- when it was sent ("Just now", "5 min ago", "Yesterday", ...)
- the status code
- the response time
- the environment it was sent with

Newest entries are at the top. The **Search history...** box filters entries by URL.

## Restoring an entry

Click an entry to show the stored response. If it was sent from a request file that still exists, that file opens; otherwise the method, URL, headers and body are loaded as an unsaved request, which `⌘ S` saves. The response is labelled with the environment it came from, and with **from .env.dev** (say) once a different environment is selected. The list then closes.

Bodies of images and binary responses aren't stored in history. A restored binary response shows its type and size, but you can't save its contents.

## Storage and limits

| | |
|---|---|
| File | `~/.mercury/history.json` (one list for all workspaces) |
| Entries kept | The newest 50 |
| Response bodies | Kept up to 1 MB. A bigger body is shown in full when it arrives, but history keeps only its status, headers and timing: send it again to see the body |
| Age | Entries older than 7 days are dropped |
| Clearing | The trash icon in the history header clears it at once. The notification that follows has an **Undo** link for a few seconds |

If Mercury can't write `~/.mercury` (a read-only folder, a full disk), a notification says so. The list still works until you quit.

History stores full request headers and response bodies, including any tokens they contain, in plain text. The history panel says so under its search box. Clear it if that's a concern.

History is read from disk only when you first open the list or send a request, and only a summary of each entry is kept in memory. See [Performance](/docs/performance#lazy-loading).

To keep Mercury's data somewhere other than `~/.mercury/`, set the `MERCURY_HOME` environment variable before launching. See the [FAQ](/docs/faq#where-is-my-data-stored).

## Related

- [Requests](/docs/features/requests)
- [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts)
