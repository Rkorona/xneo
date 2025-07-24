

//! # xneo
//! 
//! A smarter `cd` command with memory and intelligence.
//! 
//! xneo learns from your directory navigation patterns and provides:
//! - Intelligent directory jumping with frecency algorithm
//! - Bookmark management
//! - Context-aware ancestor matching
//! - Fuzzy matching support
//! - Cross-shell compatibility (Fish, Bash, Zsh, PowerShell)
//! 
//! ## Quick Start
//! 
//! ```bash
//! # Install shell integration
//! xneo init fish >> ~/.config/fish/config.fish
//! 
//! # Use the enhanced cd command
//! x project    # Jump to any directory containing "project"
//! x src        # Smart jump to src directory
//! xb add work  # Bookmark current directory as "work"
//! x work       # Jump to work bookmark
//! ```

pub mod config;
pub mod db;
pub mod shell;

pub use config::Config;
pub use db::{Database, DirEntry, Bookmark, Stats};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.max_entries, 1000);
        assert!(config.enable_fuzzy_matching);
        assert!(!config.ignored_patterns.is_empty());
    }

    #[test]
    fn test_database_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config::default();
        
        // Create a temporary database
        std::env::set_var("XDG_DATA_HOME", temp_dir.path());
        let mut db = Database::new(config)?;
        
        // Test adding directories
        let test_path = "/tmp/test/project";
        db.add(test_path)?;
        
        // Test querying
        let results = db.query(&["project".to_string()])?;
        assert!(!results.is_empty());
        assert_eq!(results[0].path, test_path);
        
        // Test bookmarks
        db.add_bookmark("test", test_path)?;
        let bookmark_path = db.get_bookmark("test")?;
        assert_eq!(bookmark_path, Some(test_path.to_string()));
        
        // Test stats
        let stats = db.get_stats()?;
        assert!(stats.total_entries > 0);
        
        Ok(())
    }

    #[test]
    fn test_ignored_patterns() {
        let config = Config::default();
        
        // Should ignore common build/cache directories
        assert!(config.is_ignored("/project/node_modules/package"));
        assert!(config.is_ignored("/project/.git/hooks"));
        assert!(config.is_ignored("/rust/project/target/debug"));
        
        // Should not ignore regular directories
        assert!(!config.is_ignored("/home/user/projects"));
        assert!(!config.is_ignored("/usr/local/bin"));
    }

    #[test]
    fn test_shell_scripts_exist() {
        // Ensure all shell scripts are available
        assert!(!shell::FISH_INIT_SCRIPT.is_empty());
        assert!(!shell::BASH_INIT_SCRIPT.is_empty());
        assert!(!shell::ZSH_INIT_SCRIPT.is_empty());
        assert!(!shell::POWERSHELL_INIT_SCRIPT.is_empty());
        
        // Check that scripts contain expected functionality
        assert!(shell::FISH_INIT_SCRIPT.contains("function x"));
        assert!(shell::BASH_INIT_SCRIPT.contains("x() {"));
        assert!(shell::ZSH_INIT_SCRIPT.contains("x() {"));
        assert!(shell::POWERSHELL_INIT_SCRIPT.contains("function x"));
    }
}