---
title: Request History
sidebar_label: History
sidebar_position: 5
---

# Request History

> Mercury keeps a timeline of your executed requests. Quickly view past responses and rerun previous requests.

## What is Request History?

Every time you send a request, Mercury records:
- The request details (method, URL, headers, body)
- Timestamp of execution
- Response status, time, and size
- Response body and headers

This lets you:
- Review past API responses
- Review previous responses
- Restore and rerun previous requests
- Debug API behavior over time

## Viewing History

The history timeline appears in the **Response panel**.

1. Click the **Timeline** tab in the response panel
2. See a list of recent executions
3. Click any entry to view that response

![History timeline - Replace with: Screenshot showing timeline tab with list of past request executions](/img/screenshots/placeholder.png)

## Timeline Entry Details

Each timeline entry shows:

| Field | Description |
|-------|-------------|
| **Timestamp** | When the request was executed (relative time) |
| **Status** | HTTP status code (color-coded) |
| **Duration** | Response time in milliseconds |
| **Method** | HTTP Method (GET, POST, etc.) |

Click an entry to restore it to the request panel.

## Restoring a Request

To reuse a previous request:

1. Click on any history entry
2. The request panel updates with that request's details (URL, method, headers, body)
3. **The response panel immediately shows the stored response body and headers**
4. Modify if needed, then click **Send** to rerun

## History Persistence

Mercury automatically persists your request history:

- **Automatic saving** — History is saved to `~/.mercury/history.json` after each request
- **Survives restarts** — History is loaded when the app starts
- **7-day retention** — Entries older than 7 days are automatically removed
- **50 entry limit** — The most recent 50 entries are kept
- **Global storage** — History is shared across all workspaces

## Clearing History

To clear all history entries:
1. Open the History panel (click **History** or press `Cmd/Ctrl + H`)
2. Click the **Clear** button in the header

This permanently removes all history entries. The change is saved immediately.

:::tip Timestamps
Each history entry shows when it was executed using relative timestamps like "Just now", "5 min ago", "Yesterday", or "3 days ago".
:::

## Use Cases

### Track API Changes

Run the same request multiple times to see how the response evolves as you develop your API.

### Debug Flaky Endpoints

Review past responses to identify intermittent issues.

### Reproduce Issues

Restore a request that caused an error to investigate further.

## Related Features

- [Requests](/docs/features/requests) — Creating and editing requests
- [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts) — Shortcuts for history navigation
