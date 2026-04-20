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
    pub settings_path: PathBuf,
}

impl BlenderManager {
    pub fn get_config_path() -> Result<PathBuf, anyhow::Error> {
        let home = directories::BaseDirs::new()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.home_dir().join(".bvm_config.json"))
    }

    pub fn get_stored_base_path() -> Option<PathBuf> {
        let config_path = Self::get_config_path().ok()?;
        if let Ok(content) = fs::read_to_string(config_path) {
            let v: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            return v.get("base_path").and_then(|v| v.as_str()).map(PathBuf::from);
        }
        None
    }

    pub fn store_base_path(path: &std::path::Path) -> Result<(), anyhow::Error> {
        let config_path = Self::get_config_path()?;
        let v = serde_json::json!({
            "base_path": path.to_string_lossy()
        });
        let content = serde_json::to_string_pretty(&v)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    pub fn new() -> Result<Self, anyhow::Error> {
        let base_path = Self::get_stored_base_path()
            .ok_or_else(|| anyhow::anyhow!("No base path configured"))?;

        let settings_path = base_path.join("settings.json");
        let manager = Self { base_path, settings_path };
        manager.ensure_dirs()?;
        Ok(manager)
    }

    fn ensure_dirs(&self) -> Result<(), anyhow::Error> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)?;
        }

        let versions_dir = self.base_path.join("versions");
        if !versions_dir.exists() {
            fs::create_dir_all(&versions_dir)?;
        }

        let shared_dir = self.base_path.join("shared_config");
        let subdirs = ["config", "scripts", "datafiles", "scripts/addons", "scripts/presets", "scripts/startup", "scripts/modules"];
        for sub in subdirs {
            let path = shared_dir.join(sub);
            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
        }
        Ok(())
    }

    pub fn get_default_version(&self) -> Option<String> {
        if let Ok(content) = fs::read_to_string(&self.settings_path) {
            let v: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            return v.get("default_version").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
        None
    }

    pub fn set_default_version(&self, version: &str) -> Result<(), anyhow::Error> {
        let mut v: serde_json::Value = if let Ok(content) = fs::read_to_string(&self.settings_path) {
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        
        v["default_version"] = serde_json::Value::String(version.to_string());
        let content = serde_json::to_string_pretty(&v)?;
        fs::write(&self.settings_path, content)?;
        Ok(())
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
