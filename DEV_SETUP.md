# Developer Setup Guide for Commitaura

Welcome to the Commitaura development environment! This guide will help you set up, test, and contribute to the project across different platforms and environments.

---

## 1. Prerequisites

- **Rust**: Install from https://rustup.rs/
- **Git**: Required for version control and some runtime features
- **Docker**: (Optional, for containerized builds/tests) https://www.docker.com/products/docker-desktop

---

## 2. Building and Testing Locally

Open your terminal (PowerShell on Windows, bash/zsh on Linux/macOS) and run:

```sh
cargo build --release
cargo test --all-targets --all-features
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

- `cargo build --release`: Builds the project in release mode
- `cargo test`: Runs all tests
- `cargo fmt -- --check`: Checks code formatting
- `cargo clippy`: Lints the code and fails on warnings

---

## 3. Setting the OpenAI API Key

### Using a .env File (Recommended for Local Dev)
Create a `.env` file in the project root:
```
OPENAI_API_KEY=sk-...
```

### Setting Globally (for all terminals)
- **Windows PowerShell:**
  ```powershell
  [Environment]::SetEnvironmentVariable("OPENAI_API_KEY", "sk-...", "User")
  ```
- **Linux/macOS (bash/zsh):**
  ```sh
  echo 'export OPENAI_API_KEY=sk-...' >> ~/.bashrc
  # or for zsh
  echo 'export OPENAI_API_KEY=sk-...' >> ~/.zshrc
  source ~/.bashrc  # or source ~/.zshrc
  ```

---

## 4. Docker Development & Testing

You can build and test Commitaura in a containerized environment:

```sh
docker build -t commitaura-test .
docker run --rm commitaura-test --help
```

- This builds the Docker image and runs the CLI help command inside the container.
- To open a shell in the container for further testing:
  ```sh
  docker run -it --rm commitaura-test /bin/bash
  ```

---

## 5. CI/CD and Multi-Platform Testing

Commitaura uses GitHub Actions for CI/CD. The workflows:
- Test on Ubuntu, Windows, and macOS (build, test, lint, format)
- Build and run the Docker image to verify container compatibility

You can find the workflow files in `.github/workflows/`.

---

## 6. Troubleshooting

- **API Key Issues:**
  - Make sure the key is set in your environment or .env file.
  - Remove old keys from your system/user environment variables if needed.
- **Build Failures:**
  - Ensure you have the latest stable Rust toolchain (`rustup update`)
  - Check for missing dependencies (e.g., Git, Docker)
- **Windows-specific:**
  - Run PowerShell as Administrator if you need to set environment variables globally.

---

## 7. Contributing

- Fork the repo and create a feature branch
- Make your changes and add tests if needed
- Run all checks locally (see above)
- Open a pull request on GitHub

---

## 8. Useful Commands

- Run the CLI after staging changes:
  ```sh
  commitaura
  # or, if running from source
  cargo run --release
  ```
- Run with debug logging:
  ```sh
  $env:RUST_LOG = "debug"  # PowerShell
  cargo run --release
  ```

---

For more, see the main README.md or open an issue for help!
