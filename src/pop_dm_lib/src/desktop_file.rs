use crate::error::Result;
use std::path::PathBuf;

pub struct DesktopFile {
    pub name: String,
    pub comment: Option<String>,
    pub exec: String,
    pub desktop_names: Vec<String>,
}

pub fn discover_wayland_sessions(_dirs: &[PathBuf]) -> Result<Vec<DesktopFile>> {
    Ok(vec![])
}
