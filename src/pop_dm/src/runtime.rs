use std::{thread, time::Duration};

use crate::{config::Config, tty};
use pop_dm_lib::{
    auth::{Credentials, PamAuthenticator},
    error::Result,
    session::{SessionCommand, run_session},
};

pub fn boot() -> Result<Config> {
    Ok(Config::default())
}

pub fn run(config: Config) -> Result<()> {
    tty::print_logo();

    let authenticator = PamAuthenticator::new("popdm");
    let session_command = SessionCommand::new(config.session_command, config.session_command_args);

    loop {
        let username = tty::prompt_line("login: ")?;
        let password = tty::prompt_password("password: ", config.password_feedback)?;
        let credentials = Credentials { username, password };

        match authenticator.open_session(credentials) {
            Ok(session) => {
                println!("starting session for {}", session.user().username());
                println!(
                    "session exited with status: {}",
                    run_session(&session_command)?
                );
                session.close()?;
            }
            Err(err) => {
                eprintln!("login failed: {}", err);
                thread::sleep(Duration::from_secs(config.failed_login_delay_seconds));
            }
        }
    }
}

pub fn boot_and_run() -> Result<()> {
    return run(boot()?);
}
