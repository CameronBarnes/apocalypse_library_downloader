use std::{fs, path::Path, process::Command};

use anyhow::{anyhow, Result};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::types::{Category, LibraryItem};

pub fn load_library(path: &Path, direct_json: bool) -> Result<Category> {
    let mut root = Category::new("Apocalypse Library".into(), vec![], false);

    if direct_json {
        let results: Vec<Vec<LibraryItem>> = path
            .read_dir()
            .unwrap()
            .par_bridge()
            .map(|file| file.unwrap())
            .filter(|file| !file.path().is_dir())
            .map(|file| {
                let file_path = file.path();
                let extension = file_path.extension();
                if extension.is_some_and(|ext| ext.eq_ignore_ascii_case("json")) {
                    println!("Loading fron file: {:?}", file.path());
                    let str = fs::read_to_string(file.path()).unwrap();
                    if str.contains('\n') {
                        let mut coll: Vec<LibraryItem> = Vec::new();
                        for line in str.lines() {
                            coll.push(serde_json::from_str(line).unwrap());
                        }
                        coll
                    } else {
                        vec![serde_json::from_str(&str).unwrap()]
                    }
                } else {
                    vec![]
                }
            })
            .collect();
        results
            .into_iter()
            .flatten()
            .for_each(|item| root.add(item));
    } else {
        for file in path.read_dir().unwrap() {
            // TODO: Make parallel
            let file = file.unwrap();
            let file_path = file.path();
            if file_path.is_dir() {
                continue;
            }
            let extension = file_path.extension();
            if (!crate::IS_WINDOWS && extension.is_some())
                || (crate::IS_WINDOWS
                    && !extension.is_some_and(|ext| ext.eq_ignore_ascii_case("exe")))
            {
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
            let str = String::from_utf8(output.stdout).unwrap();
            if str.contains('\n') {
                for line in str.lines() {
                    root.add(serde_json::from_str(line)?);
                }
            } else {
                root.add(serde_json::from_str(&str)?);
            }
        }
    }

    root.fix_counter(); // Will recursively fix the list counter objects of all contained
                        // categories

    Ok(root)
}
