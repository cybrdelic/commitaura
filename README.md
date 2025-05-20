<!-- filepath: c:\Users\alexf\software-projects\commitaura\README.md -->

<p align="center">
  <img src="https://raw.githubusercontent.com/alexfigueroa-solutions/commitaura/main/assets/logo.png" alt="Commitaura Logo" width="120" />
</p>

<h1 align="center">Commitaura</h1>

<p align="center">
  <b>Intelligent, context-aware Git commit assistant powered by OpenAI and Rust</b>
</p>

<p align="center">
  <a href="https://github.com/alexfigueroa-solutions/commitaura/actions">
    <img src="https://github.com/alexfigueroa-solutions/commitaura/actions/workflows/ci.yml/badge.svg" alt="CI Status" />
  </a>
  <a href="https://crates.io/crates/commitaura">
    <img src="https://img.shields.io/crates/v/commitaura.svg" alt="Crates.io" />
  </a>
  <a href="https://github.com/alexfigueroa-solutions/commitaura/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" />
  </a>
  <a href="https://github.com/alexfigueroa-solutions/commitaura/stargazers">
    <img src="https://img.shields.io/github/stars/alexfigueroa-solutions/commitaura?style=social" alt="GitHub stars" />
  </a>
</p>

---

## 🚀 Overview

Commitaura is an intelligent Git commit assistant designed to streamline and enhance your commit workflow by leveraging the power of OpenAI's language models. It automatically generates concise, meaningful, and context-aware commit messages based on your staged changes and recent commit history. The tool is built in Rust for performance, reliability, and ease of integration into modern development environments.

---

## ✨ Why Commitaura?

Writing high-quality commit messages is a crucial part of software development. Good commit messages:
- Improve project maintainability
- Make code reviews easier
- Help future contributors understand the evolution of the codebase

However, developers often struggle to write clear, specific, and consistent commit messages, especially when under time pressure. Commitaura solves this by:
- Analyzing your staged changes
- Considering your recent commit history for context
- Using an advanced LLM (OpenAI GPT-4o) to generate a commit message that is specific, non-repetitive, and meaningful

This approach ensures that your commit history remains clean, informative, and professional, with minimal manual effort.

---

## 🛠️ How It Works

1. **Check for Staged Changes**: Commitaura first checks if you have any staged changes. If not, it will prompt you to stage your changes before proceeding.
2. **Fetch Recent Commits**: It retrieves the last five commit messages to provide context to the LLM, helping it avoid repetition and maintain consistency.
3. **Generate Commit Message**: The tool sends your staged diff and recent commit messages to OpenAI's API, requesting a concise and meaningful commit message.
4. **User Confirmation**: The generated message is displayed, and you are prompted to confirm or cancel the commit.
5. **Perform Commit**: If confirmed, Commitaura commits your changes with the generated message.

---

## 🎯 Features

- **Context-Aware**: Considers both your current changes and recent commit history.
- **Token Management**: Automatically truncates input to fit within the LLM's token limits.
- **Interactive**: Asks for confirmation before committing.
- **Modern CLI**: Uses colorful, user-friendly terminal output.
- **Error Handling**: Provides clear error messages for common issues (e.g., no staged changes, API errors).

---

## ⚡ Installation

### Install via Cargo

You can install Commitaura directly from crates.io:

```sh
cargo install commitaura
```

### Build from Source

1. **Clone the Repository**
```sh
git clone https://github.com/alexfigueroa-solutions/commitaura.git
cd commitaura
```

2. **Set Up OpenAI API Key**
Create a `.env` file in the project root with your OpenAI API key:
```
OPENAI_API_KEY=sk-...
```

3. **Build the Project**
```sh
cargo build --release
```

---

## 📝 Usage

1. **Stage your changes** as usual with `git add ...`.
2. **Run Commitaura**:
   - If installed via Cargo:
     ```sh
     commitaura
     ```
   - If running from source:
     ```sh
     cargo run --release
     ```
3. **Review the generated commit message** and confirm to commit.

### Example Session

```shell
$ commitaura
Commit Changes

📜 Recent Commit Messages:
────────────────────────────────────────
1. Refactor user authentication logic
2. Fix typo in README
3. Add logging to payment service
4. Update dependencies
5. Initial commit
────────────────────────────────────────

Generated commit message:
Improve error handling in payment processing

Do you want to proceed with this commit message? [Y/n]:
```

---

## 🧩 Design and Implementation

Commitaura is written in Rust for its safety, speed, and excellent ecosystem. The project uses the following crates:
- `clap` for command-line argument parsing
- `console`, `colored`, and `indicatif` for rich terminal UI
- `dialoguer` for interactive prompts
- `openai_api_rust` for communicating with OpenAI's API
- `tiktoken-rs` for token counting and truncation
- `dotenv` and `env_logger` for environment and logging management
- `thiserror` for ergonomic error handling

### Token Management
OpenAI models have strict token limits. Commitaura estimates the number of tokens in your prompt and diff, truncating the diff if necessary to ensure the request fits within the model's constraints. This is handled using the `tiktoken-rs` crate, which provides accurate tokenization compatible with OpenAI models.

### Error Handling
All major operations are wrapped in robust error handling. Custom error types provide clear, actionable feedback for issues like missing API keys, no staged changes, or API failures.

### Extensibility
The codebase is modular and easy to extend. You can add new subcommands, integrate with other LLM providers, or customize the prompt for different commit message styles.

---

## 🔒 Security
- Your OpenAI API key is read from the environment and never logged or stored.
- No data is sent to third parties except OpenAI, and only the minimal required context (diff and recent commit messages) is transmitted.

---

## ⚠️ Limitations
- Requires an OpenAI API key and internet connection.
- Only works with staged changes (does not auto-stage files).
- Generated messages should be reviewed for accuracy and appropriateness.

---

## 🤝 Contributing
Contributions are welcome! Please open issues or pull requests on GitHub. For major changes, open an issue first to discuss your ideas.

---

## ❓ FAQ / Troubleshooting

### Q: I get an error about the OpenAI API key not being set.
A: Make sure you have a `.env` file in your project root with `OPENAI_API_KEY=sk-...` set, or that the environment variable is set in your shell.

### Q: Commitaura says "No staged changes detected" but I have changes.
A: Make sure you have staged your changes with `git add ...` before running Commitaura. Only staged changes are considered.

### Q: The generated commit message is empty or not useful.
A: Try re-running Commitaura, or review your staged changes and recent commit history. If the problem persists, check your OpenAI API quota.

### Q: How do I use a different OpenAI model?
A: Currently, the model is set in the source code. You can change the `MODEL_NAME` constant in `src/main.rs` to another supported model.

### Q: How do I debug or get more logs?
A: Set the `RUST_LOG` environment variable to `debug` or `info` before running Commitaura for more verbose output.

---

## 🧪 Development & Testing

To run tests:
```sh
cargo test
```

To run with debug logging:
```sh
$env:RUST_LOG = "debug"  # PowerShell
cargo run --release
```

To lint and check formatting:
```sh
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 📬 Contact & Support

For questions, bug reports, or feature requests, please open an issue on GitHub or contact the author.

---

## 📄 License
MIT License. See [LICENSE](LICENSE) for details.

## 👤 Author
Commitaura is maintained by Alex Figueroa.

---

<p align="center"><b>Commitaura: Let AI handle your commit messages, so you can focus on building great software.</b></p>
