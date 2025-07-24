
```markdown
# xneo

A smarter `cd` command with memory and intelligence.

xneo learns from your directory navigation patterns and provides instant, intelligent directory jumping with a frecency algorithm (frequency + recency).

## Features

- **Smart Navigation**: Learns your patterns and suggests the most relevant directories
- **Instant Jumping**: Jump to any directory by partial name matching
- **Bookmarks**: Save and quickly access your favorite directories
- **Context-Aware**: Finds ancestor directories in your current path
- **Fuzzy Matching**: Find directories even with typos
- **Cross-Shell**: Works with Fish, Bash, and Zsh
- **Statistics**: Track your navigation patterns
- **Auto-Cleanup**: Removes non-existent directories automatically

## Quick Start

### Installation

```bash
# Install from source (requires Rust)
git clone https://github.com/Rkorona/xneo.git
cd xneo
cargo install --path .

# Or download pre-built binary from releases
```

### Shell Integration

Choose your shell and add the integration:

#### Fish
```bash
xneo init fish >> ~/.config/fish/config.fish
```

#### Bash
```bash
xneo init bash >> ~/.bashrc
```

#### Zsh
```bash
xneo init zsh >> ~/.zshrc
```

Then restart your shell or source the config file.

## Usage

### Basic Navigation

```bash
# Jump to any directory containing "project"
x project

# Jump to nested directories instantly
x docs/api

# Navigate to parent directories by name
x src        # Jumps to nearest 'src' ancestor directory

# No arguments? Go home
x
```

### Bookmarks

```bash
# Bookmark current directory
xb add work

# Bookmark specific directory
xb add dotfiles ~/.config

# Jump to bookmark
x work

# List all bookmarks
xb list

# Remove bookmark
xb remove work
```

### Statistics & Maintenance

```bash
# View usage statistics
xneo stats

# Clean up stale entries
xneo clean

# View current configuration
xneo config show

# Edit configuration
xneo config edit
```

## Configuration

xneo stores its configuration in `~/.config/xneo/config.json`. You can customize:

```json
{
  "max_entries": 1000,
  "ignored_patterns": [
    "**/node_modules",
    "**/node_modules/**",
    "**/.git",
    "**/.git/**",
    "**/target",
    "**/target/**",
    "**/*.log",
    "**/*.tmp"
  ],
  "update_threshold_hours": 168,
  "enable_fuzzy_matching": true,
  "show_stats_on_query": false,
  "auto_clean_on_startup": false,
  "fzf_options": "--height=40% --reverse --border"
}
```

### Configuration Options

- **max_entries**: Maximum number of directories to remember.
- **ignored_patterns**: Directory patterns to ignore. **Supports glob patterns** (e.g., `**/target/**`, `*.log`).
- **update_threshold_hours**: Hours after which entries are considered for cleanup.
- **enable_fuzzy_matching**: Enable fuzzy matching for queries.
- **auto_clean_on_startup**: Automatically remove stale entries on startup.
- **fzf_options**: Custom options for the `fzf` selection menu.

## How It Works

xneo uses a **frecency algorithm** that combines:

- **Frequency**: How often you visit a directory
- **Recency**: How recently you visited it
- **Context**: Your current location and path structure

The ranking formula: `rank = (ln(visits + 1) * 0.7) + (recency_score * 0.3)`

This ensures frequently used and recently accessed directories appear first.

## Examples

### Smart Project Navigation
```bash
# Working on multiple projects
cd ~/code/awesome-project
cd ~/code/another-project/src
cd ~/documents/project-notes

# Later, from anywhere:
x awesome    # → ~/code/awesome-project
x src        # → ~/code/another-project/src  
x notes      # → ~/documents/project-notes
```

### Bookmark Workflows
```bash
# Set up project bookmarks
xb add backend ~/code/myapp/backend
xb add frontend ~/code/myapp/frontend  
xb add deploy ~/code/myapp/deploy

# Quick switching
x backend    # Jump to backend code
x frontend   # Jump to frontend code
x deploy     # Jump to deployment scripts
```

### Context-Aware Navigation
```bash
# In /home/user/projects/myapp/src/components
x myapp      # Jumps to /home/user/projects/myapp
x projects   # Jumps to /home/user/projects
x src        # Jumps to /home/user/projects/myapp/src
```

## Statistics

Track your navigation patterns with detailed statistics:

```bash
xneo stats
```

Shows:
- Total directories and visits
- Most frequently visited directories
- Recently accessed directories  
- Active bookmarks

## Advanced Usage

### Multiple Matches
When multiple directories match, xneo will:
1. Show interactive selection with `fzf` (if available)
2. Fallback to the highest-ranked match
3. Display suggestions for partial matches

### Cleaning Database
```bash
# Interactive cleanup
xneo clean

# Auto-cleanup without prompts
xneo clean --yes
```

### Custom FZF Options
```bash
# Edit config to customize fzf appearance
xneo config edit

# Example: Change fzf to use different theme
# "fzf_options": "--height=60% --reverse --border --color=dark"
```

### Development Setup

```bash
git clone https://github.com/Rkorona/xneo.git
cd xneo
cargo build
cargo test
```

### Adding Shell Support

To add support for a new shell:
1. Add init script to `src/shell.rs`
2. Update `handle_init()` in `src/main.rs`
3. Test the integration
4. Update documentation

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [autojump](https://github.com/wting/autojump), [z](https://github.com/rupa/z), and [zoxide](https://github.com/ajeetdsouza/zoxide)
- Uses the frecency algorithm concept from Mozilla Firefox's address bar
- Built with ❤️ in Rust

---

**Happy navigating!**
```
