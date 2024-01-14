use std::{path::Path, fs, process::Command};

use crate::types::Category;
use anyhow::{Result, anyhow};

pub fn load_library(path: &Path, direct_json: bool) -> Result<Vec<Category>> {

    let mut categories: Vec<Category> = Vec::new();

    if direct_json {
        for file in path.read_dir().unwrap() {
            let file = file.unwrap();
            if file.path().is_dir() {
                continue;
            }
            if file.path().ends_with(".json") {
                categories.push(serde_json::from_str(&fs::read_to_string(file.path()).unwrap()).unwrap());
            }
        }
    } else {
        for file in path.read_dir().unwrap() {
            let file = file.unwrap();
            if file.path().is_dir() {
                continue;
            }
            let output = Command::new(file.path()).output()?;
            if !output.status.success() {
                return Err(anyhow!("Command: {:?} failed with output: {}{}", file.file_name(), String::from_utf8(output.stdout).unwrap(), String::from_utf8(output.stderr).unwrap()));
            }
            categories.push(serde_json::from_str(&String::from_utf8(output.stdout).unwrap())?);
        }
    }

    Ok(categories)
    
}
