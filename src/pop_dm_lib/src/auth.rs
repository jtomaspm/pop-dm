use pam_client::{Context, Flag, SessionToken, conv_mock::Conversation};

use crate::{PopDMLibError, Result};

#[derive(Clone, PartialEq, Eq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub username: String,
}

impl AuthenticatedUser {
    pub fn username(&self) -> &str {
        &self.username
    }
}

pub trait Authenticator {
    fn authenticate(&self, credentials: Credentials) -> Result<AuthenticatedUser>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PamAuthenticator {
    service_name: String,
}

impl PamAuthenticator {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn open_session(&self, credentials: Credentials) -> Result<AuthenticatedSession> {
        let (mut context, user) = self.authenticate_context(credentials)?;
        let token = context.open_session(Flag::NONE)?.leak();

        Ok(AuthenticatedSession {
            user,
            context,
            token: Some(token),
        })
    }

    fn authenticate_context(
        &self,
        credentials: Credentials,
    ) -> Result<(Context<Conversation>, AuthenticatedUser)> {
        let Credentials { username, password } = credentials;
        let conversation = Conversation::with_credentials(username.clone(), password);
        let mut context = Context::new(&self.service_name, Some(&username), conversation)?;

        context.authenticate(Flag::DISALLOW_NULL_AUTHTOK)?;
        context.acct_mgmt(Flag::NONE)?;

        let username = context.user().unwrap_or(username);

        Ok((context, AuthenticatedUser { username }))
    }
}

impl Authenticator for PamAuthenticator {
    fn authenticate(&self, credentials: Credentials) -> Result<AuthenticatedUser> {
        let (_, user) = self.authenticate_context(credentials)?;
        Ok(user)
    }
}

pub struct AuthenticatedSession {
    user: AuthenticatedUser,
    context: Context<Conversation>,
    token: Option<SessionToken>,
}

impl AuthenticatedSession {
    pub fn user(&self) -> &AuthenticatedUser {
        &self.user
    }

    pub fn close(mut self) -> Result<()> {
        self.close_inner()
    }

    fn close_inner(&mut self) -> Result<()> {
        let Some(token) = self.token.take() else {
            return Ok(());
        };

        self.context
            .unleak_session(token)
            .close(Flag::NONE)
            .map_err(|err| PopDMLibError::PamAuth(err.into_without_payload()))
    }
}

impl Drop for AuthenticatedSession {
    fn drop(&mut self) {
        if let Some(token) = self.token.take() {
            drop(self.context.unleak_session(token));
        }
    }
}
