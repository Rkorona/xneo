 ---
# xneo

A smarter `cd` command with memory and intelligence.

`xneo` learns from your directory navigation patterns and provides instant, intelligent directory jumping using a **frecency** algorithm (frequency + recency). It acts as a powerful replacement for the standard `cd` command, allowing you to navigate your filesystem with minimal keystrokes.

## Features

-   **Smart Navigation**: Learns your habits and jumps to the most relevant directories based on frequency and recency.
-   **Instant Jumping**: Navigate to any deep-nested directory by matching parts of its name.
-   **Bookmarks**: Create short, memorable aliases for your most-used directories.
-   **Context-Aware**: Intelligently finds ancestor directories in your current path (e.g., jump from `~/project/src/api` to `~/project` with `x project`).
-   **Fuzzy Matching**: Finds directories even if you have typos in your query.
-   **Cross-Shell Support**: Works seamlessly with `fish`, `bash`, and `zsh`.
-   **Usage Statistics**: Get insights into your navigation patterns.
-   **Auto-Cleanup**: Automatically finds and purges stale, non-existent directory entries from its database.
-   **Customizable**: Fine-tune its behavior, from ignored directories to `fzf` integration options, via a simple JSON config.

## Quick Start

### 1. Installation

You'll need Rust and Cargo installed.

```bash
# Clone the repository
git clone https://github.com/Rkorona/xneo.git
cd xneo

# Install the binary
cargo install --path .
```

### 2. Shell Integration

`xneo` works by hooking into your shell. Add the following line to your shell's configuration file:

#### Fish

```fish
# Add to ~/.config/fish/config.fish
xneo init fish | source
```

#### Bash

```bash
# Add to ~/.bashrc
eval "$(xneo init bash)"
```

#### Zsh

```zsh
# Add to ~/.zshrc
eval "$(xneo init zsh)"
```

After adding the line, restart your shell or source the config file (e.g., `source ~/.bashrc`). This will define the `x` function and the `xb` alias for bookmarks.

## Usage

### Basic Navigation

The `x` command replaces `cd`. It automatically records every directory you visit.

```bash
# Jump to a directory by name
x my-project

# Jump to a nested directory instantly
x api/v2

# Go to your home directory
x

# It still works with direct paths
x /etc/nginx
x ../../
```

### Context-Aware Navigation

This is one of `xneo`'s most powerful features. It can find parent directories by name from your current location.

```bash
# You are in /home/user/work/project-alpha/src/components

# Jump up to the 'project-alpha' directory
x project-alpha
# -> Navigates to /home/user/work/project-alpha

# Jump up to 'work'
x work
# -> Navigates to /home/user/work
```

### Bookmarks

Use the `xb` alias to manage bookmarks for frequently accessed paths.

```bash
# Bookmark the current directory as 'dotfiles'
xb add dotfiles

# Bookmark a specific path
xb add server /var/www/my-app

# Jump to a bookmark
x dotfiles
# -> Navigates to the bookmarked path

# List all your bookmarks
xb list

# Remove a bookmark
xb remove server
```

### Interactive Selection with FZF

If your query matches multiple directories, `xneo` will automatically open an `fzf` menu for you to choose from.

```bash
# You have ~/dev/project-a and ~/work/project-b
x project

# fzf will open with:
# > ~/dev/project-a
#   ~/work/project-b
```

### Statistics & Maintenance

```bash
# View your navigation statistics
xneo stats

# Find and remove non-existent directories from the database
xneo clean

# Run cleanup without the confirmation prompt
xneo clean --yes
```

### Configuration

```bash
# View the current configuration
xneo config show

# Open the config file in your default editor
xneo config edit

# Reset the configuration to its default values
xneo config reset
```

## How It Works

`xneo` maintains a small SQLite database of the directories you visit, tracking frequency and recency.

The ranking formula: `rank = (ln(visits + 1) * 0.7) + (recency_score * 0.3)`.

This ensures frequently used and recently accessed directories appear first.

1.  **Recording**: A shell hook automatically calls `xneo add "$PWD"` every time your current directory changes, updating the database.
2.  **Ranking**: When you use `x`, it queries the database and ranks results using a **frecency** algorithm. The rank is a weighted score of:
    -   **Frequency**: How many times you've visited a directory.
    -   **Recency**: How recently you visited it (older entries have their score decay over time).
3.  **Querying**: The `x` function is smart and tries the following logic in order:
    1.  Is it a valid, direct path (e.g., `../`, `/tmp`)?
    2.  Is it a bookmark?
    3.  Is it an ancestor of the current directory?
    4.  If none of the above, perform a global search in the database using the frecency rank.

## Configuration

You can customize `xneo` by editing `~/.config/xneo/config.json`.

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

-   `max_entries`: Max number of directory records to keep in the database.
-   `ignored_patterns`: A list of **glob patterns**. Directories matching these patterns will never be added to the database.
-   `enable_fuzzy_matching`: Use fuzzy search for queries that don't have an exact match.
-   `auto_clean_on_startup`: If `true`, runs `xneo clean` automatically.
-   `fzf_options`: Pass custom command-line options to `fzf` to change its appearance or behavior.

---

## Acknowledgments

Inspired by amazing tools like [zoxide](https://github.com/ajeetdsouza/zoxide), [autojump](https://github.com/wting/autojump), and [z](https://github.com/rupa/z).

---
## Happy navigating!