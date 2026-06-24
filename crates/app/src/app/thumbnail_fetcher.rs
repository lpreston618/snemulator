use std::path::PathBuf;
use crate::app::rom_paths::RomPaths;
use crate::app::settings::Settings;
use std::sync::mpsc::Sender;

const INDEX_FILENAME: &str = "thumbnail_index.txt";
const GITHUB_TREE_URL: &str =
    "https://api.github.com/repos/libretro-thumbnails/\
     Nintendo_-_Super_Nintendo_Entertainment_System/git/trees/HEAD";
const RAW_BASE_URL: &str =
    "https://raw.githubusercontent.com/libretro-thumbnails/\
     Nintendo_-_Super_Nintendo_Entertainment_System/master/Named_Boxarts";

const MIN_SIMILARITY: f64 = 0.35;

pub struct ThumbnailResult {
    pub stem: String,
    pub path: Option<PathBuf>, // None = not found
}

pub fn spawn_thumbnail_resolver(
    stems: Vec<(String, PathBuf)>, // (stem, rom_path) for each Loading entry
    tx: Sender<ThumbnailResult>,
) {
    std::thread::spawn(move || {
        let index = match ensure_thumbnail_index() {
            Some(idx) => idx,
            None => {
                log::warn!("Thumbnail index unavailable; skipping thumbnail fetch.");
                return;
            }
        };

        for (stem, _rom_path) in stems {
            let result = try_fetch_thumbnail(&stem, &index);
            let _ = tx.send(ThumbnailResult { stem, path: result });
            // tx.send failing just means the receiver was dropped (app closed), safe to ignore
        }

        log::debug!("Finished fetching thumbnails, closing thread.");
    });
}

/// Fetches and writes a single thumbnail. Returns the saved path on success.
fn try_fetch_thumbnail(stem: &str, index: &[String]) -> Option<PathBuf> {
    let rom_paths = RomPaths::new(stem)?;
    let candidates = best_matches(stem, index, 3);

    for candidate in candidates {
        let Some(bytes) = fetch_valid_png_data(&candidate) else { continue };

        let dest = rom_paths.thumbnail_path();
        if std::fs::write(&dest, &bytes).is_err() {
            log::warn!("Failed to write thumbnail for '{}'", stem);
            continue;
        }

        // Update manifest
        let mut manifest = RomPaths::find_manifest_by_stem(stem).unwrap_or_default();
        manifest.thumbnail_path = Some(dest.clone());
        rom_paths.write_manifest(&manifest);

        log::info!("Fetched thumbnail for '{}' -> '{}'", stem, candidate);
        return Some(dest);
    }

    None
}

/// Returns path to the index file, downloading it first if absent.
fn ensure_thumbnail_index() -> Option<Vec<String>> {
    let index_path = index_file_path()?;

    if !index_path.exists() {
        log::info!("Thumbnail index not found, fetching from GitHub...");
        let filenames = fetch_thumbnail_index()?;
        let text = filenames.join("\n");
        if let Err(e) = std::fs::write(&index_path, &text) {
            log::warn!("Could not save thumbnail index: {e}");
        }
        return Some(filenames);
    }

    let text = std::fs::read_to_string(&index_path).ok()?;
    Some(text.lines().map(|l| l.to_string()).collect())
}

fn index_file_path() -> Option<PathBuf> {
    Some(Settings::data_dir()?.join(INDEX_FILENAME))
}

