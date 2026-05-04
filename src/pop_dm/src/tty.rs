use pop_dm_lib::{PopDMLibError, Result};
use rpassword::{ConfigBuilder};
use std::{io::{self, Write}};

pub fn print_logo() {
    println!("Welcome to pop_dm!");
}

pub fn prompt_line(label: &str) -> Result<String> {
    print!("{label}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    return Ok(input.trim().to_string());
}

pub fn prompt_password(label: &str) -> Result<String> {
    let config = ConfigBuilder::new().password_feedback_mask('*').build();
    return match rpassword::prompt_password_with_config(label, config) {
        Ok(res) => Ok(res),
        Err(err) => Err(PopDMLibError::Io(err)),
    };
}
