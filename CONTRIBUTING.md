# Contributing to Autonomi

We love your input! We want to make contributing to Autonomi as easy and transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Improving documentation

## Contributing Documentation

Our documentation is hosted at [https://dirvine.github.io/autonomi/](https://dirvine.github.io/autonomi/) and is built using MkDocs with the Material theme.

### Setting Up Documentation Locally

1. Clone the repository:

```bash
git clone https://github.com/dirvine/autonomi.git
cd autonomi
```

2. Install documentation dependencies:

```bash
pip install mkdocs-material mkdocstrings mkdocstrings-python mkdocs-git-revision-date-localized-plugin
```

3. Run the documentation server locally:

```bash
mkdocs serve
```

4. Visit `http://127.0.0.1:8000` to see your changes.

### Documentation Structure

```
docs/
├── api/                    # API Reference
│   ├── nodejs/
│   ├── python/
│   └── rust/
├── guides/                 # User Guides
│   ├── local_network.md
│   ├── evm_integration.md
│   └── testing_guide.md
└── getting-started/        # Getting Started
    ├── installation.md
    └── quickstart.md
```

### Making Documentation Changes

1. Create a new branch:

```bash
git checkout -b docs/your-feature-name
```

2. Make your changes to the documentation files in the `docs/` directory.

3. Test your changes locally using `mkdocs serve`.

4. Commit your changes:

```bash
git add docs/
git commit -m "docs: describe your changes"
```

5. Push to your fork and submit a pull request.

## Development Process

1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the test suite passes.
5. Make sure your code lints.
6. Issue that pull request!

## Any contributions you make will be under the MIT Software License

In short, when you submit code changes, your submissions are understood to be under the same [MIT License](LICENSE) that covers the project. Feel free to contact the maintainers if that's a concern.

## Report bugs using GitHub's [issue tracker](https://github.com/dirvine/autonomi/issues)

We use GitHub issues to track public bugs. Report a bug by [opening a new issue](https://github.com/dirvine/autonomi/issues/new).

## Write bug reports with detail, background, and sample code

**Great Bug Reports** tend to have:

- A quick summary and/or background
- Steps to reproduce
  - Be specific!
  - Give sample code if you can.
- What you expected would happen
- What actually happens
- Notes (possibly including why you think this might be happening, or stuff you tried that didn't work)

## License

By contributing, you agree that your contributions will be licensed under its MIT License.
