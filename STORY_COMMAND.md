# Commitaura Story Command

The `story` command is a powerful feature that analyzes your entire git history and generates comprehensive documentation about your project's development journey.

## Usage

```bash
# Generate a basic project story
commitaura story

# Generate a detailed story with commit statistics
commitaura story --detailed

# Specify a custom output file
commitaura story --output my_project_journey.md

# Combine options
commitaura story --detailed --output detailed_analysis.md
```

## What It Analyzes

The story command examines:

- **Complete Git History**: Every commit from the first to the most recent
- **Repository Metadata**: Contributors, branches, development timeline
- **Commit Patterns**: Feature development, bug fixes, refactoring, documentation
- **Development Phases**: Automatic detection of project phases based on activity
- **Project Structure**: File types and organization
- **Growth Patterns**: Development velocity and evolution over time

## Generated Documentation Sections

1. **Executive Summary** - Brief overview of the project
2. **Project Genesis** - How the project began and initial goals
3. **Development Journey** - Key phases, milestones, and evolution
4. **Technical Evolution** - Major technical decisions and architectural changes
5. **Challenges & Solutions** - Problems encountered and how they were resolved
6. **Key Achievements** - Notable accomplishments and breakthroughs
7. **Growth Patterns** - Development velocity, learning curves, and trends
8. **Current State** - Project status and maturity level
9. **Future Outlook** - Potential next steps and opportunities

## Command Options

- `--output` / `-o`: Specify the output file name (default: `project_story.md`)
- `--detailed`: Include detailed commit statistics and analysis
- `--help`: Show help information

## AI-Powered Analysis

The story command uses advanced AI to:
- Identify development patterns and themes
- Recognize technical achievements and milestones
- Understand the evolution of coding practices
- Detect periods of intense development or refactoring
- Provide insights into project growth and maturity

## Use Cases

- **Portfolio Documentation**: Create compelling project narratives for portfolios
- **Team Retrospectives**: Understand how projects evolved over time
- **Technical Documentation**: Generate comprehensive project histories
- **Project Handoffs**: Provide context to new team members
- **Personal Reflection**: Understand your own development journey

## Example Output

The generated markdown includes:
- Structured sections with clear headings
- Timeline analysis with specific dates and commits
- Technical insights and pattern recognition
- Professional formatting suitable for documentation

## Requirements

- Git repository with commit history
- OpenAI API key set in environment (`OPENAI_API_KEY`)
- Internet connection for AI analysis

## Tips

- Use `--detailed` for projects with many commits to get richer analysis
- The AI can analyze projects of any size, but very large histories may be summarized
- Generated stories are in markdown format, perfect for GitHub, documentation sites, or conversion to other formats
