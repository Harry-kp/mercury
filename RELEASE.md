# Mercury Release Guide

This guide explains how to manage versions and release new updates for Mercury.

## Concepts (The "Why")

1.  **Version (`Cargo.toml`)**: The source of truth for the code version (e.g., `0.1.0`).
2.  **Tag (`git tag`)**: A sticky note on a specific commit history that says "This code is version 0.1.0".
3.  **Release (GitHub)**: A downloadable page created from a Tag. Our CI/CD pipeline automatically builds the Mac/Windows/Linux apps whenever you push a tag starting with `v` (e.g., `v0.1.0`).
4.  **Changelog**: A record for humans to know what changed.

## Manual Process (The "Hard Way")
1.  Edit `Cargo.toml` to bump version.
2.  Edit `CHANGELOG.md` to move "Unreleased" changes to a new version header.
3.  `git commit -m "release v0.2.0"`
4.  `git tag v0.2.0`
5.  `git push origin master`
6.  `git push origin v0.2.0`

## Automated Process (The "Easy Way") 🚀

We have a script that does all of the above for you.

### Prerequisite
Ensure your git working directory is clean (commit your code changes first).

### Command
Run the script from the project root:

```bash
./scripts/release.sh
```

### What it does
1.  Asks you if this is a `patch` (bugfix), `minor` (feature), or `major` (breaking) change.
2.  Updates `Cargo.toml`.
3.  Updates `CHANGELOG.md` automatically.
4.  Creates the git commit and tag.
5.  Pushes to GitHub (after asking for confirmation).

### Post-Release
After the script pushes, go to [GitHub Actions](https://github.com/Harry-kp/mercury/actions) to watch the build. When it turns green, your release is live!
