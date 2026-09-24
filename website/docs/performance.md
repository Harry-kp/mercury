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

Two constants in `src/http.rs` control how much of a response Mercury handles:

| Limit | Value | What happens |
|-------|-------|--------------|
| `MAX_RESPONSE_SIZE` | 10 MB | If the server's `Content-Length` is larger, the body isn't downloaded and the panel shows **Response Too Large**. |
| `MAX_INLINE_SIZE` | 100 KB | Text bodies larger than this aren't shown in the panel. You get a **Save** link so you can open the body in an editor. |

The inline limit exists because syntax highlighting builds thousands of text spans every frame, and a large highlighted body would drop the UI below 60 fps. For the same reason, a body that grows past 100 KB when pretty-printed is shown without highlighting.

Images and binary content (PDF, audio, video, archives, `application/octet-stream`) are never rendered inline. Use **Save** to write them to disk.

## Lazy loading

- **Folders.** At startup Mercury reads only the workspace root. It reads a subfolder the first time you expand it.
- **History.** History isn't read at startup. It is loaded the first time you open the history list or send a request, and only a summary of each entry is kept in memory (method, URL, status, duration). The full request and response are read from disk when you click an entry.
