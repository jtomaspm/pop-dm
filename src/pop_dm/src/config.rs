use std::path::PathBuf;

pub struct Config {
    pub session_dirs: Vec<PathBuf>,
    pub default_session: Option<String>,
    pub tty: u32,
    pub failed_login_delay_seconds: u64,
}

impl Config {
    pub(crate) fn default() -> Config {
        Config {
            session_dirs: vec![PathBuf::from("/usr/share/wayland-sessions")],
            default_session: None,
            tty: 1,
            failed_login_delay_seconds: 2,
        }
    }
}
