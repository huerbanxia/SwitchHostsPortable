//! Portable ("green") mode path resolution.
//!
//! A portable build keeps every piece of writable user data inside the
//! green-software directory it lives in — conventionally `App/` holds the
//! read-only program body and `Data/` holds the user data, matching the
//! PortableApps-style layout (e.g. the qBittorrent Enhanced portable
//! distribution). Nothing is written to the user profile (`%APPDATA%`,
//! `%LOCALAPPDATA%`, `%TEMP%`, `~/.SwitchHosts`), so copying the whole
//! directory is a complete backup and a reinstalled system never loses
//! the configuration location.
//!
//! Two signals turn portable mode on (OR):
//!   1. the `SWITCHHOSTS_PORTABLE` environment variable is set (to any
//!      non-empty value), or
//!   2. a `.portable` marker file exists in the executable's own
//!      directory or up to `MAX_ANCESTOR_DEPTH` levels above it.
//!
//! When active, the data root is fixed at `<green-root>/Data/SwitchHosts`
//! and logs go to `<green-root>/Data/Logs`. The green root is the directory
//! that directly contains `App/` and `Data/` — i.e. two levels up from the
//! executable at `<green-root>/App/SwitchHosts/SwitchHosts.exe`.
//!
//! Every function here is a pure function over explicit inputs so it can be
//! unit tested without touching globals. If none of the portable signals
//! applies, portable resolution returns `None` and the installer behaviour
//! (in `paths` / `data_dir_pointer`) is left completely unchanged.

use std::path::{Path, PathBuf};

/// Environment variable that switches portable mode on.
pub const PORTABLE_ENV: &str = "SWITCHHOSTS_PORTABLE";

/// Portable marker file/directory name.
pub const PORTABLE_MARKER: &str = ".portable";

/// How many ancestor levels above the executable directory we search for
/// the portable marker. The exe lives at `<green>/App/SwitchHosts/`, so we
/// look at itself (level 0), `App` (1), and the green root (2).
pub const MAX_ANCESTOR_DEPTH: usize = 3;

/// Writable sub-directory of the green root that holds user data.
pub const DATA_DIR: &str = "Data";

/// The exact data root under `Data/`, mirroring the installer's
/// `~/.SwitchHosts` semantics but scoped to the green directory.
pub const DATA_ROOT_NAME: &str = "SwitchHosts";

/// Logs sub-directory under `Data/`.
pub const LOG_DIR_NAME: &str = "Logs";

