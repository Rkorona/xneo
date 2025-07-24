// src/main.rs

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::env; // 需要引入 env
use std::io::{self, Write};

// ... (mod声明和struct Cli不变) ...
mod config;
mod db;
mod shell;

use config::Config;
use db::Database;

#[derive(Parser, Debug)]
#[command(name = "xneo", version = "0.2.0", author = "Your Name")]
#[command(about = "A smarter cd command with memory and intelligence")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}


#[derive(Subcommand, Debug)]
enum Commands {
    /// [Internal] Adds a directory to the database
    Add { 
        path: String 
    },

    /// [Internal] Queries the database for directories
    Query { 
        keywords: Vec<String>,
        
        /// Show suggestions for similar paths
        #[arg(long)]
        suggest: bool,

        /// [Internal] Find a matching ancestor directory
        #[arg(long)]
        ancestor: bool, // <-- 新增 ancestor 标志
    },

    // ... (其他 Commands 枚举成员不变) ...
    /// Generates shell initialization script
    Init { 
        /// Shell type: fish, bash, zsh, powershell
        shell: String 
    },

    /// Manages bookmarks
    Bookmark {
        #[command(subcommand)]
        action: BookmarkAction,
    },

    /// Shows usage statistics
    Stats,

    /// Database maintenance
    Clean {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
}

// ... (BookmarkAction 和 ConfigAction 不变) ...
#[derive(Subcommand, Debug)]
enum BookmarkAction {
    /// Add a bookmark for current or specified directory
    Add { 
        name: String, 
        path: Option<String> 
    },
    /// Remove a bookmark
    Remove { name: String },
    /// List all bookmarks
    List,
    /// Get bookmark path (internal use)
    Get { name: String },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration file
    Edit,
    /// Reset to default configuration
    Reset,
    /// [Internal] Get a specific config value
    Get { key: String },
}


fn main() -> Result<()> {
    if env::var("RUST_BACKTRACE").is_err() {
        env::set_var("RUST_BACKTRACE", "0");
    }

    let cli = Cli::parse();
    let config = Config::load()?;
    let mut db = Database::new(config.clone())?;

    match cli.command {
        Some(Commands::Init { shell }) => handle_init(&shell)?,
        Some(Commands::Add { path }) => db.add(&path)?,
        
        // 更新 Query 的匹配
        Some(Commands::Query { keywords, suggest, ancestor }) => {
            if ancestor {
                // 如果是 ancestor 查询，调用新的专用函数
                handle_ancestor_query(&keywords)?;
            } else {
                // 否则，走原来的查询逻辑
                handle_query(&db, &keywords, suggest)?;
            }
        }
        
        Some(Commands::Bookmark { action }) => handle_bookmark(&mut db, action)?,
        Some(Commands::Stats) => handle_stats(&db)?,
        Some(Commands::Clean { yes }) => handle_clean(&mut db, yes)?,
        Some(Commands::Config { action }) => handle_config(&config, action)?,
        None => {
            if let Some(home) = dirs::home_dir() {
                print!("{}", home.display());
            }
        }
    }

    Ok(())
}

// 新增：处理父目录查询的函数
fn handle_ancestor_query(keywords: &[String]) -> Result<()> {
    // 父目录查询只接受单个词
    if keywords.len() != 1 {
        return Ok(());
    }
    let name_to_find = &keywords[0];

    let current_dir = env::current_dir()?;
    for ancestor in current_dir.ancestors() {
        if let Some(dir_name) = ancestor.file_name().and_then(|s| s.to_str()) {
            if dir_name == name_to_find {
                // 找到了！打印路径并成功退出
                print!("{}", ancestor.display());
                return Ok(());
            }
        }
    }

    // 如果循环结束还没找到，就什么也不打印，安静地退出
    // Shell 脚本会根据是否有输出来决定下一步做什么
    Ok(())
}


// ... (handle_init, handle_query, handle_bookmark 等函数保持不变) ...
fn handle_init(shell: &str) -> Result<()> {
    match shell {
        "fish" => print!("{}", shell::FISH_INIT_SCRIPT),
        "bash" => print!("{}", shell::BASH_INIT_SCRIPT),
        "zsh" => print!("{}", shell::ZSH_INIT_SCRIPT),
        _ => {
            eprintln!("{}: Unsupported shell: {}","Error".red().bold(), shell);
            eprintln!("Supported shells: fish, bash, zsh, powershell");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn handle_query(db: &Database, keywords: &[String], suggest: bool) -> Result<()> {
    if keywords.is_empty() {
        return Ok(());
    }

    // 优先检查书签
    let keyword = keywords.join(" ");
    if keywords.len() == 1 {
        if let Some(path) = db.get_bookmark(&keyword)? {
            println!("{}", path);
            return Ok(());
        }
    }

    let results = db.query(keywords)?;
    
    if suggest {
        // 为建议模式，只返回路径列表
        for entry in results.iter().take(10) {
            println!("{}", entry.path);
        }
    } else {
        // 正常查询模式
        if results.is_empty() {
            // 尝试提供建议
            if let Ok(suggestions) = db.query(&[keyword.chars().take(3).collect()]) {
                if !suggestions.is_empty() {
                    eprintln!("{}: No exact match found", "Info".yellow().bold());
                    eprintln!("Similar paths:");
                    for (i, entry) in suggestions.iter().take(3).enumerate() {
                        eprintln!("  {}) {}", i + 1, entry.path.bright_blue());
                    }
                    return Ok(());
                }
            }
        } else {
            for entry in results {
                println!("{}", entry.path);
            }
        }
    }
    
    Ok(())
}

fn handle_bookmark(db: &mut Database, action: BookmarkAction) -> Result<()> {
    match action {
        BookmarkAction::Add { name, path } => {
            let target_path = match path {
                Some(p) => shellexpand::tilde(&p).to_string(),
                None => env::current_dir()?.to_string_lossy().to_string(),
            };
            
            if !std::path::Path::new(&target_path).exists() {
                eprintln!("{}: Path does not exist: {}", "Error".red().bold(), target_path);
                std::process::exit(1);
            }
            
            db.add_bookmark(&name, &target_path)?;
            println!("{}: Bookmark '{}' created for {}", "Success".green().bold(), name.bright_yellow(), target_path.bright_blue());
        }
        BookmarkAction::Remove { name } => {
            if db.remove_bookmark(&name)? {
                println!("{}: Bookmark '{}' removed", "Success".green().bold(), name.bright_yellow());
            } else {
                eprintln!("{}: Bookmark '{}' not found", "Error".red().bold(), name.bright_yellow());
                std::process::exit(1);
            }
        }
        BookmarkAction::List => {
            let bookmarks = db.get_bookmarks()?;
            if bookmarks.is_empty() {
                println!("No bookmarks found.");
            } else {
                println!("{}", "Bookmarks:".bright_green().bold());
                for bookmark in bookmarks {
                    println!("  {} -> {}", bookmark.name.bright_yellow(), bookmark.path.bright_blue());
                }
            }
        }
        BookmarkAction::Get { name } => {
            if let Some(path) = db.get_bookmark(&name)? {
                print!("{}", path);
            }
        }
    }
    Ok(())
}

fn handle_stats(db: &Database) -> Result<()> {
    let stats = db.get_stats()?;
    
    println!("{}", "📊 xneo Statistics".bright_green().bold());
    println!("──────────────────────────────");
    println!("Total directories: {}", stats.total_entries.to_string().bright_cyan());
    println!("Total visits: {}", stats.total_visits.to_string().bright_cyan());
    
    if !stats.most_visited.is_empty() {
        println!("\n{}", "🔥 Most Visited:".bright_yellow().bold());
        for (i, entry) in stats.most_visited.iter().enumerate() {
            println!("  {}. {} ({} visits)", (i + 1).to_string().bright_white(), entry.path.bright_blue(), entry.visits.to_string().bright_green());
        }
    }
    
    if !stats.recently_visited.is_empty() {
        println!("\n{}", "⏰ Recently Visited:".bright_yellow().bold());
        for (i, entry) in stats.recently_visited.iter().enumerate() {
            let time_ago = format_time_ago(&entry.last_access);
            println!("  {}. {} ({})", (i + 1).to_string().bright_white(), entry.path.bright_blue(), time_ago.bright_green());
        }
    }
    
    let bookmarks = db.get_bookmarks()?;
    if !bookmarks.is_empty() {
        println!("\n{}", "🔖 Bookmarks:".bright_yellow().bold());
        for bookmark in bookmarks.iter().take(5) {
            println!("  {} -> {}", bookmark.name.bright_yellow(), bookmark.path.bright_blue());
        }
        if bookmarks.len() > 5 {
            println!("  ... and {} more", (bookmarks.len() - 5).to_string().bright_cyan());
        }
    }
    
    Ok(())
}

fn format_time_ago(datetime: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*datetime);
    
    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}

fn handle_clean(db: &mut Database, yes: bool) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    
    println!("{}", "🔍 Scanning for stale entries...".bright_blue());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}").unwrap());
    pb.set_message("Checking directories...");
    
    let stale_entries = db.find_stale()?;
    pb.finish_and_clear();
    
    if stale_entries.is_empty() {
        println!("{}: Database is clean. No stale entries found.", "✓".green().bold());
        return Ok(());
    }

    println!("{}: Found {} stale entries:", "⚠".yellow().bold(), stale_entries.len().to_string().bright_red());
    
    for (i, entry) in stale_entries.iter().enumerate() {
        if i < 10 {
            println!("  - {}", entry.bright_red());
        } else if i == 10 {
            println!("  ... and {} more", (stale_entries.len() - 10).to_string().bright_cyan());
            break;
        }
    }

    let mut confirmed = yes;
    if !confirmed {
        print!("\n{} [y/N] ", "Do you want to remove them?".bright_yellow());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        confirmed = input.trim().eq_ignore_ascii_case("y");
    }
    
    if confirmed {
        let pb = ProgressBar::new(stale_entries.len() as u64);
        pb.set_style(ProgressStyle::default_bar().template("{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").unwrap());
        pb.set_message("Cleaning...");
        
        let cleaned_count = db.purge(&stale_entries)?;
        pb.finish_with_message("Done!");
        
        println!("\n{}: Successfully removed {} stale entries.", "✓".green().bold(), cleaned_count.to_string().bright_green());
    } else {
        println!("\n{}: No changes were made.", "ℹ".blue().bold());
    }
    
    Ok(())
}


fn handle_config(config: &Config, action: Option<ConfigAction>) -> Result<()> {
    match action {
        Some(ConfigAction::Show) | None => {
            println!("{}", "🔧 xneo Configuration".bright_green().bold());
            println!("──────────────────────────────");
            println!("Max entries: {}", config.max_entries.to_string().bright_cyan());
            println!("Update threshold: {} hours", config.update_threshold_hours.to_string().bright_cyan());
            println!("Fuzzy matching: {}", 
                if config.enable_fuzzy_matching { "enabled".green() } else { "disabled".red() }
            );
            println!("Auto clean on startup: {}", 
                if config.auto_clean_on_startup { "enabled".green() } else { "disabled".red() }
            );
            println!("FZF options: {}", config.fzf_options.bright_blue());
            
            if !config.ignored_patterns.is_empty() {
                println!("\n{}", "🚫 Ignored patterns:".bright_yellow().bold());
                for pattern in &config.ignored_patterns {
                    println!("  - {}", pattern.bright_red());
                }
            }
        }
        Some(ConfigAction::Edit) => {
            let config_path = dirs::config_dir()
                .unwrap()
                .join("xneo")
                .join("config.json");
            
            println!("Opening config file: {}", config_path.display().to_string().bright_blue());
            
            let editors = ["code", "vim", "nano", "emacs", "notepad"];
            let mut opened = false;
            
            for editor in &editors {
                if let Ok(_) = std::process::Command::new(editor)
                    .arg(&config_path)
                    .spawn()
                {
                    opened = true;
                    break;
                }
            }
            
            if !opened {
                println!("{}: Could not find a suitable editor", "Error".red().bold());
                println!("Please edit the file manually: {}", config_path.display());
            }
        }
        Some(ConfigAction::Reset) => {
            let new_config = Config::default();
            new_config.save()?;
            println!("{}: Configuration reset to defaults", "✓".green().bold());
        }
        // 新增：处理 get 命令
        Some(ConfigAction::Get { key }) => {
            match key.as_str() {
                "fzf_options" => print!("{}", config.fzf_options),
                _ => {
                    eprintln!("{}: Unknown config key: {}", "Error".red().bold(), key);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}