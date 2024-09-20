use anthropic::client::ClientBuilder;
use anthropic::types::CompleteRequestBuilder;
use anthropic::{AI_PROMPT, HUMAN_PROMPT};
use clap::{Parser, Subcommand};
use colored::*;
use log::{error, info};
use std::{env, fs};
use thiserror::Error;

const MODEL_NAME: &str = "claude-instant-1";

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

    #[error("Anthropic API error: {0}")]
    AnthropicError(String),
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

#[tokio::main]
async fn main() -> Result<(), CommitauraError> {
    env_logger::init();
    dotenv::dotenv().ok();

    let api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| CommitauraError::EnvVarNotSet("ANTHROPIC_API_KEY".to_string()))?;

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
            match handle_commit(&api_key).await {
                Ok(msg) => println!("{}", format!("✅ Committed: {}", msg).green()),
                Err(e) => eprintln!("{}", format!("❌ Error: {}", e).red()),
            }
        }
        Commands::UpdateReadme => {
            println!("{}", "📚 Updating README...".cyan());
            match handle_update_readme(&api_key).await {
                Ok(msg) => println!("{}", format!("✅ README Updated: {}", msg).green()),
                Err(e) => eprintln!("{}", format!("❌ Error: {}", e).red()),
            }
        }
        Commands::CommitAndUpdate => {
            println!("{}", "🚀 Committing changes and updating README...".cyan());
            match handle_commit(&api_key).await {
                Ok(msg) => {
                    println!("{}", format!("✅ Committed: {}", msg).green());
                    match handle_update_readme(&api_key).await {
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

async fn handle_commit(api_key: &str) -> Result<String, CommitauraError> {
    check_staged_changes()?;
    let commit_message = generate_commit_message(api_key).await?;
    perform_git_commit(&commit_message)?;
    Ok(commit_message)
}

async fn handle_update_readme(api_key: &str) -> Result<String, CommitauraError> {
    check_staged_changes()?;
    let readme_update = generate_readme_update(api_key).await?;
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

async fn generate_commit_message(api_key: &str) -> Result<String, CommitauraError> {
    let client = ClientBuilder::default()
        .api_key(api_key.to_string())
        .build()
        .map_err(|e| CommitauraError::AnthropicError(e.to_string()))?;

    let diff_output = std::process::Command::new("git")
        .args(&["diff", "--staged"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let diff = String::from_utf8(diff_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    if diff.trim().is_empty() {
        return Err(CommitauraError::NoStagedChanges);
    }

    let prompt = format!(
        "{HUMAN_PROMPT}Write a concise and meaningful Git commit message based on the following changes:\n{}\n{AI_PROMPT}",
        diff
    );

    let complete_request = CompleteRequestBuilder::default()
        .prompt(prompt)
        .model(MODEL_NAME.to_string())
        .max_tokens_to_sample(1000_usize) // Add this line
        .stream(false)
        .stop_sequences(vec![HUMAN_PROMPT.to_string()])
        .build()
        .map_err(|e| CommitauraError::AnthropicError(e.to_string()))?;

    let complete_response = client
        .complete(complete_request)
        .await
        .map_err(|e| CommitauraError::AnthropicError(e.to_string()))?;

    let commit_message = complete_response.completion.trim().to_string();

    if commit_message.is_empty() {
        Err(CommitauraError::ApiRequestFailed(
            "Received empty commit message from LLM.".to_string(),
        ))
    } else {
        info!("Generated commit message: {}", commit_message);
        Ok(commit_message)
    }
}

async fn generate_readme_update(api_key: &str) -> Result<String, CommitauraError> {
    let client = ClientBuilder::default()
        .api_key(api_key.to_string())
        .build()
        .map_err(|e| CommitauraError::AnthropicError(e.to_string()))?;

    let diff_output = std::process::Command::new("git")
        .args(&["diff", "--staged", "--name-only"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let files_changed = String::from_utf8(diff_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    if files_changed.trim().is_empty() {
        return Err(CommitauraError::NoStagedChanges);
    }

    let prompt = format!(
        "{HUMAN_PROMPT}Based on the following changes in the repository:\n{}\nProvide suggestions to update the README.md to reflect these changes.\n{AI_PROMPT}",
        files_changed
    );

    let complete_request = CompleteRequestBuilder::default()
        .prompt(prompt)
        .model(MODEL_NAME.to_string())
        .max_tokens_to_sample(1000_usize) // Add this line
        .stream(false)
        .stop_sequences(vec![HUMAN_PROMPT.to_string()])
        .build()
        .map_err(|e| CommitauraError::AnthropicError(e.to_string()))?;

    let complete_response = client
        .complete(complete_request)
        .await
        .map_err(|e| CommitauraError::AnthropicError(e.to_string()))?;

    let readme_updates = complete_response.completion.trim().to_string();

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
