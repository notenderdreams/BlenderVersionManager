use crate::blender::{BlenderVersion};
use crate::app::Action;
use anyhow::Result;
use tokio::sync::mpsc;
use std::fs;
use std::io;
use futures_util::StreamExt;

pub async fn fetch_blender_versions() -> Result<Vec<BlenderVersion>> {
    let client = reqwest::Client::new();
    let res = client.get("https://download.blender.org/release/").send().await?.text().await?;
    
    let mut versions = Vec::new();
    let re = regex::Regex::new(r#"href="Blender(\d+\.\d+)/""#).unwrap();
    
    let mut major_versions = Vec::new();
    for cap in re.captures_iter(&res) {
        major_versions.push(cap[1].to_string());
    }
    
    major_versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.split('.').map(|s| s.parse().unwrap_or(0)).collect();
        let b_parts: Vec<u32> = b.split('.').map(|s| s.parse().unwrap_or(0)).collect();
        b_parts.cmp(&a_parts)
    });
    
    for v in major_versions.iter().take(10) {
        let sub_url = format!("https://download.blender.org/release/Blender{}/", v);
        if let Ok(sub_res) = client.get(&sub_url).send().await?.text().await {
            let zip_re = regex::Regex::new(r#"href="(blender-[0-9.]+-windows-x64\.zip)""#).unwrap();
            let mut found = Vec::new();
            for cap in zip_re.captures_iter(&sub_res) {
                found.push(cap[1].to_string());
            }
            
            found.sort();
            if let Some(zip_name) = found.last() {
                versions.push(BlenderVersion {
                    version: v.clone(),
                    url: format!("{}{}", sub_url, zip_name),
                });
            }
        }
    }
    
    Ok(versions)
}

pub async fn install_version(v: BlenderVersion, base_path: std::path::PathBuf, tx: mpsc::Sender<Action>) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(&v.url).send().await?;
    
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    
    let filename = v.url.split('/').last().unwrap_or("blender.zip");
    let versions_dir = base_path.join("versions");
    let zip_path = versions_dir.join(filename);
    
    let mut file = std::fs::File::create(&zip_path)?;
    let mut stream = response.bytes_stream();
    
    while let Some(item) = stream.next().await {
        let chunk = item?;
        std::io::copy(&mut &chunk[..], &mut file)?;
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let progress = downloaded as f64 / total_size as f64;
            let _ = tx.send(Action::UpdateProgress(progress)).await;
        }
    }
    
    let _ = tx.send(Action::SetStatus(format!("Extracting {}...", filename))).await;
    
    let file = std::fs::File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    let target_dir = versions_dir.join(&v.version);
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir)?;
    }
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    
    fs::remove_file(&zip_path)?;
    
    let _ = tx.send(Action::SetStatus(format!("Successfully installed Blender {}", v.version))).await;
    let _ = tx.send(Action::RefreshInstalled).await;
    
    Ok(())
}
