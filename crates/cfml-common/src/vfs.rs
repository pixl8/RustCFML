//! Virtual filesystem abstraction for RustCFML.
//!
//! Allows the VM to read source files from either the real filesystem (`RealFs`)
//! or from an in-memory archive embedded in the binary (`EmbeddedFs`).

use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

/// Directory entry returned by `Vfs::read_dir`.
#[derive(Debug, Clone)]
pub struct VfsDirEntry {
    pub name: String,
    pub is_file: bool,
    pub is_dir: bool,
}

/// Virtual filesystem trait — abstracts source file I/O so the VM can read
/// from disk or from an embedded archive.
pub trait Vfs: Send + Sync {
    fn read_to_string(&self, path: &str) -> io::Result<String>;
    fn read(&self, path: &str) -> io::Result<Vec<u8>>;
    fn exists(&self, path: &str) -> bool;
    fn is_file(&self, path: &str) -> bool;
    fn is_dir(&self, path: &str) -> bool;
    fn read_dir(&self, path: &str) -> io::Result<Vec<VfsDirEntry>>;
    /// File modification time (for bytecode cache invalidation).
    fn modified(&self, path: &str) -> io::Result<SystemTime>;
    /// Canonicalize a path (resolve symlinks, make absolute).
    fn canonicalize(&self, path: &str) -> io::Result<String>;
}

// ---------------------------------------------------------------------------
// RealFs — delegates to std::fs (default behavior)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RealFs;

impl Vfs for RealFs {
    fn read_to_string(&self, path: &str) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn is_file(&self, path: &str) -> bool {
        Path::new(path).is_file()
    }

    fn is_dir(&self, path: &str) -> bool {
        Path::new(path).is_dir()
    }

    fn read_dir(&self, path: &str) -> io::Result<Vec<VfsDirEntry>> {
        let entries = std::fs::read_dir(path)?;
        let mut result = Vec::new();
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                // Use metadata() (not file_type()) to follow symlinks
                let (is_file, is_dir) = entry.metadata()
                    .map(|md| (md.is_file(), md.is_dir()))
                    .unwrap_or((false, false));
                result.push(VfsDirEntry {
                    name: name.to_string(),
                    is_file,
                    is_dir,
                });
            }
        }
        Ok(result)
    }

    fn modified(&self, path: &str) -> io::Result<SystemTime> {
        std::fs::metadata(path)?.modified()
    }

    fn canonicalize(&self, path: &str) -> io::Result<String> {
        std::fs::canonicalize(path).map(|p| p.to_string_lossy().to_string())
    }
}

// ---------------------------------------------------------------------------
// EmbeddedFs — reads from an in-memory archive
// ---------------------------------------------------------------------------

/// An in-memory filesystem backed by a map of normalized paths to file contents.
/// All paths are stored as forward-slash-separated, lowercase, without leading slash.
pub struct EmbeddedFs {
    /// Normalized path → file contents
    files: HashMap<String, Vec<u8>>,
    /// Normalized directory paths that exist (computed from file paths)
    dirs: std::collections::HashSet<String>,
    /// The base directory that was embedded (used for canonicalize)
    base_dir: String,
    /// Fixed mtime for all embedded files (set at build time)
    mtime: SystemTime,
}

impl std::fmt::Debug for EmbeddedFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddedFs")
            .field("file_count", &self.files.len())
            .field("base_dir", &self.base_dir)
            .finish()
    }
}

impl EmbeddedFs {
    /// Create from a map of relative paths to file contents.
    /// Paths should use forward slashes and be relative to the app root.
    pub fn new(files: HashMap<String, Vec<u8>>, base_dir: String) -> Self {
        let mut dirs = std::collections::HashSet::new();
        // Normalize all file keys to lowercase for case-insensitive lookup
        let mut normalized_files = HashMap::new();
        for (path, data) in files {
            let normalized = Self::normalize_path_static(&path);
            let mut current = String::new();
            for segment in normalized.split('/') {
                if !current.is_empty() {
                    current.push('/');
                }
                current.push_str(segment);
                // Don't add the file itself as a dir
                if current != normalized {
                    dirs.insert(current.clone());
                }
            }
            // Also add the root dir
            dirs.insert(String::new());
            normalized_files.insert(normalized, data);
        }
        Self {
            files: normalized_files,
            dirs,
            base_dir,
            mtime: SystemTime::now(),
        }
    }

    fn normalize_path_static(path: &str) -> String {
        // Strip base_dir prefix if present, normalize separators and case
        path.replace('\\', "/")
            .trim_start_matches('/')
            .to_lowercase()
    }

