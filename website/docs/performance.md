---
title: Performance
sidebar_label: Performance
sidebar_position: 3
---

# Performance

Mercury aims to stay responsive, and it does this with a few design choices rather than tuning.

## Native rendering

- **Rust, single binary.** There is no bundled browser engine or JavaScript runtime to start.
- **egui, immediate mode.** The whole UI is redrawn each frame and rendered on the GPU through OpenGL (`glow`). No DOM or layout engine sits between the input and the pixels.
- **Background work stays off the UI thread.** Requests, file dialogs and imports run on background threads, so the UI keeps drawing while they run.

## Response size limits

One constant in `src/http.rs` limits how much of a response Mercury handles:

| Limit | Value | What happens |
|-------|-------|--------------|
| `MAX_RESPONSE_SIZE` | 10 MB | If the server's `Content-Length` is larger, the body isn't downloaded and the panel shows **Response Too Large**. |

Any text body under that limit is shown, syntax-coloured, however big it is. Only the lines on screen are laid out each frame, so scrolling a multi-megabyte body costs the same as scrolling a small one (a 6.6 MB JSON body costs about 0.1 ms per frame on an M-series Mac). The body is pretty-printed on the network thread when the response arrives, and **Find** searches it once per query, not once per frame.

Images and binary content (PDF, audio, video, archives, `application/octet-stream`) are never rendered inline. Use **Save** to write them to disk.

## Lazy loading

- **Folders.** At startup Mercury reads only the workspace root. It reads a subfolder the first time you expand it.
- **Command palette.** `⌘ K` lists request names without opening the files, and reads a request's method only when its row scrolls into view. Only the rows on screen are drawn, so it opens and filters at the same speed in a workspace of 2,000 requests as in one of 20.
- **History.** History isn't read at startup. It is loaded the first time you open the history list or send a request, and only a summary of each entry is kept in memory (method, URL, status, duration). The full request and response are read from disk when you click an entry.
