use clap::{Parser, Subcommand};
use colored::*;
use log::{error, info};
use openai_api_rust::chat::*;
use openai_api_rust::*;
use std::{env, fs};
use thiserror::Error;

const MODEL_NAME: &str = "gpt-3.5-turbo";

#[derive(Error, Debug)]
enum CommitauraError {
    #[error("No staged changes detected")]
    NoStagedChanges,

    #[error("Git operation failed: {0}")]
    GitOperationFailed(String),

    #[error("API request failed: {0}")]
    ApiRequestFailed(String),

    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("OpenAI API error: {0}")]
    OpenAIError(String),
}

#[derive(Parser)]
#[command(name = "Commitaura")]
#[command(about = "Intelligent Git Commit Assistant with README Integration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Automatically generate commit message and commit
    Commit,
    /// Update README based on changes
    UpdateReadme,
    /// Commit and update README
    CommitAndUpdate,
}

fn main() -> Result<(), CommitauraError> {
    env_logger::init();
    dotenv::dotenv().ok();

    let auth = Auth::from_env()
        .map_err(|_| CommitauraError::EnvVarNotSet("OPENAI_API_KEY".to_string()))?;
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let cli = Cli::parse();

    println!(
        "{}",
        "🌟 Welcome to Commitaura - Intelligent Git Commit Assistant 🌟"
            .yellow()
            .bold()
    );

    match cli.command {
        Commands::Commit => {
            println!("{}", "📝 Generating commit message...".cyan());
            match handle_commit(&openai) {
                Ok(msg) => println!("{}", format!("✅ Committed: {}", msg).green()),
                Err(e) => eprintln!("{}", format!("❌ Error: {}", e).red()),
            }
        }
        Commands::UpdateReadme => {
            println!("{}", "📚 Updating README...".cyan());
            match handle_update_readme(&openai) {
                Ok(msg) => println!("{}", format!("✅ README Updated: {}", msg).green()),
                Err(e) => eprintln!("{}", format!("❌ Error: {}", e).red()),
            }
        }
        Commands::CommitAndUpdate => {
            println!("{}", "🚀 Committing changes and updating README...".cyan());
            match handle_commit(&openai) {
                Ok(msg) => {
                    println!("{}", format!("✅ Committed: {}", msg).green());
                    match handle_update_readme(&openai) {
                        Ok(readme_msg) => {
                            println!("{}", format!("✅ README Updated: {}", readme_msg).green())
                        }
                        Err(e) => eprintln!("{}", format!("❌ Error updating README: {}", e).red()),
                    }
                }
                Err(e) => eprintln!("{}", format!("❌ Error committing: {}", e).red()),
            }
        }
    }

    println!("{}", "Thank you for using Commitaura! 👋".yellow());

    Ok(())
}

fn handle_commit(openai: &OpenAI) -> Result<String, CommitauraError> {
    check_staged_changes()?;
    let commit_message = generate_commit_message(openai)?;
    perform_git_commit(&commit_message)?;
    Ok(commit_message)
}

fn handle_update_readme(openai: &OpenAI) -> Result<String, CommitauraError> {
    check_staged_changes()?;
    let readme_update = generate_readme_update(openai)?;
    update_readme_file(&readme_update)?;
    Ok("README.md updated based on recent changes.".to_string())
}

fn check_staged_changes() -> Result<(), CommitauraError> {
    let output = std::process::Command::new("git")
        .args(&["diff", "--staged", "--quiet"])
        .status()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    if output.success() {
        Err(CommitauraError::NoStagedChanges)
    } else {
        Ok(())
    }
}

fn perform_git_commit(message: &str) -> Result<(), CommitauraError> {
    let status = std::process::Command::new("git")
        .args(&["commit", "-m", message])
        .status()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    if status.success() {
        Ok(())
    } else {
        Err(CommitauraError::GitOperationFailed(
            "Git commit failed".to_string(),
        ))
    }
}

fn generate_commit_message(openai: &OpenAI) -> Result<String, CommitauraError> {
    let diff_output = std::process::Command::new("git")
        .args(&["diff", "--staged"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let diff = String::from_utf8(diff_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    if diff.trim().is_empty() {
        return Err(CommitauraError::NoStagedChanges);
    }

    let body = ChatBody {
        model: MODEL_NAME.to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: Some(1.0),
        n: Some(1),
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: vec![
            Message { role: Role::System, content: "You are a helpful assistant that generates concise and meaningful Git commit messages.".to_string() },
            Message { role: Role::User, content: format!("Write a concise and meaningful Git commit message based on the following changes (do not include any other text other than the commit message):\n{}", diff) },
        ],
    };

    let rs = openai
        .chat_completion_create(&body)
        .map_err(|e| CommitauraError::OpenAIError(e.to_string()))?;

    let choice = rs.choices;
    let message = &choice[0].message.as_ref().ok_or_else(|| {
        CommitauraError::ApiRequestFailed("No message in API response".to_string())
    })?;
    let commit_message = message.content.trim().to_string();

    if commit_message.is_empty() {
        Err(CommitauraError::ApiRequestFailed(
            "Received empty commit message from LLM.".to_string(),
        ))
    } else {
        info!("Generated commit message: {}", commit_message);
        Ok(commit_message)
    }
}

fn generate_readme_update(openai: &OpenAI) -> Result<String, CommitauraError> {
    let diff_output = std::process::Command::new("git")
        .args(&["diff", "--staged", "--name-only"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let files_changed = String::from_utf8(diff_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    if files_changed.trim().is_empty() {
        return Err(CommitauraError::NoStagedChanges);
    }

    let body = ChatBody {
        model: MODEL_NAME.to_string(),
        max_tokens: Some(500),
        temperature: Some(0.7),
        top_p: Some(1.0),
        n: Some(1),
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: vec![
            Message { role: Role::System, content: "You are a helpful assistant that suggests updates for README files based on changes in a Git repository.".to_string() },
            Message { role: Role::User, content: format!("Based on the following changes in the repository:\n{}\nProvide suggestions to update the README.md to reflect these changes.", files_changed) },
        ],
    };

    let rs = openai
        .chat_completion_create(&body)
        .map_err(|e| CommitauraError::OpenAIError(e.to_string()))?;

    let choice = rs.choices;
    let message = &choice[0].message.as_ref().ok_or_else(|| {
        CommitauraError::ApiRequestFailed("No message in API response".to_string())
    })?;
    let readme_updates = message.content.trim().to_string();

    if readme_updates.is_empty() {
        Err(CommitauraError::ApiRequestFailed(
            "Received empty README update suggestions from LLM.".to_string(),
        ))
    } else {
        info!("Generated README updates: {}", readme_updates);
        Ok(readme_updates)
    }
}

fn update_readme_file(updates: &str) -> Result<(), CommitauraError> {
    let current_readme = fs::read_to_string("README.md").unwrap_or_default();
    let updated_readme = format!("{}\n\n{}", current_readme, updates);
    fs::write("README.md", updated_readme)?;
    Ok(())
}