    /// Normalize a path: resolve relative to base_dir, strip prefix, lowercase
    fn normalize(&self, path: &str) -> String {
        let path = path.replace('\\', "/");

        // If it starts with the base_dir, strip it
        let stripped = if !self.base_dir.is_empty() {
            let base_lower = self.base_dir.replace('\\', "/").to_lowercase();
            let path_lower = path.to_lowercase();
            if path_lower.starts_with(&base_lower) {
                let remainder = &path[self.base_dir.len()..];
                remainder.trim_start_matches('/').to_lowercase()
            } else {
                path.trim_start_matches('/').to_lowercase()
            }
        } else {
            path.trim_start_matches('/').to_lowercase()
        };

        // Clean up . and .. segments
        let mut parts: Vec<&str> = Vec::new();
        for segment in stripped.split('/') {
            match segment {
                "." | "" => {}
                ".." => { parts.pop(); }
                s => parts.push(s),
            }
        }
        parts.join("/")
    }
}

impl Vfs for EmbeddedFs {
    fn read_to_string(&self, path: &str) -> io::Result<String> {
        let normalized = self.normalize(path);
        self.files.get(&normalized)
            .map(|data| String::from_utf8_lossy(data).to_string())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound,
                format!("embedded file not found: {} (normalized: {})", path, normalized)))
    }

    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        let normalized = self.normalize(path);
        self.files.get(&normalized)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound,
                format!("embedded file not found: {} (normalized: {})", path, normalized)))
    }

    fn exists(&self, path: &str) -> bool {
        let normalized = self.normalize(path);
        self.files.contains_key(&normalized) || self.dirs.contains(&normalized)
    }

    fn is_file(&self, path: &str) -> bool {
        let normalized = self.normalize(path);
        self.files.contains_key(&normalized)
    }

    fn is_dir(&self, path: &str) -> bool {
        let normalized = self.normalize(path);
        self.dirs.contains(&normalized)
    }

    fn read_dir(&self, path: &str) -> io::Result<Vec<VfsDirEntry>> {
        let normalized = self.normalize(path);
        if !self.dirs.contains(&normalized) {
            return Err(io::Error::new(io::ErrorKind::NotFound,
                format!("embedded directory not found: {}", path)));
        }

        let prefix = if normalized.is_empty() {
            String::new()
        } else {
            format!("{}/", normalized)
        };

        let mut seen = std::collections::HashSet::new();
        let mut entries = Vec::new();

        // Find direct children (files and dirs)
        for file_path in self.files.keys() {
            if file_path.starts_with(&prefix) {
                let remainder = &file_path[prefix.len()..];
                // Direct child: no more slashes
                if let Some(slash_pos) = remainder.find('/') {
                    // It's a subdirectory entry
                    let dir_name = &remainder[..slash_pos];
                    if seen.insert(dir_name.to_string()) {
                        entries.push(VfsDirEntry {
                            name: dir_name.to_string(),
                            is_file: false,
                            is_dir: true,
                        });
                    }
                } else {
                    // Direct file child
                    entries.push(VfsDirEntry {
                        name: remainder.to_string(),
                        is_file: true,
                        is_dir: false,
                    });
                }
            }
        }

        Ok(entries)
    }

    fn modified(&self, path: &str) -> io::Result<SystemTime> {
        let normalized = self.normalize(path);
        if self.files.contains_key(&normalized) {
            Ok(self.mtime)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
        }
    }

    fn canonicalize(&self, path: &str) -> io::Result<String> {
        // For embedded fs, return the path joined with base_dir
        let normalized = self.normalize(path);
        if self.files.contains_key(&normalized) || self.dirs.contains(&normalized) {
            if self.base_dir.is_empty() {
                Ok(format!("/{}", normalized))
            } else {
                Ok(format!("{}/{}", self.base_dir, normalized))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound,
                format!("cannot canonicalize: {}", path)))
        }
    }
}

// ---------------------------------------------------------------------------
// FallbackFs — tries embedded FS first, falls back to real filesystem.
// Used in embedded binaries so they can load external files (e.g. modules).
// ---------------------------------------------------------------------------

pub struct FallbackFs {
    pub embedded: EmbeddedFs,
    pub real: RealFs,
    /// When true, only the embedded FS is used (no disk fallback).
    /// Set this in sandbox mode to prevent filesystem access.
    pub sandbox: bool,
}

impl std::fmt::Debug for FallbackFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FallbackFs")
            .field("embedded", &self.embedded)
            .finish()
    }
}

impl Vfs for FallbackFs {
    fn read_to_string(&self, path: &str) -> io::Result<String> {
        let result = self.embedded.read_to_string(path);
        if result.is_ok() || self.sandbox { return result; }
        self.real.read_to_string(path)
    }
    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        let result = self.embedded.read(path);
        if result.is_ok() || self.sandbox { return result; }
        self.real.read(path)
    }
    fn exists(&self, path: &str) -> bool {
        self.embedded.exists(path) || (!self.sandbox && self.real.exists(path))
    }
    fn is_file(&self, path: &str) -> bool {
        self.embedded.is_file(path) || (!self.sandbox && self.real.is_file(path))
    }
    fn is_dir(&self, path: &str) -> bool {
        self.embedded.is_dir(path) || (!self.sandbox && self.real.is_dir(path))
    }
    fn read_dir(&self, path: &str) -> io::Result<Vec<VfsDirEntry>> {
        if self.sandbox { return self.embedded.read_dir(path); }
        // Prefer real FS for directories (modules live on disk),
        // fall back to embedded
        self.real.read_dir(path)
            .or_else(|_| self.embedded.read_dir(path))
    }
    fn modified(&self, path: &str) -> io::Result<SystemTime> {
        let result = self.embedded.modified(path);
        if result.is_ok() || self.sandbox { return result; }
        self.real.modified(path)
    }
    fn canonicalize(&self, path: &str) -> io::Result<String> {
        let result = self.embedded.canonicalize(path);
        if result.is_ok() || self.sandbox { return result; }
        self.real.canonicalize(path)
    }
}

