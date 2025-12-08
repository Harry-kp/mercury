# Quick Start Guide

## Get Started in 3 Steps

### 1. Build Mercury

```bash
cargo build --release
```

Or use the quick launch script:

```bash
./run.sh
```

### 2. Try the Example Project

1. Launch Mercury:
   ```bash
   cargo run --release
   ```

2. Click "📁 Open Folder" in the top bar

3. Navigate to and select the `example-project` folder

4. Select ".env.dev" from the Environment dropdown

5. Click on "list.http" in the sidebar

6. Click "Send" or press Cmd+Enter

You should see a JSON response with posts from the JSONPlaceholder API!

### 3. Create Your Own Project

1. Create a new folder for your API project:
   ```bash
   mkdir my-api-project
   cd my-api-project
   ```

2. Create your first request file:
   ```bash
   cat > get-users.http << 'EOF'
   GET https://jsonplaceholder.typicode.com/users
   EOF
   ```

3. Create an environment file:
   ```bash
   cat > .env.local << 'EOF'
   api_url=https://jsonplaceholder.typicode.com
   api_key=your-key-here
   EOF
   ```

4. Open this folder in Mercury and start testing!

## File Format Reference

### Basic GET Request
```http
GET https://api.example.com/users
```

### With Headers
```http
GET https://api.example.com/users
Authorization: Bearer token123
Accept: application/json
```

### POST with Body
```http
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer token123

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

### Using Variables
```http
POST https://{{host}}/api/{{version}}/users
Authorization: Bearer {{token}}
Content-Type: application/json

{
  "name": "{{username}}",
  "email": "{{email}}"
}
```

Then in your `.env.dev`:
```
host=api.dev.example.com
version=v1
token=dev_token_abc123
username=Test User
email=test@example.com
```

## Tips & Tricks

### Organize with Folders
```
my-api/
  ├── auth/
  │   ├── login.http
  │   └── logout.http
  ├── users/
  │   ├── list.http
  │   ├── create.http
  │   └── update.http
  ├── posts/
  │   └── list.http
  ├── .env.dev
  └── .env.production
```

### Multiple Environments
Create different environment files:
- `.env.local` - Your local development
- `.env.dev` - Shared dev environment
- `.env.staging` - Staging environment
- `.env.production` - Production environment

Switch between them using the Environment dropdown.

### Use Git
Since everything is just text files, you can:
```bash
git init
git add .
git commit -m "Add API requests"
```

Share your requests with your team through Git!

### Common Headers
Instead of repeating headers, use variables:

`.env.dev`:
```
host=api.example.com
token=abc123
content_type=application/json
```

In your requests:
```http
POST https://{{host}}/users
Content-Type: {{content_type}}
Authorization: Bearer {{token}}

{"name": "John"}
```

## Keyboard Shortcuts

- `Cmd+N` (or `Ctrl+N`) - Create new request
- `Cmd+Enter` (or `Ctrl+Enter`) - Send request
- `Cmd+,` (or `Ctrl+,`) - Open settings

## Troubleshooting

### "Connection refused" error
- Check that the API server is running
- Verify the URL in your request
- Check your network connection

### Variables not substituting
- Make sure you've selected an environment from the dropdown
- Verify the variable name in the .env file matches exactly
- Variable names are case-sensitive

### Request not loading
- Ensure the file has a `.http` extension
- Check that the first line is a valid HTTP method + URL
- Verify the file is inside your opened project folder

## Next Steps

- Read the full [README.md](README.md) for more details
- Check out the [PRD.md](PRD.md) to understand the philosophy
- Start building your own API request collections!

---

Need help? Open an issue on GitHub!
