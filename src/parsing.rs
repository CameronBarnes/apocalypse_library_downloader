use std::{fs, path::Path, process::Command};

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::types::Category;

/// Loads a library of items from either executables or json files in the directory at the provided
/// `path`. `direct_json` will make it ignore executables and load from json files instead if true
// FIXME: Fix the unpleasant amount of unwraps in the code here
pub fn load_library(path: &Path, direct_json: bool) -> Category {
    let mut root = Category::new("Apocalypse Library".into(), vec![], false);

    let results: Vec<Vec<String>> = if direct_json {
        path.read_dir()
            .unwrap()
            .par_bridge()
            .map(std::result::Result::unwrap)
            .filter(|file| !file.path().is_dir())
            .map(|file| {
                let file_path = file.path();
                let extension = file_path.extension();
                if extension.is_some_and(|ext| ext.eq_ignore_ascii_case("json")) {
                    println!("Loading fron file: {:?}", file.path());
                    let str = fs::read_to_string(file.path()).unwrap();
                    if str.contains('\n') {
                        let mut coll: Vec<String> = Vec::new();
                        for line in str.lines() {
                            coll.push(line.to_string());
                        }
                        coll
                    } else {
                        vec![str]
                    }
                } else {
                    vec![]
                }
            })
            .collect()
    } else {
        path.read_dir()
            .unwrap()
            .par_bridge()
            .map(|file| {
                let mut coll: Vec<String> = Vec::new();
                let file = file.unwrap();
                let file_path = file.path();
                if file_path.is_dir() {
                    return vec![];
                }
                let extension = file_path.extension();
                if (!crate::IS_WINDOWS && extension.is_some())
                    || (crate::IS_WINDOWS
                        && !extension.is_some_and(|ext| ext.eq_ignore_ascii_case("exe")))
                {
                    return vec![];
                }
                let output = Command::new(file.path()).output().unwrap();
                assert!(
                    output.status.success(),
                    "Command: {:?} failed with output: {}{}",
                    file.file_name(),
                    String::from_utf8(output.stdout).unwrap(),
                    String::from_utf8(output.stderr).unwrap()
                );
                let str = String::from_utf8(output.stdout).unwrap();
                if str.contains('\n') {
                    for line in str.lines() {
                        coll.push(line.to_string());
                    }
                } else {
                    coll.push(str);
                }
                coll
            })
            .collect()
    };

    results
        .into_iter()
        .flatten()
        .flat_map(|str| serde_json::from_str(&str))
        .for_each(|item| {
            root.add(item);
        });

    root.fix_counter(); // Will recursively fix the list counter objects of all contained
                        // categories
    root.items.iter_mut().for_each(|item| {
        item.set_enabled_recursive();
    }); // Getting some odd behavior, this should fix it

    root
}
