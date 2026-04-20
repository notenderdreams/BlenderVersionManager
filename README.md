# BVM - Blender Version Manager

BVM is an utility to manage multiple Blender installations while keeping all your **Addons, Keymaps, and Preferences** in one shared location.

![Preview](https://img.shields.io/badge/Blender-Community-orange?logo=blender&logoColor=white) 
![Rust](https://img.shields.io/badge/Made%20with-Rust-dea584?logo=rust&logoColor=white)

---

## Setup & Environment

BVM no longer requires an environment variable. 

### First Run
When you run BVM for the first time, it will ask you to enter the **absolute path** where you want to store your Blender versions and shared settings. 

- This path is stored in a global config file: `~/.bvm_config.json`.
- All your Blender installations and shared configurations will be managed inside the directory you choose.

### Changing the Path
If you want to move your BVM core to a different location:
1. Move your current BVM folder to the new location.
2. Update the `base_path` in `~/.bvm_config.json`.

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
