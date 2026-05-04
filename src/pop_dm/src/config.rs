use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub session_dirs: Vec<PathBuf>,
    pub default_session: Option<String>,
    pub tty: u32,
    pub failed_login_delay_seconds: u64,
    pub session_command: String,
    pub session_command_args: Vec<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            session_dirs: vec![PathBuf::from("/usr/share/wayland-sessions")],
            default_session: None,
            tty: 1,
            failed_login_delay_seconds: 2,
            session_command: "/usr/bin/niri".to_string(),
            session_command_args: vec![],
        }
    }
}
