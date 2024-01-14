use std::{fs, path::Path, process::Command};

use anyhow::{anyhow, Result};

use crate::types::LibraryItem;

pub fn load_library(path: &Path, direct_json: bool) -> Result<Vec<LibraryItem>> {
    let mut categories: Vec<LibraryItem> = Vec::new(); //TODO: allow merging of categories with the
                                                       //same name and parent

    if direct_json {
        for file in path.read_dir().unwrap() {
            let file = file.unwrap();
            if file.path().is_dir() {
                continue;
            }
            if file.path().ends_with(".json") {
                for line in fs::read_to_string(file.path()).unwrap().lines() {
                    categories.push(serde_json::from_str(line)?);
                }
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
                return Err(anyhow!(
                    "Command: {:?} failed with output: {}{}",
                    file.file_name(),
                    String::from_utf8(output.stdout).unwrap(),
                    String::from_utf8(output.stderr).unwrap()
                ));
            }
            for line in String::from_utf8(output.stdout).unwrap().lines() {
                categories.push(serde_json::from_str(line)?);
            }
        }
    }

    Ok(categories)
}
