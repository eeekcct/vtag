# vtag

A CLI tool for creating and managing semantic version tags in Git repositories.

## Features

- 🏷️ Create semantic version tags (Patch/Minor/Major)
- 🔐 GPG-signed tags (`git tag -s`)
- 🚀 Push tags to remote (`git push origin <tag>`)
- 🔄 Fetch and check local branch is up-to-date (`git fetch origin`)
- 📦 Create GitHub releases with auto-generated release notes
- ✅ Validates working tree is clean before tagging

## Installation

Download pre-built binaries from [Releases](https://github.com/eeekcct/vtag/releases).

## Usage

```sh
# Interactive mode (select version bump)
vtag

# Specify tag manually
vtag v1.2.3

# Create tag and GitHub release
vtag --release
```

## Requirements

- Git repository on `main` branch
- Clean working tree
- GitHub token for releases (set `GITHUB_TOKEN` or `GH_TOKEN`, or use `gh auth login`)

## License

[MIT](./LICENSE)
