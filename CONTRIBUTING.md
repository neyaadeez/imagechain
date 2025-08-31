# Contributing to ImageChain

Thank you for your interest in contributing to ImageChain! We appreciate your time and effort in helping to improve this project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)
- [Feature Requests](#feature-requests)
- [License](#license)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Set up the development environment (see README.md for details)
4. Create a new branch for your changes
5. Make your changes
6. Run tests and ensure they pass
7. Submit a pull request

## Development Workflow

### Prerequisites

- Rust (latest stable)
- FFmpeg
- libtorch (for deep learning features)
- OpenSSL

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/imagechain.git
cd imagechain

# Build in debug mode
cargo build

# Build in release mode
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with detailed output
cargo test -- --nocapture

# Run a specific test
cargo test test_name -- --nocapture
```

## Code Style

We follow the official Rust style guidelines. Please run the following before committing:

```bash
cargo fmt
cargo clippy -- -D warnings
```

## Testing

We encourage test-driven development. Please add tests for new features and bug fixes.

- Unit tests should be in the same file as the code they test
- Integration tests go in the `tests/` directory
- Use `#[ignore]` for slow tests that shouldn't run in CI
- Document test dependencies and setup requirements

## Documentation

- Update documentation when adding new features or changing behavior
- Document public APIs with Rustdoc
- Keep the README up-to-date
- Add examples for new functionality

## Pull Request Process

1. Fork the repository and create your branch from `main`
2. If you've added code that should be tested, add tests
3. If you've changed APIs, update the documentation
4. Ensure the test suite passes
5. Make sure your code lints
6. Issue a pull request with a clear description of your changes

## Reporting Issues

When reporting issues, please include:

- A clear, descriptive title
- A description of the problem
- Steps to reproduce the issue
- Expected vs. actual behavior
- Any relevant logs or screenshots
- Your environment details (OS, Rust version, etc.)

## Feature Requests

We welcome feature requests! Please open an issue with:

- A clear description of the feature
- The problem it solves
- Any alternative solutions you've considered
- Additional context or examples

## License

By contributing to ImageChain, you agree that your contributions will be licensed under the [MIT License](LICENSE).
