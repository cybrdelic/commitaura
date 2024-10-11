# commitaura

commitaura is an intelligent Git commit assistant. It helps you generate high-quality commit messages using OpenAI's GPT-4o model. That's it. I kept it simple.

## Description

commitaura is a command-line tool written in Rust that leverages the OpenAI API to generate commit messages based on the changes you've made to your codebase. It analyzes the changes and the past 5 commits, then generates a clear and concise commit message using OpenAI's language model.

## Installation

You can install commitaura via Cargo, the Rust package manager:

```
cargo install commitaura
```

## Setting up the OpenAI API Key

commitaura requires an OpenAI API key to function. You can set it up by adding the API key to your shell profile (e.g., `~/.zshrc`):

```bash
echo "export OPENAI_API_KEY=your-api-key-here" >> ~/.zshrc
```

Replace `your-api-key-here` with your actual OpenAI API key. After adding the line to your `.zshrc` file, restart your terminal or run `source ~/.zshrc` to apply the changes.

Alternatively, you can create a `.env` file in your project's root directory with the following content:

```
OPENAI_API_KEY=your-api-key-here
```

commitaura will automatically load the API key from this file.

## Usage

After installation and setting up the OpenAI API key, you can run commitaura from the command line within your Git repository:

```
commitaura
```

commitaura will guide you through the process of generating a commit message and updating your README file.

## Features

- **Intelligent commit message generation**: commitaura uses the OpenAI API to generate clear and concise commit messages based on the changes you've made to your codebase.
- **Cross-platform**: commitaura is a command-line tool written in Rust, so it can be used on any platform that supports Rust.

## Contributing

Contributions to commitaura are welcome! If you find a bug or have a feature request, please open an issue on the GitHub repository. If you'd like to contribute code, please fork the repository and submit a pull request.

## License

commitaura is released under the [MIT License](https://opensource.org/licenses/MIT).
