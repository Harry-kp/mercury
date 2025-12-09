# Contributing to Mercury

Thanks for your interest in contributing to Mercury! 

## Philosophy First

Mercury follows the minimalist approach to software:

1. **Say no by default** - Every feature is a liability
2. **Build half a product, not a half-assed product** - Do less, but do it well
3. **It's a problem when it's a problem** - Don't solve imaginary issues

Before contributing, ask yourself:
- Is this feature truly essential?
- Can this be solved outside the app (Git, text editor, shell)?
- Does this add complexity that 80% of users won't need?

## What We're Looking For

✅ **Welcome**
- Bug fixes
- Performance improvements
- Better error messages
- Documentation improvements
- Code cleanup and refactoring
- Accessibility improvements

🤔 **Maybe** (discuss first)
- UI/UX refinements
- Keyboard shortcut additions
- File format improvements
- Import/export features (that maintain simplicity)

❌ **Not Welcome**
- Cloud sync features
- User accounts/authentication
- Team collaboration features
- GraphQL/WebSocket/gRPC support
- Mock servers
- Code generation
- Test automation frameworks
- Analytics/tracking
- Any feature that requires a backend

## Development Setup

1. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone and Build**
   ```bash
   git clone <your-fork>
   cd mercury
   cargo build
   ```

3. **Run Tests**
   ```bash
   cargo test
   ```

4. **Run the App**
   ```bash
   cargo run
   ```

## Project Structure

```
mercury/
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # Main application UI
│   ├── http_parser.rs       # Parse .http files
│   ├── request_executor.rs  # Execute HTTP requests
│   └── env_parser.rs        # Parse .env files
├── example-project/         # Example API project
├── PRD.md                   # Product requirements
├── README.md                # Main documentation
└── QUICKSTART.md            # Getting started guide
```

## Code Style

- Follow standard Rust conventions (`cargo fmt`)
- Run `cargo clippy` before submitting
- Write tests for new functionality
- Keep functions small and focused
- Comment the "why", not the "what"

## Submitting Changes

1. **Fork the repository**

2. **Create a feature branch**
   ```bash
   git checkout -b fix-something
   ```

3. **Make your changes**
   - Write clear, concise commit messages
   - One logical change per commit
   - Add tests if applicable

4. **Test thoroughly**
   ```bash
   cargo test
   cargo build --release
   # Test the actual app
   ```

5. **Submit a Pull Request**
   - Describe what changed and why
   - Reference any related issues
   - Keep PRs focused on one thing

## Pull Request Template

```markdown
## What Changed
Brief description of the change

## Why
Explanation of why this change is needed

## Testing
How you tested this change

## Checklist
- [ ] Ran `cargo fmt`
- [ ] Ran `cargo clippy`
- [ ] Ran `cargo test`
- [ ] Tested the app manually
- [ ] Updated documentation if needed
```

## Design Decisions

When in doubt, prefer:
- **Simplicity** over features
- **Speed** over flexibility  
- **Files** over databases
- **Convention** over configuration
- **Explicitness** over magic
- **Manual** over automatic

## Getting Help

- Open an issue for bugs or feature discussions
- Check existing issues first
- Be clear and specific
- Include reproduction steps for bugs

## Code of Conduct

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Remember**: The best code is no code. The best feature is no feature. Can we solve this without adding it?
