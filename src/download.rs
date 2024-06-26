use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

use crate::types::LibraryItem;

pub fn setup_folder(path_str: &str) -> Result<()> {
    let path = Path::new(path_str);
    if path.exists() {
        if path.is_file() {
            panic!("Cannot create {path_str} folder as a file exists in its place");
        } else {
            return Ok(());
        }
    } else {
        fs::create_dir(path)?;
    }

    Ok(())
}

// Code sourced from https://gist.github.com/giuliano-oliveira/4d11d6b3bb003dba3a1b53f43d81b30d
pub async fn get_file(client: &reqwest::Client, url: &str, path: &str) -> Result<(), String> {
    // Reqwest setup
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    // Indicatif setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {url}"));

    // download chunks
    let mut file = File::create(path).or(Err(format!("Failed to create file '{path}'")))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err("Error while downloading file"))?;
        file.write_all(&chunk)
            .or(Err("Error while writing to file"))?;
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {url} to {path}"));
    Ok(())
}

pub fn handle_download_file(url: &str, path_str: &str, overwrite: bool) -> Result<()> {
    static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
        reqwest::ClientBuilder::new()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/117.0")
            .use_rustls_tls()
            .build()
            .unwrap()
    });

    static RT: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

    let path = Path::new(path_str);
    if path.exists() {
        if path.is_dir() {
            panic!("Cant download file {path_str}, as a folder exists in its place");
        } else if !overwrite {
            // If we're not overwriting, then we assume this file is done
            return Ok(());
        } else {
            // We're overwriting an existing file, so we need to delete the old one first
            fs::remove_file(path)?;
        }
    }

    RT.block_on(get_file(&CLIENT, url, path_str))
        .err()
        .map_or_else(|| Ok(()), |err| Err(anyhow!("{err}")))
}

fn handle_download_rsync(url: &str, path_str: &str) -> Result<()> {
    println!("Starting Download: {url}");

    let mut url = url.to_string();

    if url.starts_with("http://") {
        url = url.strip_prefix("http://").unwrap().to_string();
    }

    if !url.starts_with("rsync://") {
        url = format!("rsync://{url}");
    }

    Command::new("rsync")
        .args([
            "-rlptH",
            "--safe-links",
            "--delete-delay",
            "--delay-updates",
            "--no-motd",
            "--progress",
            "-h",
            &url,
            path_str,
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;

    println!("Completed Download: {url}");

    Ok(())
}

pub fn check_for_rsync() -> bool {
    Command::new("which")
        .arg("rsync")
        .output()
        .unwrap()
        .status
        .success()
}

pub fn get_item(path: &str, item: &LibraryItem, prefer_http: bool) {
    if !item.enabled() {
        return;
    }
    match item {
        LibraryItem::Document(doc) => match doc.download_type() {
            crate::types::DownloadType::Http => {
                let path = format!("{path}/{}", doc.url().split('/').last().unwrap());
                handle_download_file(doc.url(), &path, false)
            }
            crate::types::DownloadType::Rsync => {
                handle_download_rsync(doc.url(), &format!("{path}/{}", doc.name()))
            }
            crate::types::DownloadType::Either => {
                if crate::IS_WINDOWS || !*crate::HAS_RSYNC || prefer_http {
                    let path = format!("{path}/{}", doc.url().split('/').last().unwrap());
                    handle_download_file(doc.url(), &path, false)
                } else {
                    handle_download_rsync(doc.url(), &format!("{path}/{}", doc.name()))
                }
            }
        }
        .unwrap(),
        LibraryItem::Category(cat) => {
            let path = format!("{path}/{}", cat.name());
            setup_folder(&path).unwrap();
            cat.items
                .iter()
                .for_each(|item| get_item(&path, item, prefer_http));
        }
    }
}
