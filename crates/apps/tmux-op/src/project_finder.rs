use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

const IGNORED_DIRS: [&str; 5] = ["node_modules", "build", "target", "dist", "out"];

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub language: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug)]
pub struct ProjectInfo {
    pub name: String,
    pub language: String,
    pub directory: String,
}

pub fn find_project_files(root_dirs: &[PathBuf]) -> Result<Vec<ProjectInfo>> {
    let projects: Vec<ProjectInfo> = root_dirs
        .par_iter()
        .flat_map(|dir| {
            eprintln!("Searching in: {}", dir.display());
            let walker = WalkBuilder::new(dir)
                .hidden(false)
                .git_ignore(false)
                .filter_entry(|entry| {
                    !entry
                        .path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map_or(false, |name| IGNORED_DIRS.contains(&name))
                })
                .build();

            // Collect paths first, then process them in parallel
            let project_paths: Vec<_> = walker
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .path()
                        .file_name()
                        .map(|name| name == ".dexproject")
                        .unwrap_or(false)
                })
                .map(|entry| entry.into_path())
                .collect();

            // Process files in parallel
            project_paths
                .par_iter()
                .filter_map(|path| {
                    fs::read_to_string(path).ok().and_then(|content| {
                        serde_json::from_str::<ProjectConfig>(&content)
                            .map_err(|e| {
                                eprintln!("Failed to parse JSON from {}: {}", path.display(), e);
                            })
                            .ok()
                            .map(|config| {
                                let project_dir = path.parent().unwrap_or_else(|| Path::new(""));
                                let default_name = project_dir
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                ProjectInfo {
                                    name: config.name.unwrap_or(default_name),
                                    language: config
                                        .language
                                        .unwrap_or_else(|| "UNKNOWN".to_string())
                                        .to_uppercase(),
                                    directory: project_dir.to_string_lossy().to_string(),
                                }
                            })
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(projects)
}
