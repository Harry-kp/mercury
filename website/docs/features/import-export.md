---
title: Import & Export
sidebar_label: Import & Export
sidebar_position: 6
---

# Import & Export

> Migrate from other tools easily. Import from Postman, Insomnia, or cURL, and export your requests as cURL commands.

## Import from Insomnia

Mercury can import Insomnia collections in JSON or YAML format.

### How to Import

1. In Insomnia, export your collection (File → Export Data)
2. Choose **JSON** or **YAML** format
3. In Mercury, click **Import** in the sidebar (or use the menu)
4. Select your exported file
5. Mercury creates:
   - `.json` files for each request
   - Folders matching your Insomnia groups
   - `.env` files for environments

![Import Insomnia - Replace with: Screenshot showing import dialog with Insomnia file selected](/img/screenshots/placeholder.png)

### What Gets Imported

| Insomnia Item | Mercury Equivalent |
|---------------|-------------------|
| Requests | `.json` files |
| Request Groups | Folders |
| Environments | `.env.{name}` files |
| Headers | Headers in `.json` file |
| Body | Body in `.json` file |
| Auth | Auth headers |

### File Structure After Import

If you import an Insomnia collection with:
- Request group "Users" containing "Get User" and "List Users"
- Request group "Products" containing "Create Product"
- Environment "Development"

Mercury creates:
```
your-workspace/
├── .env.development
├── users/
│   ├── get-user.json
│   └── list-users.json
└── products/
    └── create-product.json
```

## Import from Postman

Mercury can import Postman Collection v2.1 files (JSON format).

### How to Import

1. In Postman, click **Export** on your collection (or go to File → Export Collection)
2. Choose **Collection v2.1** format
3. Save as JSON file
4. In Mercury, click **Import Postman...** in the Open menu
5. Select your exported file
6. Mercury creates:
   - `.json` files for each request
   - Folders matching your Postman folder structure
   - `.env.{collection-name}` file for collection variables

### What Gets Imported

| Postman Item | Mercury Equivalent |
|--------------|-------------------|
| Requests | `.json` files |
| Folders | Directories |
| Collection Variables | `.env.{name}` file |
| Headers | Headers in `.json` file |
| Body (raw/JSON) | Body in `.json` file |
| Query Parameters | URL with query string |

### File Structure After Import

If you import a Postman collection named "My API" with:
- Folder "Auth" containing "Login" request
- Folder "Users" with subfolder "Admin" containing "List Admins"
- Top-level "Health Check" request
- Collection variables: `base_url`, `token`

Mercury creates:
```
your-workspace/
├── .env.my-api            # Collection variables
├── auth/
│   └── login.json
├── users/
│   └── admin/
│       └── list-admins.json
└── health-check.json
```

:::tip Variable Preservation
Postman variables like `{{base_url}}` are preserved in the `.json` files. Define them in your `.env` file to use them.
:::

## Import from cURL

Paste a cURL command to create a request.

### How to Import

1. Copy a cURL command from your terminal, browser DevTools, or API docs
2. In Mercury, press `⌘+V` (Mac) or `Ctrl+V` (Windows/Linux) in the request panel
3. Mercury parses and fills in the request details

### Example

This cURL command:
```bash
curl -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer token123" \
  -d '{"name": "John"}'
```

Becomes:
```http
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer token123

{"name": "John"}
```

### Supported cURL Flags

| Flag | Support |
|------|---------|
| `-X, --request` | ✅ Method |
| `-H, --header` | ✅ Headers |
| `-d, --data` | ✅ Body |
| `--data-raw` | ✅ Body |
| `--data-binary` | ✅ Body |
| `--json` | ✅ JSON body + Content-Type header |
| `-u, --user` | ✅ Basic authentication |
| `-A, --user-agent` | ✅ User-Agent header |
| `-b, --cookie` | ✅ Cookie header |
| `-I, --head` | ✅ HEAD request |
| `-G, --get` | ✅ Force GET with data |
| `-L, --location` | Ignored (Mercury handles redirects) |
| `-k, --insecure` | Ignored |
| `-s, --silent` | Ignored |
| `--compressed` | Ignored |

## Export as cURL

Convert any request to a cURL command for sharing or CLI use.

### How to Export

1. Open a request
2. Right-click → **Copy as cURL**
3. Or press `⌘+Shift+C` (Mac) / `Ctrl+Shift+C` (Windows/Linux)

The cURL command is copied to your clipboard.

### Example

This request:
```http
GET https://api.example.com/users
Authorization: Bearer {{API_TOKEN}}
Accept: application/json
```

Becomes (with variables substituted):
```bash
curl -X GET "https://api.example.com/users" \
  -H "Authorization: Bearer your-substituted-token" \
  -H "Accept: application/json"
```

:::tip Variable Substitution
When exporting, Mercury substitutes environment variables with their current values.
:::

## File-Based Portability

Since Mercury uses plain `.json` files, you can also:

### Share via Email/Slack

Just send the `.json` file — anyone with Mercury can open it.

### Commit to Git

```bash
git add requests/
git commit -m "Add user API endpoints"
git push
```

### Copy Between Projects

```bash
cp project-a/users/*.json project-b/users/
```

## Related Features

- [Collections](/docs/features/collections) — Organizing imported requests
- [Environments](/docs/features/environments) — Setting up imported environments
- [File Format](/docs/reference/file-format) — The `.json` file specification
