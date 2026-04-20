# BVM - Blender Version Manager

BVM is an utility to manage multiple Blender installations while keeping all your **Addons, Keymaps, and Preferences** in one shared location.

![Preview](https://img.shields.io/badge/Blender-Community-orange?logo=blender&logoColor=white) 
![Rust](https://img.shields.io/badge/Made%20with-Rust-dea584?logo=rust&logoColor=white)

---

## Setup & Environment

BVM relies on the `BVM_PATH` environment variable to determine where to store Blender versions and your shared settings.

### 1. Set the Path
You must set `BVM_PATH` before running the application.

#### **Windows (PowerShell)**:
```powershell
[System.Environment]::SetEnvironmentVariable('BVM_PATH', 'C:\Path\To\Your\BVM', 'User')
```
*Note: Restart your terminal after setting this.*

#### **Linux / macOS**:
Add this to your `.bashrc` or `.zshrc`:
```bash
export BVM_PATH="$HOME/.bvm"
```

### 2. Folder Structure
Once set, BVM will create the following within that directory:
- `/versions`: Where different Blender instances are installed.
- `/shared_config`: The centralized folder for `addons`, `presets`, `config`, etc.
- `settings.json`: Stores your BVM preferences (like your default version).

---

## Usage
To launch your default Blender version without opening the menu:
```bash
bvm open
```
---

## Tips for Addons
If you have existing addons in your `%APPDATA%`, move them to:
`$BVM_PATH/shared_config/scripts/addons`

BVM maps the following Blender environment variables automatically during launch:
- `BLENDER_USER_RESOURCES`
- `BLENDER_USER_CONFIG`
- `BLENDER_USER_SCRIPTS`
- `BLENDER_USER_DATAFILES`

---

## Installation

1. Clone the repository.
2. Then install with cargo:
   ```bash
   cargo install --path bvm 
```