/// Whether a `.portable` marker is present at or above `exe_dir`, the
/// `SWITCHHOSTS_PORTABLE` env var is set, **or** `exe_dir` sits inside the
/// PortableApps-style layout (`App/` + `Data/` siblings).
///
/// `exe_dir` is the directory containing the executable (or a cached value
/// of it; `None` if it cannot be resolved). `env_override` should pass a
/// cached `std::env::var(PORTABLE_ENV)`; a `Some(Some(_))` with any
/// non-empty value forces portable mode on, exactly like marking `.portable`.
///
/// Checking the layout is what makes detection robust regardless of which
/// exe is launched: the top-level `SwitchHostsPortable.exe` sits at the
/// green root (next to `Data/`), while `App/SwitchHosts/SwitchHosts.exe`
/// sits two levels down. Both are inside the green layout.
pub fn is_portable(exe_dir: Option<&Path>, env_override: Option<Option<String>>) -> bool {
    if env_override
        .flatten()
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        return true;
    }
    let dir = match exe_dir {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        _ => return false,
    };
    let mut current = dir.to_path_buf();
    for _ in 0..=MAX_ANCESTOR_DEPTH {
        // A portable marker in this directory.
        if current.join(PORTABLE_MARKER).exists() {
            return true;
        }
        // The green layout: this directory directly contains `App/` and
        // `Data/` (the PortableApps convention), so it is a green root.
        if current.join(DATA_DIR).is_dir() && current.join("App").is_dir() {
            return true;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    false
}

/// Locate the green root (the directory that directly contains `App/` and
/// `Data/`) from the executable directory.
///
/// The green root is defined as the nearest ancestor (at or above `exe_dir`)
/// that has a `Data/` sub-directory directly inside it — the layout shipped
/// by the packaging script. This is robust to where exactly the exe lives:
///   - `SwitchHostsPortable.exe` at the green root (depth 0), or
///   - `App/SwitchHosts/SwitchHosts.exe` (depth 2).
///
/// Returns `None` when no ancestor with a `Data/` sibling is found within
/// `MAX_ANCESTOR_DEPTH`, i.e. the layout is not a portable green directory.
pub fn green_root(exe_dir: &Path) -> Option<PathBuf> {
    let dir = if exe_dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        exe_dir
    };
    let mut current = dir.to_path_buf();
    for _ in 0..=MAX_ANCESTOR_DEPTH {
        if current.join(DATA_DIR).is_dir() {
            return Some(current);
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    None
}

/// The fixed portable default data root: `<green>/Data/SwitchHosts`.
pub fn default_data_root(exe_dir: &Path) -> Option<PathBuf> {
    green_root(exe_dir).map(|root| root.join(DATA_DIR).join(DATA_ROOT_NAME))
}

/// The portable log directory: `<green>/Data/Logs`.
pub fn portable_log_dir(exe_dir: &Path) -> Option<PathBuf> {
    green_root(exe_dir).map(|root| root.join(DATA_DIR).join(LOG_DIR_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "swh-portable-test-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn env_override_forces_portable() {
        let dir = tmp("env");
        assert!(is_portable(Some(&dir), Some(Some("1".into()))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_env_value_not_portable() {
        let dir = tmp("env-empty");
        assert!(!is_portable(Some(&dir), Some(Some(String::new()))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn env_absent_check() {
        let dir = tmp("noenv");
        assert!(!is_portable(Some(&dir), Some(None)));
        assert!(!is_portable(None, Some(None)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn marker_in_own_dir_enables_portable() {
        let green = tmp("green-own");
        let marker = green.join(PORTABLE_MARKER);
        std::fs::write(&marker, b"").unwrap();
        let exe = green.join("App/SwitchHosts");
        std::fs::create_dir_all(&exe).unwrap();
        assert!(is_portable(Some(&exe), None));
        let _ = std::fs::remove_dir_all(&green);
    }

    #[test]
    fn marker_at_green_root_enables_portable() {
        let green = tmp("green-root");
        let exe = green.join("App/SwitchHosts");
        std::fs::create_dir_all(&exe).unwrap();
        let marker = green.join(PORTABLE_MARKER);
        std::fs::write(&marker, b"").unwrap();
        // exe at green/App/SwitchHosts; green is 2 levels up; within MAX_ANCESTOR_DEPTH.
        assert!(is_portable(Some(&exe), None));
        let _ = std::fs::remove_dir_all(&green);
    }

    #[test]
    fn no_marker_not_portable() {
        let green = tmp("green-no-marker");
        let exe = green.join("App/SwitchHosts");
        std::fs::create_dir_all(&exe).unwrap();
        assert!(!is_portable(Some(&exe), None));
        let _ = std::fs::remove_dir_all(&green);
    }

    #[test]
    fn layout_app_data_detects_portable_even_without_marker() {
        // PortableApps layout: `App/` + `Data/` as siblings. No marker file.
        // Launching from the green-root launcher (depth 0) must still detect
        // portable mode by layout alone.
        let green = tmp("green-layout");
        std::fs::create_dir_all(green.join("App/SwitchHosts")).unwrap();
        std::fs::create_dir_all(green.join(DATA_DIR)).unwrap();
        let launcher = green.join("SwitchHostsPortable.exe");
        assert!(is_portable(Some(&launcher), None));
        let inner = green.join("App/SwitchHosts");
        assert!(is_portable(Some(&inner), None));
        let _ = std::fs::remove_dir_all(&green);
    }

    #[test]
    fn green_root_resolves_two_levels_up() {
        // App binary at <green>/App/SwitchHosts; Data/ is a sibling of App/.
        let green = tmp("green-resolve");
        let exe = green.join("App/SwitchHosts");
        std::fs::create_dir_all(&exe).unwrap();
        std::fs::create_dir_all(green.join(DATA_DIR)).unwrap();
        assert_eq!(green_root(&exe), Some(green.clone()));
        assert_eq!(
            default_data_root(&exe),
            Some(green.join(DATA_DIR).join(DATA_ROOT_NAME))
        );
        assert_eq!(
            portable_log_dir(&exe),
            Some(green.join(DATA_DIR).join(LOG_DIR_NAME))
        );
        let _ = std::fs::remove_dir_all(&green);
    }

    #[test]
    fn green_root_resolves_from_top_level_launcher_exe() {
        // `SwitchHostsPortable.exe` sits at the green root itself (depth 0)
        // with Data/ directly beside it.
        let green = tmp("green-launcher");
        std::fs::create_dir_all(green.join(DATA_DIR)).unwrap();
        let launcher = green.join("SwitchHostsPortable.exe");
        assert_eq!(green_root(&launcher), Some(green.clone()));
        assert_eq!(
            default_data_root(&launcher),
            Some(green.join(DATA_DIR).join(DATA_ROOT_NAME))
        );
        let _ = std::fs::remove_dir_all(&green);
    }

    #[test]
    fn green_root_none_without_data_sibling() {
        // No Data/ anywhere in the ancestor chain -> not a portable layout.
        let green = tmp("green-nodata");
        let exe = green.join("App/SwitchHosts");
        std::fs::create_dir_all(&exe).unwrap();
        assert_eq!(green_root(&exe), None);
        assert_eq!(default_data_root(&exe), None);
        assert_eq!(portable_log_dir(&exe), None);
        let _ = std::fs::remove_dir_all(&green);
    }

    #[test]
    fn exotic_shallow_exe_has_no_green_root() {
        // A single-component path has no parent chain -> no green root.
        assert_eq!(green_root(Path::new("SwitchHosts.exe")), None);
        assert_eq!(default_data_root(Path::new("SwitchHosts.exe")), None);
    }
}
