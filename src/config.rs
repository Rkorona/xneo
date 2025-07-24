// src/config.rs

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub max_entries: usize,
    pub ignored_patterns: Vec<String>,
    pub update_threshold_hours: u64,
    pub enable_fuzzy_matching: bool,
    pub show_stats_on_query: bool,
    pub auto_clean_on_startup: bool,
    pub fzf_options: String,

    #[serde(skip)]
    #[serde(default = "default_globset")]
    pub(crate) compiled_ignores: GlobSet,
}

fn default_globset() -> GlobSet {
    GlobSetBuilder::new().build().unwrap()
}

impl Default for Config {
    fn default() -> Self {
       
        let ignored_patterns = vec![
            // node_modules
            "**/node_modules".to_string(),
            "**/node_modules/**".to_string(),
            // .git
            "**/.git".to_string(),
            "**/.git/**".to_string(),
            // target
            "**/target".to_string(),
            "**/target/**".to_string(),
            // .cache
            "**/.cache".to_string(),
            "**/.cache/**".to_string(),
            // build
            "**/build".to_string(),
            "**/build/**".to_string(),
            // dist
            "**/dist".to_string(),
            "**/dist/**".to_string(),
            // File patterns
            "**/*.log".to_string(),
            "**/*.tmp".to_string(),
        ];

        let mut builder = GlobSetBuilder::new();
        for pattern in &ignored_patterns {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }
        
        let compiled_ignores = builder.build().unwrap_or_else(|_| {
            GlobSetBuilder::new().build().unwrap()
        });

        Self {
            max_entries: 1000,
            ignored_patterns,
            update_threshold_hours: 168,
            enable_fuzzy_matching: true,
            show_stats_on_query: false,
            auto_clean_on_startup: false,
            fzf_options: "--height=40% --reverse --border".to_string(),
            compiled_ignores,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
            let mut config: Config = serde_json::from_str(&content)
                .with_context(|| "Failed to parse config file")?;
            
            config.compile_ignores()?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    fn compile_ignores(&mut self) -> Result<()> {
        let mut builder = GlobSetBuilder::new();
        for pattern in &self.ignored_patterns {
            let glob = Glob::new(pattern)
                .with_context(|| format!("Invalid glob pattern in config: '{}'", pattern))?;
            builder.add(glob);
        }
        self.compiled_ignores = builder.build()
            .context("Failed to build globset from ignored patterns")?;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to find user's config directory")?;
        Ok(config_dir.join("xneo").join("config.json"))
    }
    
    pub fn is_ignored(&self, path: &str) -> bool {
        self.compiled_ignores.is_match(Path::new(path))
    }
}