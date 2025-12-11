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
- Compare different responses
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
| **Timestamp** | When the request was executed |
| **Status** | HTTP status code (color-coded) |
| **Duration** | Response time in milliseconds |
| **Size** | Response body size |

Click an entry to expand and view the full response.

## Restoring a Request

If you want to rerun a previous request:

1. Click on the history entry
2. Click **Restore** (or right-click → Restore)
3. The request panel updates with that request's details
4. Modify if needed, then send

:::tip Quick Rerun
Hold `⌘` (Mac) or `Ctrl` (Windows/Linux) and click a history entry to rerun immediately.
:::

## History Persistence

History is stored locally on your machine:
- History persists between sessions
- Each workspace has its own history
- History is not stored with your `.http` files (it's separate)

## Clearing History

To clear request history:
1. Right-click in the timeline
2. Select **Clear History**

:::warning
Clearing history cannot be undone. All past executions will be deleted.
:::

## Use Cases

### Compare API Changes

Run the same request multiple times to compare responses as you develop your API.

### Debug Flaky Endpoints

Review past responses to identify intermittent issues.

### Reproduce Issues

Restore a request that caused an error to investigate further.

## Related Features

- [Requests](/docs/features/requests) — Creating and editing requests
- [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts) — Shortcuts for history navigation
