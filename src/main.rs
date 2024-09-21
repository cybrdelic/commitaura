use clap::{Parser, Subcommand};
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use openai_api_rust::chat::*;
use openai_api_rust::*;
use pulldown_cmark::{html, Event, Parser as CmarkParser};
use std::{fs, time::Duration};
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
    #[error("Template error: {0}")]
    TemplateError(#[from] indicatif::style::TemplateError),
    #[error("Dialoguer error: {0}")]
    DialoguerError(#[from] dialoguer::Error),
}

#[derive(Parser)]
#[command(name = "Commitaura")]
#[command(about = "Intelligent Git Commit Assistant with README Integration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
    let term = Term::stdout();

    term.clear_screen()?;
    println!("{}", style("Welcome to Commitaura").bold().cyan());
    println!("{}", style("Your Intelligent Git Assistant").italic());
    println!();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            let items = vec![
                "Commit",
                "Update README",
                "Commit and Update README",
                "Exit",
            ];
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("What would you like to do?")
                .items(&items)
                .default(0)
                .interact_on(&term)?;

            match selection {
                0 => Commands::Commit,
                1 => Commands::UpdateReadme,
                2 => Commands::CommitAndUpdate,
                _ => return Ok(()),
            }
        }
    };

    match command {
        Commands::Commit => handle_commit(&openai, &term)?,
        Commands::UpdateReadme => handle_update_readme(&openai, &term)?,
        Commands::CommitAndUpdate => {
            handle_commit(&openai, &term)?;
            handle_update_readme(&openai, &term)?;
        }
    }

    println!();
    println!(
        "{}",
        style("Thank you for using Commitaura!").green().bold()
    );
    Ok(())
}

fn handle_commit(openai: &OpenAI, term: &Term) -> Result<(), CommitauraError> {
    term.clear_screen()?;
    println!("{}", style("Commit Changes").bold().underlined());
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.blue} {msg}")?,
    );

    pb.set_message("Checking staged changes...");
    check_staged_changes()?;

    pb.set_message("Generating commit message...");
    let commit_message = generate_commit_message(openai)?;

    pb.finish_and_clear();

    println!("Generated commit message:");
    println!("{}", style(&commit_message).yellow());
    println!();

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to proceed with this commit message?")
        .default(true)
        .interact()?
    {
        pb.set_message("Committing changes...");
        pb.enable_steady_tick(Duration::from_millis(100));
        perform_git_commit(&commit_message)?;
        pb.finish_with_message("Commit successful!");
    } else {
        println!("{}", style("Commit cancelled.").red());
    }

    Ok(())
}

fn handle_update_readme(openai: &OpenAI, term: &Term) -> Result<(), CommitauraError> {
    term.clear_screen()?;
    println!("{}", style("Update README").bold().underlined());
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.blue} {msg}")?,
    );

    pb.set_message("Checking staged changes...");
    check_staged_changes()?;

    pb.set_message("Generating README updates...");
    let readme_updates = generate_readme_update(openai)?;

    pb.finish_and_clear();

    println!("Suggested README updates:");
    println!("{}", style(&readme_updates).yellow());
    println!();

    let update_options = vec!["Apply all updates", "Select updates to apply", "Cancel"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("How would you like to proceed?")
        .items(&update_options)
        .default(0)
        .interact_on(term)?;

    match selection {
        0 => update_readme_file(&readme_updates)?,
        1 => {
            let selected_updates = select_updates(&readme_updates, term)?;
            update_readme_file(&selected_updates)?;
        }
        _ => println!("{}", style("README update cancelled.").red()),
    }

    Ok(())
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
            Message {
                role: Role::System,
                content: "You are a helpful assistant that generates concise and meaningful Git commit messages.".to_string(),
            },
            Message {
                role: Role::User,
                content: format!(
                    "Write a concise and meaningful Git commit message based on the following changes (do not include any other text other than the commit message). Be extremely specific. do not be vague:\n{}",
                    diff
                ),
            },
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
        .args(&["diff", "--staged"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let diff = String::from_utf8(diff_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    if diff.trim().is_empty() {
        return Err(CommitauraError::NoStagedChanges);
    }

    let current_readme = fs::read_to_string("README.md").unwrap_or_default();

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
            Message {
                role: Role::System,
                content: "You are a helpful assistant that suggests updates for README files based on changes in a Git repository.".to_string(),
            },
            Message {
                role: Role::User,
                content: format!(
                    "Based on the following changes in the repository:\n{}\n\nAnd the current README content:\n{}\n\nProvide suggestions to update the README.md to reflect these changes. Return only the new or modified sections of the README, not the entire file. Format your response as a series of numbered update suggestions.",
                    diff, current_readme
                ),
            },
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

    // Parse the current README
    let current_events: Vec<Event> = CmarkParser::new(&current_readme).collect();

    // Parse the updates
    let update_events: Vec<Event> = CmarkParser::new(updates).collect();

    // Merge the events
    let merged_events = merge_markdown_events(current_events, update_events);

    // Convert the merged events back to markdown
    let mut merged_markdown = String::new();
    html::push_html(&mut merged_markdown, merged_events.into_iter());

    fs::write("README.md", merged_markdown)?;
    Ok(())
}
fn select_updates(updates: &str, term: &Term) -> Result<String, CommitauraError> {
    let update_lines: Vec<&str> = updates.lines().collect();
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the updates to apply")
        .items(&update_lines)
        .interact_on(term)?;

    let selected_updates: String = selections
        .into_iter()
        .map(|i| update_lines[i])
        .collect::<Vec<&str>>()
        .join("\n");

    Ok(selected_updates)
}

fn merge_markdown_events<'a>(
    mut current: Vec<Event<'a>>,
    updates: Vec<Event<'a>>,
) -> Vec<Event<'a>> {
    // This is a simplified merge function. You might need to implement a more sophisticated
    // merging logic based on your specific README structure and update patterns.
    current.extend(updates);
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_staged_changes() {
        // This test assumes that there are no staged changes in the test environment
        assert!(matches!(
            check_staged_changes(),
            Err(CommitauraError::NoStagedChanges)
        ));
    }

    #[test]
    fn test_generate_commit_message() {
        // Mock the OpenAI client and test the generate_commit_message function
        // This is a placeholder and should be implemented with proper mocking
    }

    #[test]
    fn test_generate_readme_update() {
        // Mock the OpenAI client and test the generate_readme_update function
        // This is a placeholder and should be implemented with proper mocking
    }

    #[test]
    fn test_update_readme_file() {
        // Test the update_readme_file function with a mock README and updates
        let mock_readme = "# Test README\n\nThis is a test.".to_string();
        let mock_updates = "## New Section\n\nThis is a new section.".to_string();

        // Write mock README to a temporary file
        let temp_dir = tempfile::tempdir().unwrap();
        let readme_path = temp_dir.path().join("README.md");
        fs::write(&readme_path, &mock_readme).unwrap();

        // Update the README
        update_readme_file(&mock_updates).unwrap();

        // Read the updated README
        let updated_readme = fs::read_to_string(&readme_path).unwrap();

        // Check if the update was applied
        assert!(updated_readme.contains("# Test README"));
        assert!(updated_readme.contains("This is a test."));
        assert!(updated_readme.contains("## New Section"));
        assert!(updated_readme.contains("This is a new section."));
    }
}
