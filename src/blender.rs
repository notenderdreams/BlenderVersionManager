use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderVersion {
    pub version: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVersion {
    pub version: String,
    pub path: PathBuf,
}

pub struct BlenderManager {
    pub base_path: PathBuf,
}

impl BlenderManager {
    pub fn new() -> Result<Self, anyhow::Error> {
        let base_path = std::env::var("BVM_PATH")
            .map(PathBuf::from)
            .or_else(|_| -> Result<PathBuf, anyhow::Error> {
                // Fallback to a default if not set, but we should inform the user
                let home = directories::BaseDirs::new()
                    .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
                Ok(home.home_dir().join(".bvm"))
            })?;

        if !base_path.exists() {
            fs::create_dir_all(&base_path)?;
        }

        // Create subdirectories
        let versions_dir = base_path.join("versions");
        if !versions_dir.exists() {
            fs::create_dir_all(&versions_dir)?;
        }

        let shared_dir = base_path.join("shared_config");
        let config_dir = shared_dir.join("config");
        let scripts_dir = shared_dir.join("scripts");
        let datafiles_dir = shared_dir.join("datafiles");

        for dir in &[&config_dir, &scripts_dir, &datafiles_dir] {
            if !dir.exists() {
                fs::create_dir_all(dir)?;
            }
        }

        // Standard subdirectories for scripts to ensure Blender detects them
        for sub in &["addons", "presets", "startup", "modules"] {
            let sub_path = scripts_dir.join(sub);
            if !sub_path.exists() {
                fs::create_dir_all(sub_path)?;
            }
        }

        Ok(Self { base_path })
    }

    pub fn get_versions_dir(&self) -> PathBuf {
        self.base_path.join("versions")
    }

    pub fn get_shared_config_dir(&self) -> PathBuf {
        self.base_path.join("shared_config")
    }

    pub fn list_installed(&self) -> Result<Vec<InstalledVersion>, anyhow::Error> {
        let versions_dir = self.get_versions_dir();
        let mut installed = Vec::new();

        if versions_dir.exists() {
            for entry in fs::read_dir(versions_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let version = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    installed.push(InstalledVersion { version, path });
                }
            }
        }
        
        // Sort by version (simple string sort for now)
        installed.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(installed)
    }

    pub fn remove_version(&self, version: &str) -> Result<(), anyhow::Error> {
        let path = self.get_versions_dir().join(version);
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    pub fn get_launch_env(&self) -> Vec<(String, String)> {
        let shared = self.get_shared_config_dir();
        vec![
            ("BLENDER_USER_RESOURCES".to_string(), shared.to_string_lossy().to_string()),
            ("BLENDER_USER_CONFIG".to_string(), shared.join("config").to_string_lossy().to_string()),
            ("BLENDER_USER_SCRIPTS".to_string(), shared.join("scripts").to_string_lossy().to_string()),
            ("BLENDER_USER_DATAFILES".to_string(), shared.join("datafiles").to_string_lossy().to_string()),
        ]
    }
}

use std::process::{Command, Stdio};

pub fn launch_blender(path: PathBuf, env_vars: Vec<(String, String)>) -> anyhow::Result<()> {
    // Find blender.exe
    fn find_exe(dir: &std::path::Path) -> Option<std::path::PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(found) = find_exe(&p) {
                        return Some(found);
                    }
                } else if p.file_name().and_then(|n| n.to_str()) == Some("blender.exe") {
                    return Some(p);
                }
            }
        }
        None
    }
    
    let exe_path = find_exe(&path);
    
    if let Some(exe) = exe_path {
        let mut cmd = Command::new(exe);
        for (k, v) in env_vars {
            cmd.env(k, v);
        }
        // Silence standard output and error to avoid messing up the TUI
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        cmd.spawn()?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Could not find blender.exe in {:?}", path))
    }
}