// ---------------------------------------------------------------------------
// Archive format for embedding files in the binary
// ---------------------------------------------------------------------------

/// Magic bytes appended at the very end of a self-contained binary.
pub const ARCHIVE_MAGIC: &[u8; 5] = b"RCFML";

/// Serialize a file map into a binary archive.
/// Format: [file_count:u32] [path_len:u32 path_bytes data_len:u32 data_bytes]...
pub fn serialize_archive(files: &HashMap<String, Vec<u8>>) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(files.len() as u32).to_le_bytes());
    for (path, data) in files {
        let path_bytes = path.as_bytes();
        buf.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(path_bytes);
        buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
        buf.extend_from_slice(data);
    }
    buf
}

/// Deserialize a binary archive into a file map.
pub fn deserialize_archive(data: &[u8]) -> io::Result<HashMap<String, Vec<u8>>> {
    let mut pos = 0;
    if data.len() < 4 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "archive too small"));
    }
    let file_count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    let mut files = HashMap::with_capacity(file_count);
    for _ in 0..file_count {
        if pos + 4 > data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "truncated archive"));
        }
        let path_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        if pos + path_len > data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "truncated path"));
        }
        let path = String::from_utf8_lossy(&data[pos..pos + path_len]).to_string();
        pos += path_len;

        if pos + 4 > data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "truncated archive"));
        }
        let data_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        if pos + data_len > data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "truncated file data"));
        }
        let file_data = data[pos..pos + data_len].to_vec();
        pos += data_len;

        files.insert(path, file_data);
    }
    Ok(files)
}

/// Check if the current binary has an embedded archive and extract it.
/// Binary layout: [original_binary][archive_data][archive_len:u64][RCFML]
pub fn extract_embedded_archive() -> Option<HashMap<String, Vec<u8>>> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_data = std::fs::read(&exe_path).ok()?;
    extract_archive_from_bytes(&exe_data)
}

/// Extract archive from raw binary bytes (testable without exe).
///
/// On macOS, `codesign` may append a code signature after our archive trailer,
/// so we scan backwards (up to 64KB) for the RCFML magic bytes.
pub fn extract_archive_from_bytes(data: &[u8]) -> Option<HashMap<String, Vec<u8>>> {
    let len = data.len();
    let min_size = ARCHIVE_MAGIC.len() + 8;
    if len < min_size {
        return None;
    }

    // Scan backwards for RCFML magic (code signature may follow it).
    // macOS code signatures scale with binary size (~8 bytes per 4KB page
    // for SHA-256 hashes, plus overhead). Use 5% of binary size or 1MB,
    // whichever is larger, to handle any realistic binary.
    let scan_window = (len / 20).max(1024 * 1024);
    let scan_limit = len.saturating_sub(scan_window).max(min_size);
    let mut magic_start = None;
    let mut pos = len - ARCHIVE_MAGIC.len();
    while pos >= scan_limit {
        if &data[pos..pos + ARCHIVE_MAGIC.len()] == ARCHIVE_MAGIC.as_slice() {
            magic_start = Some(pos);
            break;
        }
        if pos == 0 { break; }
        pos -= 1;
    }
    let magic_start = magic_start?;

    // Read archive length (u64 LE before magic)
    if magic_start < 8 {
        return None;
    }
    let len_start = magic_start - 8;
    let archive_len = u64::from_le_bytes(data[len_start..len_start + 8].try_into().ok()?) as usize;

    // Extract archive data
    if archive_len > len_start {
        return None;
    }
    let archive_start = len_start - archive_len;
    let archive_data = &data[archive_start..len_start];
    deserialize_archive(archive_data).ok()
}

/// Create a self-contained binary: append archive + length + magic to the base binary.
pub fn create_self_contained_binary(
    base_binary: &[u8],
    files: &HashMap<String, Vec<u8>>,
) -> Vec<u8> {
    let archive = serialize_archive(files);
    let archive_len = archive.len() as u64;

    let mut output = Vec::with_capacity(base_binary.len() + archive.len() + 8 + ARCHIVE_MAGIC.len());
    output.extend_from_slice(base_binary);
    output.extend_from_slice(&archive);
    output.extend_from_slice(&archive_len.to_le_bytes());
    output.extend_from_slice(ARCHIVE_MAGIC);
    output
}

/// Default VFS instance (real filesystem).
pub fn real_fs() -> Arc<dyn Vfs> {
    Arc::new(RealFs)
}
