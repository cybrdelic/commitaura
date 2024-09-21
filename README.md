# Commitaura

**An Intelligent Git Commit Assistant with README Integration**

Commitaura enhances your Git workflow by generating concise, meaningful commit messages and suggesting updates to your README based on staged changes. Leveraging OpenAI's GPT models, it provides context-aware suggestions that save time and improve documentation quality.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
  - [Prerequisites](#prerequisites)
  - [Using Cargo (Recommended)](#using-cargo-recommended)
  - [Building from Source](#building-from-source)
- [Usage](#usage)
  - [Commands](#commands)
  - [Examples](#examples)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

## Features

- **Automated Commit Messages**: Generates detailed commit messages based on your staged changes.
- **README Updates**: Suggests modifications to your README.md to reflect recent changes.
- **Interactive CLI**: Offers an intuitive command-line interface with prompts and selections.
- **Flexible Operations**: Choose to commit changes, update the README, or both simultaneously.

## Installation

### Prerequisites

- **Rust and Cargo**: Ensure you have Rust and Cargo installed. Install them via [rustup.rs](https://rustup.rs/).
- **OpenAI API Key**: You'll need a valid OpenAI API key to use Commitaura.

### Using Cargo (Recommended)

Install Commitaura directly using Cargo:

```bash
cargo install commitaura
```

*Note: If Commitaura is not yet published on [crates.io](https://crates.io/), you can install it from the Git repository:*

```bash
cargo install --git https://github.com/alexfigueroa-solutions/commitaura.git
```

This command compiles the project and installs the `commitaura` binary to Cargo's bin directory, which is typically included in your system's PATH (`~/.cargo/bin`).

### Building from Source

If you prefer to build Commitaura from the source code:

1. **Clone the Repository**

   ```bash
   git clone https://github.com/alexfigueroa-solutions/commitaura.git
   ```

2. **Navigate to the Project Directory**

   ```bash
   cd commitaura
   ```

3. **Build and Install the Project**

   Use Cargo to build and install the project:

   ```bash
   cargo install --path .
   ```

   This command compiles the project and installs the `commitaura` binary to Cargo's bin directory.

### Setting Up the OpenAI API Key

Commitaura requires an OpenAI API key to function. You can set it up using one of the following methods:

- **Environment Variable**

  Add the API key to your shell profile (e.g., `~/.bashrc`, `~/.zshrc`):

  ```bash
  export OPENAI_API_KEY=your-api-key-here
  ```

- **.env File**

  In your home directory or project root, create a `.env` file:

  ```bash
  echo "OPENAI_API_KEY=your-api-key-here" > ~/.commitaura.env
  ```

  Commitaura will automatically load the API key from this file.

## Usage

Commitaura provides several commands to streamline your Git operations.

### Commands

- `commit`: Automatically generate a commit message and commit staged changes.
- `update-readme`: Generate suggestions to update your README.md based on staged changes.
- `commit-and-update`: Perform a commit and update the README in one step.

### Examples

#### Commit Changes with an Auto-Generated Message

```bash
commitaura commit
```

- Analyzes staged changes.
- Generates a commit message using OpenAI's GPT model.
- Prompts for confirmation before committing.

#### Update README Based on Changes

```bash
commitaura update-readme
```

- Analyzes staged changes and the current README.md.
- Suggests updates to the README.
- Allows selection of specific updates to apply.

#### Commit and Update README Together

```bash
commitaura commit-and-update
```

- Combines the functionalities of `commit` and `update-readme`.

## Configuration

Commitaura relies on an OpenAI API key for generating messages.

- **Environment Variable**

  Add the API key to your shell profile:

  ```bash
  export OPENAI_API_KEY=your-api-key-here
  ```

- **.env File**

  Create a `.env` file in your home directory or project root:

  ```bash
  echo "OPENAI_API_KEY=your-api-key-here" > ~/.commitaura.env
  ```

  Commitaura will automatically load the API key from this file.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/alexfigueroa-solutions/commitaura).

## License

This project is licensed under the MIT License.

## Contact

Developed by Alex Figueroa.

- **Email**: [alexfigueroa.solutions@gmail.com](mailto:alexfigueroa.solutions@gmail.com)
