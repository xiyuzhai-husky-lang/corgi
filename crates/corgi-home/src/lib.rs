use std::io;
use std::path::{Path, PathBuf};

/// Returns the storage directory used by Corgi within `cwd`.
/// For more details, see [`corgi_home`](fn.corgi_home.html).
pub fn corgi_home_with_cwd(cwd: &Path) -> io::Result<PathBuf> {
    match std::env::var_os("CORGI_HOME").filter(|h| !h.is_empty()) {
        Some(home) => {
            let home = PathBuf::from(home);
            if home.is_absolute() {
                Ok(home)
            } else {
                Ok(cwd.join(&home))
            }
        }
        _ => directories::UserDirs::new()
            .map(|p| p.home_dir().join(".corgi"))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "could not find corgi home dir")),
    }
}
#[cfg(test)]
mod tests {}
