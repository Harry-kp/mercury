---
title: Quick Start
sidebar_label: Quick Start
sidebar_position: 2
---

# Quick Start

> Get productive with Mercury in 5 minutes. This guide walks you through the essential workflow.

## Step 1: Open a Workspace

When you first launch Mercury, you'll see an empty state. Start by opening a folder where you want to store your API requests.

1. Click **Open Folder** (or press `⌘+O` on Mac, `Ctrl+O` on Windows/Linux)
2. Select or create a folder — this becomes your workspace
3. Mercury watches this folder for changes in real-time

![Opening a workspace - Replace with: Screenshot showing the folder picker dialog](/img/screenshots/placeholder.png)

:::tip Pro Tip
Any folder works! If you have an existing project, open that folder. Mercury detects all `.json` files automatically.
:::

## Step 2: Create Your First Request

Create a new request file:

1. Right-click in the sidebar → **New Request**
2. Or press `⌘+N` (Mac) / `Ctrl+N` (Windows/Linux)
3. Give it a name like `get-users`

Mercury creates a `.json` file that looks like this:

```json
{
  "method": "GET",
  "url": "https://api.example.com",
  "headers": {},
  "body": ""
}
```

## Step 3: Configure the Request

Edit the request in the center panel:

**URL Bar**: Enter the full URL with protocol
```
https://httpbin.org/get
```

**Method**: Click the method badge to change (GET, POST, PUT, PATCH, DELETE)

**Headers**: Add custom headers in the Headers tab
```
Accept: application/json
X-Custom-Header: my-value
```

**Body** (for POST/PUT/PATCH): Add JSON or text in the Body tab
```json
{
  "name": "Mercury",
  "type": "API Client"
}
```

![Configuring a request - Replace with: Screenshot showing URL bar, method selector, and headers tab](/img/screenshots/placeholder.png)

## Step 4: Send the Request

Press `⌘+Enter` (Mac) or `Ctrl+Enter` (Windows/Linux) to send.

You can also click the **Send** button in the URL bar.

The send button animates while the request is in progress.

## Step 5: View the Response

The response panel shows:

- **Status badge** — Color-coded (green for 2xx, red for 4xx/5xx)
- **Response time** — How long the request took
- **Body size** — Kilobytes received
- **Headers** — Click the Headers tab to see response headers
- **Body** — Syntax-highlighted JSON, XML, or HTML

![Response panel - Replace with: Screenshot showing response with status badge, timing, and formatted JSON](/img/screenshots/placeholder.png)

## Step 6: Use Environment Variables

Store secrets and base URLs in environment files:

1. Create `.env` file in your workspace root:
```bash
# .env
BASE_URL=https://api.example.com
API_KEY=your-secret-key
```

2. Use variables in your requests with `{{variable}}` syntax:
```json
{
  "method": "GET",
  "url": "{{BASE_URL}}/users",
  "headers": {
    "Authorization": "Bearer {{API_KEY}}"
  },
  "body": ""
}
```

Mercury shows variable indicators — green for defined, red for undefined.

![Environment variables - Replace with: Screenshot showing .env file and variable substitution in action](/img/screenshots/placeholder.png)

## Step 7: Organize with Folders

Keep your requests organized:

1. Right-click in sidebar → **New Folder**
2. Drag and drop requests into folders
3. Collapse/expand folders to focus on what you need

Your folder structure mirrors the file system:
```
my-api-project/
├── .env
├── users/
│   ├── get-users.json
│   └── create-user.json
└── products/
    └── list-products.json
```

## What's Next?

You now know the essentials! Explore more features:

| Feature | Description |
|---------|-------------|
| [Request History](/docs/features/history) | Rerun previous requests |
| [Authentication](/docs/features/auth) | Basic Auth, Bearer tokens |
| [Import Collections](/docs/features/import-export) | From Postman, Insomnia, or cURL |
| [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts) | Master the keyboard-first workflow |

:::tip Live File Sync
Edit `.json` files in VS Code or any editor — Mercury updates instantly. No import/export needed!
:::
