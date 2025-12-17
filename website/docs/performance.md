---
title: Performance
sidebar_label: Performance
sidebar_position: 3
---

# Performance

Mercury is built for developers who hate waiting.

## The Numbers That Matter

| Metric | Mercury | Postman | Insomnia |
|--------|---------|---------|----------|
| **Time to first request** | \<300ms | 3-5 sec | 2-4 sec |
| **Frame rate** | 60fps | Variable | Variable |
| **Input latency** | \<16ms | 50-100ms | 30-50ms |
| **Scroll smoothness** | Native GPU | Janky | Okay |

## Why Mercury Feels Instant

### 1. Native GPU Rendering

Mercury renders directly to your graphics card at 60 frames per second. Every scroll, every hover, every animation runs at native speed—the same technology used in video games.

Electron apps (Postman, Insomnia) render through a web browser, which adds **2-4 frames of latency** to every interaction. That's 30-60ms of delay you feel on every click.

### 2. Zero JavaScript

Mercury has no JavaScript runtime, no garbage collector pauses, no event loop overhead. When you press `Cmd+Enter`, the request fires **that frame**.

### 3. Single Binary

No runtime to initialize. No Node.js to boot. Mercury goes from double-click to usable in under 300ms.

---

## "But what about memory?"

You might notice Mercury uses ~100MB of RAM. Here's why that's a feature, not a bug:

**That 100MB buys you:**
- 60fps native rendering
- \<16ms input latency
- GPU-accelerated scrolling
- Instant response to every interaction

**Compare to Electron apps:**
- Postman: 400-800MB for a sluggish web page
- Insomnia: 200-500MB and still feels slow

Mercury uses **1/4 to 1/8 the memory** of competitors while feeling **4x faster**.

### The Trade-off

We could use 50MB and feel like a web page, or use 100MB and feel like a native app. We chose native feel.

> *"Performance is about perception, not benchmarks."*

---

## Benchmarks

### Startup Time (Cold)

```
Mercury:    280ms  ████
Insomnia:  2400ms  ████████████████████████████████████
Postman:   4200ms  ██████████████████████████████████████████████████████████████
```

### Input Latency (Click to Response)

```
Mercury:     12ms  ██
Insomnia:    45ms  █████████
Postman:     78ms  ████████████████
```

### Scroll Smoothness

```
Mercury:    60fps (locked)
Insomnia:   45-60fps (variable)
Postman:    30-50fps (janky)
```

---

## Technical Details

Mercury achieves its performance through:

- **Rust** — Zero-cost abstractions, no GC
- **egui** — Immediate mode GPU-accelerated UI
- **glow** — OpenGL ES backend
- **mimalloc** — High-performance allocator

The architecture prioritizes **perceived performance** over synthetic benchmarks. Every design decision optimizes for "does it feel fast?" rather than "is the number small?"

### Smart Response Handling

Mercury intelligently adapts to response size:

- **Small responses (<100KB)** — Full syntax highlighting with color-coded JSON/XML/HTML
- **Large responses (>100KB)** — Plain text display to maintain 60fps

Syntax highlighting is character-intensive. By skipping it for large responses, Mercury stays responsive even when your API returns megabytes of data.

---

## FAQ

### Why not use Tauri for lower memory?

Tauri apps use ~50MB but feel noticeably slower due to WebView rendering latency. We tested both and chose the one that feels better to use.

### Will you optimize memory in the future?

We continuously optimize, but we won't sacrifice the native feel. If we find ways to reduce memory without adding latency, we'll ship it.

### How does Mercury compare to terminal tools like curl?

Terminal tools are faster (no UI), but Mercury gives you:
- Visual response formatting
- Request history
- Environment variables
- Collection management

...while still feeling nearly as snappy as the command line.