fn fetch_thumbnail_index() -> Option<Vec<String>> {
    let agent = ureq::agent();

    // Step 1: get root tree, find Named_Boxarts SHA
    let root: serde_json::Value = agent
        .get(GITHUB_TREE_URL)
        .header("User-Agent", "snemulator")
        .call()
        .ok()?
        .into_body()
        .read_json()
        .ok()?;

    let boxarts_sha = root["tree"]
        .as_array()?
        .iter()
        .find(|e| e["path"] == "Named_Boxarts")?
        ["sha"].as_str()?
        .to_string();

    // Step 2: fetch the boxarts subtree recursively
    let tree_url = format!(
        "https://api.github.com/repos/libretro-thumbnails/\
         Nintendo_-_Super_Nintendo_Entertainment_System/git/trees/{}?recursive=1",
        boxarts_sha
    );

    let tree: serde_json::Value = agent
        .get(&tree_url)
        .header("User-Agent", "snemulator")
        .call()
        .ok()?
        .into_body()
        .read_json()
        .ok()?;

    if tree["truncated"].as_bool().unwrap_or(false) {
        log::warn!("Thumbnail index tree was truncated by GitHub API");
    }

    let filenames: Vec<String> = tree["tree"]
        .as_array()?
        .iter()
        .filter_map(|e| e["path"].as_str().map(|s| s.to_string()))
        .collect();

    log::info!("Fetched {} thumbnail filenames", filenames.len());
    Some(filenames)
}

/// Returns up to `n` index entries whose names best match `stem`.
/// Returns empty vec if the best match score is below MIN_SIMILARITY.
fn best_matches(stem: &str, index: &[String], n: usize) -> Vec<String> {
    let needle = normalize(stem);

    let mut scored: Vec<(f64, &String)> = index
        .iter()
        .filter(|s| s.ends_with(".png"))
        .map(|entry| {
            // Strip the .png extension for comparison
            let name = entry.strip_suffix(".png").unwrap_or(entry);
            let score = fuzzy_score(&needle, &normalize(name));
            (score, entry)
        })
        .collect();

    // Higher score = better match; sort descending
    scored.sort_unstable_by(|a, b| {
        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Return empty if best match doesn't meet minimum similarity
    if scored.first().map_or(true, |(score, _)| *score < MIN_SIMILARITY) {
        return Vec::new();
    }

    scored.into_iter().take(n).map(|(_, s)| s.clone()).collect()
}

/// Lowercase, collapse runs of whitespace/punctuation to a single space.
fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_sep = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            out.push(' ');
            last_was_sep = true;
        }
    }
    out.trim().to_string()
}

/// Bigram-overlap score: counts shared character bigrams (higher = more similar).
fn fuzzy_score(a: &str, b: &str) -> f64 {
    if a.len() < 2 || b.len() < 2 {
        return 0.0;
    }

    let bigrams_a: std::collections::HashSet<_> =
        a.chars().zip(a.chars().skip(1)).collect();

    let bigrams_b: std::collections::HashSet<_> =
        b.chars().zip(b.chars().skip(1)).collect();

    let intersection = bigrams_a.intersection(&bigrams_b).count() as f64;
    let max_possible = bigrams_a.len().max(bigrams_b.len()) as f64;

    intersection / max_possible
}

/// Fetches the PNG for `filename` from the raw GitHub URL.
/// Returns `Some(bytes)` only if the response has a valid PNG magic header.
fn fetch_valid_png_data(filename: &str) -> Option<Vec<u8>> {
    let encoded = url_encode(filename);
    let url = format!("{}/{}", RAW_BASE_URL, encoded);

    let bytes = ureq::get(&url)
        .header("User-Agent", "snemulator")
        .call()
        .ok().unwrap()
        .into_body()
        .read_to_vec()
        .ok().unwrap();

    let is_valid = is_valid_png(&bytes);

    if !is_valid {
        log::debug!("'{}' failed PNG validation (likely a git-lfs pointer)", filename);
        None
    } else {
        Some(bytes)
    }
}

fn is_valid_png(bytes: &[u8]) -> bool {
    bytes.len() > 7 && bytes[0] == 0x89 && bytes[1] == 0x50 && bytes[2] == 0x4e &&
    bytes[3] == 0x47 && bytes[4] == 0xd && bytes[5] == 0xa && bytes[6] == 0x1a
}

/// Minimal percent-encoding for URL path segments (handles spaces and parens).
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' | b'!' | b'\'' | b',' => out.push(b as char),
            b => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}