// src/db.rs

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::Path;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: String,
    pub last_access: DateTime<Utc>,
    pub visits: u32,
    pub rank: f64,
}

#[derive(Debug)]
pub struct Bookmark {
    pub name: String,
    pub path: String,
}

#[derive(Debug)]
pub struct Stats {
    pub total_entries: u32,
    pub total_visits: u32,
    pub most_visited: Vec<DirEntry>,
    pub recently_visited: Vec<DirEntry>,
}

pub struct Database {
    conn: Connection,
    config: Config,
}

impl Database {
    pub fn new(config: Config) -> Result<Self> {
        let db_path = dirs::data_dir()
            .context("Failed to find user's data directory")?
            .join("xneo/db.sqlite");
        
        if let Some(parent_dir) = db_path.parent() {
            std::fs::create_dir_all(parent_dir)
                .with_context(|| format!("Failed to create database directory at {:?}", parent_dir))?;
        }
        
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open or create database at {:?}", db_path))?;
        
        // 创建目录表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dirs (
                path        TEXT PRIMARY KEY,
                last_access INTEGER NOT NULL,
                visits_total INTEGER NOT NULL
            )",
            [],
        )?;
        
        // 创建书签表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                name        TEXT PRIMARY KEY,
                path        TEXT NOT NULL
            )",
            [],
        )?;
        
        // 创建索引提升查询性能
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dirs_visits ON dirs(visits_total DESC)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dirs_access ON dirs(last_access DESC)",
            [],
        )?;
        
        let mut db = Database { conn, config };
        
        // 自动清理过期条目
        if db.config.auto_clean_on_startup {
            let _ = db.auto_clean();
        }
        
        Ok(db)
    }
    
    pub fn add(&mut self, path: &str) -> Result<()> {
        // 检查是否应该忽略此路径
        if self.config.is_ignored(path) {
            return Ok(());
        }
        
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO dirs (path, last_access, visits_total) VALUES (?1, ?2, 1)
             ON CONFLICT(path) DO UPDATE SET
                last_access = excluded.last_access,
                visits_total = visits_total + 1",
            params![path, now],
        )?;
        
        // 如果条目数量超过限制，删除最旧的条目
        self.maintain_size_limit()?;
        
        Ok(())
    }
    
    pub fn query(&self, keywords: &[String]) -> Result<Vec<DirEntry>> {
        if keywords.is_empty() {
            return Ok(Vec::new());
        }
        
        // 获取所有条目
        let mut stmt = self.conn.prepare(
            "SELECT path, last_access, visits_total FROM dirs ORDER BY visits_total DESC"
        )?;
        
        let all_entries: Vec<DirEntry> = stmt
            .query_map([], |row| {
                let path: String = row.get(0)?;
                let last_access: DateTime<Utc> = row.get(1)?;
                let visits: u32 = row.get(2)?;
                let rank = self.calculate_rank(visits, &last_access, &Utc::now());
                
                Ok(DirEntry { path, last_access, visits, rank })
            })?
            .filter_map(Result::ok)
            .collect();
        
        let keyword = keywords.join(" ");
        let mut matches = Vec::new();
        
        // 1. 精确匹配
        for entry in &all_entries {
            if entry.path == keyword || entry.path.ends_with(&format!("/{}", keyword)) {
                matches.push(entry.clone());
            }
        }
        
        if !matches.is_empty() {
            matches.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
            return Ok(matches);
        }
        
        // 2. 目录名精确匹配
        let mut dir_matches = HashSet::new();
        for entry in &all_entries {
            for ancestor in Path::new(&entry.path).ancestors() {
                if let Some(dir_name) = ancestor.file_name().and_then(|s| s.to_str()) {
                    if dir_name == keyword {
                        if let Some(ancestor_str) = ancestor.to_str() {
                            dir_matches.insert(ancestor_str.to_string());
                        }
                    }
                }
            }
        }
        
        for path in dir_matches {
            if let Some(entry) = all_entries.iter().find(|e| e.path.starts_with(&path)) {
                matches.push(entry.clone());
            }
        }
        
        if !matches.is_empty() {
            matches.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
            return Ok(matches);
        }
        
        // 3. 模糊匹配（如果启用）
        if self.config.enable_fuzzy_matching {
            let matcher = SkimMatcherV2::default();
            let mut fuzzy_matches = Vec::new();
            
            for entry in &all_entries {
                if let Some(score) = matcher.fuzzy_match(&entry.path, &keyword) {
                    let combined_score = (score as f64) * entry.rank;
                    fuzzy_matches.push((entry.clone(), combined_score));
                }
            }
            
            fuzzy_matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            matches = fuzzy_matches.into_iter().map(|(entry, _)| entry).collect();
        }
        
        // 4. 子字符串匹配（作为最后的回退）
        if matches.is_empty() {
            for entry in &all_entries {
                if entry.path.to_lowercase().contains(&keyword.to_lowercase()) {
                    matches.push(entry.clone());
                }
            }
            matches.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
        }
        
        Ok(matches.into_iter().take(20).collect()) // 限制结果数量
    }
    
    fn calculate_rank(&self, visits: u32, last_access: &DateTime<Utc>, now: &DateTime<Utc>) -> f64 {
        let age_in_hours = (now.timestamp() - last_access.timestamp()) as f64 / 3600.0;
        let frequency_score = (visits as f64).ln() + 1.0; // 对数缩放访问次数
        let recency_score = 1.0 / (age_in_hours + 1.0); // 时间衰减
        
        frequency_score * 0.7 + recency_score * 0.3
    }
    
    pub fn find_stale(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT path FROM dirs")?;
        let paths = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(Result::ok)
            .collect::<Vec<String>>();
        
        let mut stale_paths = Vec::new();
        for path_str in paths {
            if !Path::new(&path_str).exists() {
                stale_paths.push(path_str);
            }
        }
        
        Ok(stale_paths)
    }
    
    pub fn purge(&mut self, paths_to_delete: &[String]) -> Result<usize> {
        if paths_to_delete.is_empty() {
            return Ok(0);
        }
        
        let tx = self.conn.transaction()?;
        let mut deleted_count = 0;
        
        {
            let mut stmt = tx.prepare_cached("DELETE FROM dirs WHERE path = ?")?;
            for path in paths_to_delete {
                let changed_rows = stmt.execute(params![path])?;
                deleted_count += changed_rows;
            }
        }
        
        tx.commit()?;
        Ok(deleted_count)
    }
    
    fn maintain_size_limit(&mut self) -> Result<()> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM dirs",
            [],
            |row| row.get(0)
        )?;
        
        if count > self.config.max_entries as u32 {
            let excess = count - self.config.max_entries as u32;
            self.conn.execute(
                "DELETE FROM dirs WHERE path IN (
                    SELECT path FROM dirs 
                    ORDER BY last_access ASC 
                    LIMIT ?1
                )",
                params![excess],
            )?;
        }
        
        Ok(())
    }
    
    fn auto_clean(&mut self) -> Result<usize> {
        let stale_paths = self.find_stale()?;
        self.purge(&stale_paths)
    }
    
    // 书签功能
    pub fn add_bookmark(&mut self, name: &str, path: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO bookmarks (name, path) VALUES (?1, ?2)",
            params![name, path],
        )?;
        Ok(())
    }
    
    pub fn remove_bookmark(&mut self, name: &str) -> Result<bool> {
        let changes = self.conn.execute(
            "DELETE FROM bookmarks WHERE name = ?1",
            params![name],
        )?;
        Ok(changes > 0)
    }
    
    pub fn get_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, path FROM bookmarks ORDER BY name"
        )?;
        
        let bookmarks = stmt
            .query_map([], |row| {
                Ok(Bookmark {
                    name: row.get(0)?,
                    path: row.get(1)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();
        
        Ok(bookmarks)
    }
    
    pub fn get_bookmark(&self, name: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT path FROM bookmarks WHERE name = ?1")?;
        let mut rows = stmt.query_map(params![name], |row| row.get(0))?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }
    
    // 统计功能
    pub fn get_stats(&self) -> Result<Stats> {
        let total_entries: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM dirs",
            [],
            |row| row.get(0)
        )?;
        
        let total_visits: u32 = self.conn.query_row(
            "SELECT COALESCE(SUM(visits_total), 0) FROM dirs",
            [],
            |row| row.get(0)
        )?;
        
        // 最常访问的目录
        let mut stmt = self.conn.prepare(
            "SELECT path, last_access, visits_total FROM dirs 
             ORDER BY visits_total DESC LIMIT 10"
        )?;
        
        let most_visited = stmt
            .query_map([], |row| {
                let path: String = row.get(0)?;
                let last_access: DateTime<Utc> = row.get(1)?;
                let visits: u32 = row.get(2)?;
                let rank = self.calculate_rank(visits, &last_access, &Utc::now());
                
                Ok(DirEntry { path, last_access, visits, rank })
            })?
            .filter_map(Result::ok)
            .collect();
        
        // 最近访问的目录
        let mut stmt = self.conn.prepare(
            "SELECT path, last_access, visits_total FROM dirs 
             ORDER BY last_access DESC LIMIT 10"
        )?;
        
        let recently_visited = stmt
            .query_map([], |row| {
                let path: String = row.get(0)?;
                let last_access: DateTime<Utc> = row.get(1)?;
                let visits: u32 = row.get(2)?;
                let rank = self.calculate_rank(visits, &last_access, &Utc::now());
                
                Ok(DirEntry { path, last_access, visits, rank })
            })?
            .filter_map(Result::ok)
            .collect();
        
        Ok(Stats {
            total_entries,
            total_visits,
            most_visited,
            recently_visited,
        })
    }
}