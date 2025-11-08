use clap::{Parser, Subcommand};
use colored::*;
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use openai_api_rust::chat::*;
use openai_api_rust::embeddings::*;
use openai_api_rust::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tiktoken_rs::p50k_base;

const MODEL_NAME: &str = "gpt-4o";
const MAX_TOKENS: usize = 128000; // Adjust this based on the model's actual limit
const EMBEDDING_MODEL: &str = "text-embedding-3-small";
const EMBEDDING_DIMENSION: usize = 1536;

#[derive(Error, Debug)]
enum CommitauraError {
    #[error("Tokenizer error: {0}")]
    TokenizerError(String),
    #[error("No staged changes detected")]
    NoStagedChanges,
    #[error("Git operation failed: {0}")]
    GitOperationFailed(String),
    #[error("API request failed: {0}")]
    ApiRequestFailed(String),
    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),
    #[error("OpenAI API error: {0}")]
    OpenAIError(String),
    #[error("Template error: {0}")]
    TemplateError(#[from] indicatif::style::TemplateError),
    #[error("Dialoguer error: {0}")]
    DialoguerError(#[from] dialoguer::Error),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

// Removed redundant implementation

#[derive(Parser)]
#[command(name = "Commitaura")]
#[command(about = "Intelligent Git Commit Assistant", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Automatically generate commit message and commit
    Commit,
    /// Generate comprehensive documentation from git history
    Story {
        /// Output file for the generated story (default: project_story.md)
        #[arg(short, long, default_value = "project_story.md")]
        output: String,
        /// Include detailed commit analysis
        #[arg(long)]
        detailed: bool,
    },
    /// Search git history by semantic meaning using AI embeddings
    Search {
        /// Natural language search query
        query: String,
        /// Number of results to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Rebuild embedding cache from scratch
        #[arg(long)]
        rebuild_cache: bool,
    },
}

fn main() -> Result<(), CommitauraError> {
    env_logger::init();
    dotenv::dotenv().ok();

    let auth = Auth::from_env()
        .map_err(|_| CommitauraError::EnvVarNotSet("OPENAI_API_KEY".to_string()))?;
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let cli = Cli::parse();
    let term = Term::stdout();

    match cli.command {
        Some(Commands::Commit) | None => handle_commit(&openai, &term)?,
        Some(Commands::Story { output, detailed }) => handle_story(&openai, &term, &output, detailed)?,
        Some(Commands::Search { query, limit, rebuild_cache }) => {
            handle_search(&openai, &term, &query, limit, rebuild_cache)?
        }
    }
    Ok(())
}

fn handle_commit(openai: &OpenAI, term: &Term) -> Result<(), CommitauraError> {
    term.clear_screen()?;
    println!("{} {}\n", "🚀".bold().cyan(), style("Commitaura: Commit Assistant").bold().white().on_black());
    println!("{}", "────────────────────────────────────────────".white());

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")?);
    pb.set_message("Checking for staged changes...");
    check_staged_changes()?;
    pb.set_message("Fetching recent commit messages...");
    let last_commits = get_last_commit_messages()?;
    pb.finish_and_clear();

    display_commit_messages(&last_commits);

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.magenta} {msg}")?);
    pb.set_message("Generating commit message with AI magic...");
    let commit_message = generate_commit_message(openai, &last_commits)?;
    pb.finish_and_clear();

    // Draw a box around the commit message for clarity and style    let _border = "┌".to_string() + &"─".repeat(48) + "┐";
    let _bottom = "└".to_string() + &"─".repeat(48) + "┘";
    println!("{}", "✨ Suggested Commit Message:".bold().green());
    println!("{}", "────────────────────────────────────────────".white());
    println!("{}", commit_message.bold().white());
    println!("{}", "────────────────────────────────────────────".white());

    // Allow user to edit/tweak the commit message (inline in terminal)
    println!("{}", "Edit the commit message below. Press Enter to accept, or type your own:");
    let subject: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Subject (short summary, <50 chars)")
        .default(commit_message.clone())
        .interact_text()?;
    let add_body = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add a longer description/body?")
        .default(false)
        .interact()?;
    let mut edited_message = subject.clone();
    if add_body {
        let body: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Body (optional, details)")
            .allow_empty(true)
            .interact_text()?;
        if !body.trim().is_empty() {
            edited_message = format!("{}\n\n{}", subject.trim(), body.trim());
        }
    }
    println!("{}", "────────────────────────────────────────────".white());
    println!("{}", "Final Commit Message Preview:".bold().cyan());
    println!("{}", edited_message.bold().white());
    println!("{}", "────────────────────────────────────────────".white());

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(style("Proceed with this commit message?").cyan().to_string())
        .default(true)
        .interact()? {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
        pb.set_message("Committing changes...");
        pb.enable_steady_tick(Duration::from_millis(80));
        perform_git_commit(&edited_message)?;
        pb.finish_with_message(style("✅ Commit successful!").bold().green().to_string());
    } else {
        println!("{}", style("Commit cancelled by user.").bold().yellow());
    }
    println!("\n{}", "Thank you for using Commitaura!".italic().white());
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

fn get_last_commit_messages() -> Result<String, CommitauraError> {
    let output = std::process::Command::new("git")
        .args(&["log", "-5", "--pretty=format:%s"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    String::from_utf8(output.stdout).map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))
}

fn generate_commit_message(openai: &OpenAI, last_commits: &str) -> Result<String, CommitauraError> {
    let diff_output = std::process::Command::new("git")
        .args(&["diff", "--staged"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let mut diff = String::from_utf8(diff_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    if diff.trim().is_empty() {
        return Err(CommitauraError::NoStagedChanges);
    }

    // Estimate tokens and truncate if necessary
    let system_message =
        "You are a helpful assistant that generates concise and meaningful Git commit messages.";
    let prompt = format!(
        "Write a highly specific, imperative Git commit message based only on the following changes. Do NOT use generic phrases like 'improved readability', 'aesthetic appeal', or 'refactored code'. Instead, reference concrete details such as filenames, functions, variables, or logic that was changed. Be precise about what was changed, how, and why. Do not include any other text except the commit message. Consider the context of the last 5 commit messages:\n\nLast 5 commit messages:\n{}\n\nCurrent changes:\n",
        last_commits
    );

    let system_tokens = estimate_tokens(system_message)?;
    let prompt_tokens = estimate_tokens(&prompt)?;
    let diff_tokens = estimate_tokens(&diff)?;
    let estimated_tokens = system_tokens + prompt_tokens + diff_tokens;

    if estimated_tokens > MAX_TOKENS {
        let available_tokens = MAX_TOKENS - system_tokens - prompt_tokens;
        let bpe = p50k_base().map_err(|e| CommitauraError::TokenizerError(e.to_string()))?;
        let tokens = bpe.encode_with_special_tokens(&diff);
        let truncated_tokens = tokens[..available_tokens].to_vec();
        diff = bpe
            .decode(truncated_tokens)
            .map_err(|e| CommitauraError::TokenizerError(e.to_string()))?;
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
                    "Write a concise and meaningful Git commit message based on the following changes (do not include any other text other than the commit message). Be extremely specific. Do not be vague. Consider the context of the last 5 commit messages:\n\nLast 5 commit messages:\n{}\n\nCurrent changes:\n{}",
                    last_commits, diff
                ),
            },
        ],
    };

    let rs = openai
        .chat_completion_create(&body)
        .map_err(|e| CommitauraError::OpenAIError(e.to_string()))?;

    let choice = rs.choices;
    let message = &choice[0]
        .message
        .as_ref()
        .ok_or(CommitauraError::ApiRequestFailed(
            "No message in API response".to_string(),
        ))?;
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

fn estimate_tokens(text: &str) -> Result<usize, CommitauraError> {
    let bpe = p50k_base().map_err(|e| CommitauraError::TokenizerError(e.to_string()))?;
    let tokens = bpe.encode_with_special_tokens(text);
    Ok(tokens.len())
}

fn display_commit_messages(commits: &str) {
    println!("{} {}", "📜".bold().blue(), "Recent Commit Messages:".bold().white());
    println!("{}", "────────────────────────────────────────────".white());
    for (i, message) in commits.lines().enumerate() {
        println!(
            "{} {}",
            format!("{}.", i + 1).yellow().bold(),
            message.white().italic()
        );
    }
    println!("{}\n", "────────────────────────────────────────────".white());
}

fn handle_story(openai: &OpenAI, term: &Term, output_file: &str, detailed: bool) -> Result<(), CommitauraError> {
    term.clear_screen()?;
    println!("{} {}\n", "📚".bold().cyan(), style("Commitaura: Project Story Generator").bold().white().on_black());
    println!("{}", "────────────────────────────────────────────".white());

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")?);

    // Fetch complete git history
    pb.set_message("Analyzing git repository...");
    let repo_info = get_repository_info()?;

    pb.set_message("Fetching complete git history...");
    let commit_history = get_complete_git_history(detailed)?;

    pb.set_message("Analyzing project structure...");
    let project_structure = analyze_project_structure()?;

    pb.set_message("Generating comprehensive project story...");
    let story = generate_project_story(openai, &repo_info, &commit_history, &project_structure, detailed)?;

    pb.finish_and_clear();

    // Save the story to file
    fs::write(output_file, &story)
        .map_err(|e| CommitauraError::IoError(e))?;

    println!("{}", "✅ Project Story Generated Successfully!".bold().green());
    println!("{}", "────────────────────────────────────────────".white());
    println!("{} {}", "📄".bold().blue(), format!("Story saved to: {}", output_file).bold().white());
    println!("{} {} characters", "📊".bold().yellow(), story.len().to_string().bold().white());
    println!("{}", "────────────────────────────────────────────".white());

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to preview the story?")
        .default(false)
        .interact()? {

        // Show a preview of the story
        let preview = if story.len() > 2000 {
            format!("{}...\n\n[Story truncated for preview - see {} for full content]",
                    &story[..2000], output_file)
        } else {
            story.clone()
        };

        println!("\n{}", "Story Preview:".bold().cyan());
        println!("{}", "════════════════════════════════════════════".white());
        println!("{}", preview);
        println!("{}", "════════════════════════════════════════════".white());
    }

    println!("\n{}", "Thank you for using Commitaura Story Generator!".italic().white());
    Ok(())
}

#[derive(Debug)]
struct RepositoryInfo {
    name: String,
    total_commits: usize,
    first_commit_date: String,
    last_commit_date: String,
    contributors: Vec<String>,
    branch_count: usize,
}

#[derive(Debug)]
struct CommitAnalysis {
    commits: Vec<CommitInfo>,
    file_changes: std::collections::HashMap<String, usize>,
    commit_patterns: Vec<String>,
    development_phases: Vec<String>,
}

#[derive(Debug)]
struct CommitInfo {
    hash: String,
    author: String,
    date: String,
    message: String,
    files_changed: usize,
    insertions: usize,
    deletions: usize,
}

fn get_repository_info() -> Result<RepositoryInfo, CommitauraError> {
    // Get repository name
    let repo_name = std::env::current_dir()
        .map_err(|e| CommitauraError::IoError(e))?
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Get total commit count
    let commit_count_output = std::process::Command::new("git")
        .args(&["rev-list", "--count", "HEAD"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let total_commits = String::from_utf8(commit_count_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?
        .trim()
        .parse::<usize>()
        .unwrap_or(0);

    // Get first commit date
    let first_commit_output = std::process::Command::new("git")
        .args(&["log", "--reverse", "--format=%ai", "-1"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let first_commit_date = String::from_utf8(first_commit_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?
        .trim()
        .to_string();

    // Get last commit date
    let last_commit_output = std::process::Command::new("git")
        .args(&["log", "--format=%ai", "-1"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let last_commit_date = String::from_utf8(last_commit_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?
        .trim()
        .to_string();

    // Get contributors
    let contributors_output = std::process::Command::new("git")
        .args(&["shortlog", "-sn", "--all"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let contributors = String::from_utf8(contributors_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?
        .lines()
        .map(|line| line.trim().splitn(2, '\t').nth(1).unwrap_or("").to_string())
        .filter(|name| !name.is_empty())
        .collect();

    // Get branch count
    let branch_output = std::process::Command::new("git")
        .args(&["branch", "-a"])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let branch_count = String::from_utf8(branch_output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?
        .lines()
        .count();

    Ok(RepositoryInfo {
        name: repo_name,
        total_commits,
        first_commit_date,
        last_commit_date,
        contributors,
        branch_count,
    })
}

fn get_complete_git_history(detailed: bool) -> Result<CommitAnalysis, CommitauraError> {
    let format_str = if detailed {
        "%H|%an|%ai|%s|"
    } else {
        "%H|%an|%ai|%s|"
    };

    let output = std::process::Command::new("git")
        .args(&["log", "--all", "--reverse", &format!("--format={}", format_str)])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let log_output = String::from_utf8(output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let mut commits = Vec::new();
    let file_changes = std::collections::HashMap::new();
    let mut commit_patterns = Vec::new();

    for line in log_output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            let hash = parts[0].to_string();
            let author = parts[1].to_string();
            let date = parts[2].to_string();
            let message = parts[3].to_string();

            // Get detailed stats for this commit if detailed mode
            let (files_changed, insertions, deletions) = if detailed {
                get_commit_stats(&hash)?
            } else {
                (0, 0, 0)
            };

            // Analyze commit patterns
            if message.starts_with("feat") || message.starts_with("add") {
                commit_patterns.push("Feature Development".to_string());
            } else if message.starts_with("fix") || message.starts_with("bug") {
                commit_patterns.push("Bug Fixes".to_string());
            } else if message.starts_with("refactor") || message.starts_with("clean") {
                commit_patterns.push("Code Refactoring".to_string());
            } else if message.starts_with("docs") || message.starts_with("doc") {
                commit_patterns.push("Documentation".to_string());
            } else if message.starts_with("test") {
                commit_patterns.push("Testing".to_string());
            } else {
                commit_patterns.push("General Development".to_string());
            }

            commits.push(CommitInfo {
                hash,
                author,
                date,
                message,
                files_changed,
                insertions,
                deletions,
            });
        }
    }

    // Analyze development phases
    let development_phases = analyze_development_phases(&commits);

    Ok(CommitAnalysis {
        commits,
        file_changes,
        commit_patterns,
        development_phases,
    })
}

fn get_commit_stats(hash: &str) -> Result<(usize, usize, usize), CommitauraError> {
    let output = std::process::Command::new("git")
        .args(&["show", "--stat", "--format=", hash])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let stats_output = String::from_utf8(output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let mut files_changed = 0;
    let mut insertions = 0;
    let mut deletions = 0;

    for line in stats_output.lines() {
        if line.contains("files changed") || line.contains("file changed") {
            // Parse the summary line
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "file" || *part == "files" {
                    if i > 0 {
                        files_changed = parts[i-1].parse().unwrap_or(0);
                    }
                } else if *part == "insertions(+)" {
                    if i > 0 {
                        insertions = parts[i-1].parse().unwrap_or(0);
                    }
                } else if *part == "deletions(-)" {
                    if i > 0 {
                        deletions = parts[i-1].parse().unwrap_or(0);
                    }
                }
            }
        }
    }

    Ok((files_changed, insertions, deletions))
}

fn analyze_development_phases(commits: &[CommitInfo]) -> Vec<String> {
    let mut phases = Vec::new();

    if commits.is_empty() {
        return phases;
    }

    // Simple phase detection based on commit patterns and timing
    let total_commits = commits.len();
    let early_phase = total_commits / 4;
    let mid_phase = total_commits / 2;
    let late_phase = (total_commits * 3) / 4;

    phases.push(format!("🌱 **Initial Development** (Commits 1-{})", early_phase));
    phases.push(format!("🚀 **Core Development** (Commits {}-{})", early_phase + 1, mid_phase));
    phases.push(format!("🔧 **Feature Expansion** (Commits {}-{})", mid_phase + 1, late_phase));
    phases.push(format!("✨ **Refinement & Polish** (Commits {}-{})", late_phase + 1, total_commits));

    phases
}

fn analyze_project_structure() -> Result<String, CommitauraError> {
    let output = std::process::Command::new("find")
        .args(&[".", "-type", "f", "-name", "*.rs", "-o", "-name", "*.toml", "-o", "-name", "*.md", "-o", "-name", "*.json"])
        .output()
        .or_else(|_| {
            // Fallback for Windows
            std::process::Command::new("powershell")
                .args(&["-Command", "Get-ChildItem -Recurse -Include *.rs,*.toml,*.md,*.json | Select-Object -ExpandProperty FullName"])
                .output()
        })
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let structure = String::from_utf8(output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    Ok(structure)
}

fn generate_project_story(
    openai: &OpenAI,
    repo_info: &RepositoryInfo,
    commit_analysis: &CommitAnalysis,
    project_structure: &str,
    detailed: bool,
) -> Result<String, CommitauraError> {
    let system_message = "You are an expert technical writer and software development historian. Your task is to analyze a complete git repository history and create a comprehensive, engaging narrative that tells the story of the project's development journey. Focus on identifying key phases, challenges overcome, technical achievements, and growth patterns. Write in a professional yet engaging tone that would be suitable for documentation, portfolio presentations, or technical retrospectives.";

    // Prepare commit summary
    let commit_summary = if detailed && commit_analysis.commits.len() > 50 {
        // For detailed mode with many commits, provide a structured summary
        format!(
            "Repository: {}\nTotal Commits: {}\nDevelopment Period: {} to {}\nContributors: {}\nBranches: {}\n\nCommit Patterns Analysis:\n{}\n\nDevelopment Phases:\n{}\n\nRecent Commits Sample:\n{}",
            repo_info.name,
            repo_info.total_commits,
            repo_info.first_commit_date,
            repo_info.last_commit_date,
            repo_info.contributors.join(", "),
            repo_info.branch_count,
            commit_analysis.commit_patterns.iter().collect::<std::collections::HashSet<_>>().into_iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
            commit_analysis.development_phases.join("\n"),
            commit_analysis.commits.iter().rev().take(20).map(|c| format!("{}: {}", c.date, c.message)).collect::<Vec<_>>().join("\n")
        )
    } else {
        // For regular mode or fewer commits, include all commits
        format!(
            "Repository: {}\nTotal Commits: {}\nDevelopment Period: {} to {}\nContributors: {}\nBranches: {}\n\nComplete Commit History:\n{}",
            repo_info.name,
            repo_info.total_commits,
            repo_info.first_commit_date,
            repo_info.last_commit_date,
            repo_info.contributors.join(", "),
            repo_info.branch_count,
            commit_analysis.commits.iter().map(|c| format!("{} [{}]: {}", c.date, c.author, c.message)).collect::<Vec<_>>().join("\n")
        )
    };

    let prompt = format!(
        "Analyze this git repository and create a comprehensive project story. Generate a well-structured markdown document that includes:\n\n1. **Executive Summary** - Brief overview of the project\n2. **Project Genesis** - How the project began\n3. **Development Journey** - Key phases and milestones\n4. **Technical Evolution** - Major technical decisions and changes\n5. **Challenges & Solutions** - Problems faced and how they were overcome\n6. **Key Achievements** - Notable accomplishments and breakthroughs\n7. **Growth Patterns** - Development velocity, learning curves\n8. **Current State** - Where the project stands today\n9. **Future Outlook** - Potential next steps based on trajectory\n\nRepository Data:\n{}\n\nProject Structure:\n{}\n\nMake the story engaging and insightful, identifying patterns in development style, periods of intense activity, architectural decisions, and the evolution of the codebase. Use specific commit messages and dates to support your narrative.",
        commit_summary,
        project_structure
    );

    // Estimate tokens and handle large histories
    let system_tokens = estimate_tokens(system_message)?;
    let prompt_tokens = estimate_tokens(&prompt)?;
    let estimated_tokens = system_tokens + prompt_tokens;

    let final_prompt = if estimated_tokens > MAX_TOKENS - 2000 {
        // Truncate if too large
        let available_tokens = MAX_TOKENS - system_tokens - 2000;
        let bpe = p50k_base().map_err(|e| CommitauraError::TokenizerError(e.to_string()))?;
        let tokens = bpe.encode_with_special_tokens(&prompt);
        let truncated_tokens = tokens[..available_tokens.min(tokens.len())].to_vec();
        bpe.decode(truncated_tokens)
            .map_err(|e| CommitauraError::TokenizerError(e.to_string()))?
    } else {
        prompt
    };

    let body = ChatBody {
        model: MODEL_NAME.to_string(),
        max_tokens: Some(4000),
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
                content: system_message.to_string(),
            },
            Message {
                role: Role::User,
                content: final_prompt,
            },
        ],
    };

    let rs = openai
        .chat_completion_create(&body)
        .map_err(|e| CommitauraError::OpenAIError(e.to_string()))?;

    let choice = rs.choices;
    let message = &choice[0]
        .message
        .as_ref()
        .ok_or(CommitauraError::ApiRequestFailed(
            "No message in API response".to_string(),
        ))?;

    let story = message.content.trim().to_string();

    if story.is_empty() {
        Err(CommitauraError::ApiRequestFailed(
            "Received empty story from LLM.".to_string(),
        ))
    } else {
        info!("Generated project story: {} characters", story.len());
        Ok(story)
    }
}

// ============================================================================
// SEMANTIC SEARCH FUNCTIONALITY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitEmbedding {
    hash: String,
    author: String,
    date: String,
    message: String,
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingCache {
    commits: Vec<CommitEmbedding>,
    version: String,
}

impl EmbeddingCache {
    fn new() -> Self {
        Self {
            commits: Vec::new(),
            version: "1.0".to_string(),
        }
    }

    fn cache_path() -> Result<PathBuf, CommitauraError> {
        let cache_dir = dirs::cache_dir()
            .ok_or(CommitauraError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find cache directory",
            )))?;

        let commitaura_cache = cache_dir.join("commitaura");
        fs::create_dir_all(&commitaura_cache)?;
        Ok(commitaura_cache.join("embeddings.bin"))
    }

    fn load() -> Result<Self, CommitauraError> {
        let path = Self::cache_path()?;
        if !path.exists() {
            return Ok(Self::new());
        }

        let data = fs::read(&path)?;
        bincode::deserialize(&data)
            .map_err(|e| CommitauraError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            ))
    }

    fn save(&self) -> Result<(), CommitauraError> {
        let path = Self::cache_path()?;
        let data = bincode::serialize(self)
            .map_err(|e| CommitauraError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            ))?;
        fs::write(&path, data)?;
        Ok(())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        0.0
    } else {
        dot_product / (magnitude_a * magnitude_b)
    }
}

fn generate_embedding(openai: &OpenAI, text: &str) -> Result<Vec<f32>, CommitauraError> {
    let body = EmbeddingsBody {
        model: EMBEDDING_MODEL.to_string(),
        input: vec![text.to_string()],
        user: None,
    };

    let result = openai
        .embeddings_create(&body)
        .map_err(|e| CommitauraError::OpenAIError(e.to_string()))?;

    let data = result.data.ok_or_else(|| {
        CommitauraError::ApiRequestFailed("No embedding data returned".to_string())
    })?;

    if data.is_empty() {
        return Err(CommitauraError::ApiRequestFailed(
            "Empty embedding data returned".to_string(),
        ));
    }

    let embedding = data[0].embedding.clone().ok_or_else(|| {
        CommitauraError::ApiRequestFailed("Embedding field is None".to_string())
    })?;

    // Convert from Vec<f64> to Vec<f32>
    Ok(embedding.iter().map(|&x| x as f32).collect())
}

fn get_all_commits_for_search() -> Result<Vec<CommitInfo>, CommitauraError> {
    let format_str = "%H|%an|%ai|%s";

    let output = std::process::Command::new("git")
        .args(&["log", "--all", &format!("--format={}", format_str)])
        .output()
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let log_output = String::from_utf8(output.stdout)
        .map_err(|e| CommitauraError::GitOperationFailed(e.to_string()))?;

    let mut commits = Vec::new();

    for line in log_output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            commits.push(CommitInfo {
                hash: parts[0].to_string(),
                author: parts[1].to_string(),
                date: parts[2].to_string(),
                message: parts[3].to_string(),
                files_changed: 0,
                insertions: 0,
                deletions: 0,
            });
        }
    }

    Ok(commits)
}

fn handle_search(
    openai: &OpenAI,
    term: &Term,
    query: &str,
    limit: usize,
    rebuild_cache: bool,
) -> Result<(), CommitauraError> {
    term.clear_screen()?;
    println!("{} {}\n", "🔍".bold().cyan(), style("Commitaura: Semantic Commit Search").bold().white().on_black());
    println!("{}", "────────────────────────────────────────────".white());
    println!("{} {}\n", "Query:".bold().blue(), query.italic().white());

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")?);

    // Load or rebuild cache
    pb.set_message("Loading embedding cache...");
    let mut cache = if rebuild_cache {
        EmbeddingCache::new()
    } else {
        EmbeddingCache::load()?
    };

    // Get all commits
    pb.set_message("Fetching commit history...");
    let all_commits = get_all_commits_for_search()?;

    // Build a map of existing embeddings by commit hash
    let mut existing_embeddings: HashMap<String, CommitEmbedding> = cache
        .commits
        .iter()
        .map(|ce| (ce.hash.clone(), ce.clone()))
        .collect();

    // Generate embeddings for new commits
    let mut new_embeddings_count = 0;
    for (idx, commit) in all_commits.iter().enumerate() {
        if !existing_embeddings.contains_key(&commit.hash) {
            pb.set_message(format!("Generating embeddings... ({}/{})", idx + 1, all_commits.len()));

            let text = format!("{}\n{}\n{}", commit.message, commit.author, commit.date);
            let embedding = generate_embedding(openai, &text)?;

            let commit_embedding = CommitEmbedding {
                hash: commit.hash.clone(),
                author: commit.author.clone(),
                date: commit.date.clone(),
                message: commit.message.clone(),
                embedding,
            };

            existing_embeddings.insert(commit.hash.clone(), commit_embedding);
            new_embeddings_count += 1;

            // Small delay to avoid rate limiting
            if new_embeddings_count % 10 == 0 {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    // Update cache with all embeddings
    cache.commits = existing_embeddings.values().cloned().collect();

    if new_embeddings_count > 0 {
        pb.set_message("Saving embedding cache...");
        cache.save()?;
    }

    // Generate embedding for the query
    pb.set_message("Analyzing your query...");
    let query_embedding = generate_embedding(openai, query)?;

    // Calculate similarities and sort
    pb.set_message("Searching commits...");
    let mut results: Vec<(CommitEmbedding, f32)> = cache
        .commits
        .iter()
        .map(|commit| {
            let similarity = cosine_similarity(&query_embedding, &commit.embedding);
            (commit.clone(), similarity)
        })
        .collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    pb.finish_and_clear();

    // Display results
    println!("{}", "Results:".bold().green());
    println!("{}", "────────────────────────────────────────────".white());

    let display_limit = limit.min(results.len());

    if results.is_empty() {
        println!("{}", "No commits found.".yellow());
    } else {
        for (i, (commit, score)) in results.iter().take(display_limit).enumerate() {
            let score_color = if *score > 0.8 {
                "green"
            } else if *score > 0.6 {
                "yellow"
            } else {
                "white"
            };

            println!("\n{} {} {}",
                format!("{}.", i + 1).bold().cyan(),
                format!("[Score: {:.3}]", score).color(score_color).bold(),
                format!("({})", &commit.hash[..8]).dimmed()
            );
            println!("   {} {}", "Message:".bold(), commit.message.white());
            println!("   {} {} {}", "Author:".dimmed(), commit.author.dimmed(),
                     format!("({})", commit.date).dimmed());
        }
    }

    println!("\n{}", "────────────────────────────────────────────".white());
    println!("{} {} commits searched, {} displayed",
        "📊".bold().blue(),
        cache.commits.len().to_string().bold().white(),
        display_limit.to_string().bold().white()
    );

    if new_embeddings_count > 0 {
        println!("{} {} new commits indexed",
            "✨".bold().green(),
            new_embeddings_count.to_string().bold().white()
        );
    }

    println!("\n{}", "Thank you for using Commitaura Search!".italic().white());
    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

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
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&c, &d)).abs() < 0.001);
    }
}
