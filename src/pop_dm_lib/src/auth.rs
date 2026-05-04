use crate::{Result, PopDMLibError};

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub username: String,
}

pub trait Authenticator {
    fn authenticate(&self, credentials: Credentials) -> Result<AuthenticatedUser>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DevAuthenticator;

impl Authenticator for DevAuthenticator {
    fn authenticate(&self, credentials: Credentials) -> Result<AuthenticatedUser> {
        if credentials.username.is_empty() {
            return Err(PopDMLibError::AuthFailed("empty username".to_string()))
        }

        if credentials.password == "dev" {
            return Ok(AuthenticatedUser { username: credentials.username });
        }
        return Err(PopDMLibError::AuthFailed("invalid user/pass combination".to_string()));
    }
}

//TODO
fn pam_start() {}
fn pam_authenticate() {}
fn pam_acct_mgmt() {}
fn pam_open_session() {}
fn pam_close_session() {}
