use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub struct RomPaths {
    rom_dir: PathBuf,  // Data directory for this ROM (where to store save states, thumbnail, metadata)
}

impl RomPaths {
    pub fn new(rom_name: &str) -> Option<Self> {
        let sanitized = sanitize_name(rom_name);
        let rom_dir = crate::app::settings::Settings::data_dir()?.join(sanitized);
        Some(Self { rom_dir })
    }

    /// Scans all sibling rom dirs for a manifest matching the given CRC32.
    pub fn find_by_hash(crc: u32) -> Option<Self> {
        let data_dir = crate::app::settings::Settings::data_dir()?;
        for entry in std::fs::read_dir(&data_dir).ok()?.flatten() {
            let manifest_path = entry.path().join("manifest.json");
            if let Ok(text) = std::fs::read_to_string(&manifest_path) {
                if let Ok(m) = serde_json::from_str::<RomManifest>(&text) {
                    if m.rom_crc == crc {
                        return Some(Self { rom_dir: entry.path() });
                    }
                }
            }
        }
        None
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.rom_dir)
    }

    pub fn sav_path(&self) -> PathBuf {
        self.rom_dir.join("game.sav")
    }

    pub fn state_path(&self, slot: u32) -> PathBuf {
        self.rom_dir.join(format!("state{}.ss", slot))
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.rom_dir.join("manifest.json")
    }

    pub fn thumbnail_path(&self) -> PathBuf {
        self.rom_dir.join("thumbnail.png")
    }

    pub fn write_manifest(&self, manifest: &RomManifest) {
        if let Ok(text) = serde_json::to_string_pretty(manifest) {
            let _ = std::fs::write(self.manifest_path(), text);
        }
    }

    /// Finds a manifest by the sanitized ROM stem name (used during library scan).
    pub fn find_manifest_by_stem(stem: &str) -> Option<RomManifest> {
        let dir = crate::app::settings::Settings::data_dir()?.join(sanitize_name(stem));
        let text = std::fs::read_to_string(dir.join("manifest.json")).ok()?;
        serde_json::from_str(&text).ok()
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct RomManifest {
    pub rom_crc: u32,
    pub display_name: String,
    #[serde(default)]
    pub last_played: Option<u64>,   // Unix timestamp (seconds)
    #[serde(default)]
    pub play_time_secs: u64,
    #[serde(default)]
    pub thumbnail_path: Option<PathBuf>,
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>()
        .trim_end_matches('.')
        .to_string()
}