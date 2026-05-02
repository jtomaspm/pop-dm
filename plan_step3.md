# Step 3 Deep Plan: Build the Minimal TTY Login Manager

## Goal

Build the first real version of `pop_dm`: a text-based login manager that runs on a TTY, asks for a username and password, authenticates the user, and launches one fixed session command such as `sway`.

This is the first step where the project starts behaving like a display manager.

By the end of Step 3:

1. `pop_dm` runs from a terminal or TTY.
2. It prints a simple text banner.
3. It prompts for a username.
4. It prompts for a password without echoing it.
5. It calls the authentication boundary in `pop_dm_lib::auth`.
6. It launches a fixed configured session command after successful login.
7. It waits until that session exits.
8. It returns to the login prompt.
9. Failed logins show a generic failure message and wait briefly before retrying.

Step 3 can be split into two tracks:

1. **Step 3A:** fake authentication so you can build the TTY loop and session launching safely.
2. **Step 3B:** replace fake authentication with real PAM.

That split is useful because PAM and privilege/session handling are sharp tools. Build the loop first, then make authentication real.

## How You Did On Step 2

Overall: good start. You created the right shape.

What is already good:

1. The workspace split is correct: `pop_dm` for the binary, `pop_dm_lib` for core logic.
2. `main.rs` is clean and only handles top-level error printing.
3. `boot.rs` is a nice orchestration layer.
4. `config.rs` has the important first defaults.
5. `error.rs` uses `thiserror`, which is a good choice.
6. `auth.rs`, `user.rs`, and `desktop_file.rs` establish the right boundaries.
7. Renaming `desktop_boot` to `desktop_file` was a good instinct.

Checks run from `src/`:

```sh
cargo check
cargo test
cargo clippy --all-targets --all-features
```

Result:

1. `cargo check` passes.
2. `cargo test` passes, though there are no tests yet.
3. `cargo clippy` passes, but reports warnings.
4. `cargo fmt --check` fails because formatting has not been applied.

Before starting real Step 3 code, clean these up:

1. Run `cargo fmt`.
2. Remove needless `return` from `boot.rs`.
3. Use `tty::print_logo()` inside `boot::run()` so it is not dead code.
4. Either use `config` in `run(config: Config)` or rename it to `_config` temporarily.
5. Remove the empty private PAM placeholder functions from `auth.rs`, or turn them into comments. Empty unused functions create noise.
6. Re-export the common result type from `lib.rs`:

```rust
pub use error::{PopDMLibError, Result};
```

Optional but recommended:

1. Implement `Default` for `Config` instead of an inherent `Config::default()` method.
2. Add `#[derive(Debug, Clone)]` to simple data structs.
3. Add one or two tiny tests so `cargo test` means something.

## Step 3 Safety Rule

Do not test this first on your main login flow.

Use this order:

1. Build and test from a normal terminal.
2. Test with a harmless command like `/usr/bin/env` or `/bin/sh`.
3. Test on a spare TTY.
4. Test in a VM.
5. Only much later consider boot integration.

For Step 3, do not install a systemd service yet.

## Step 3 Architecture

The login manager should look like this:

```txt
main()
  -> boot::boot_and_run()
      -> load/default Config
      -> run login loop
          -> print banner
          -> prompt username
          -> prompt password
          -> authenticate
          -> build fixed session command
          -> launch session
          -> wait for session exit
          -> loop again
```

Keep these boundaries:

```txt
pop_dm/src/tty.rs
  user-facing terminal prompts

pop_dm/src/boot.rs
  login loop orchestration

pop_dm/src/config.rs
  defaults and app config

pop_dm_lib/src/auth.rs
  authentication API

pop_dm_lib/src/session.rs
  session command and process launching

pop_dm_lib/src/user.rs
  user model and later UID/GID lookup
```

## Step 3A: Build The TTY Loop With Fake Auth

Start here. It gives you fast progress without root/PAM complexity.

### 3A.1 Add Login Prompt Types

In `pop_dm/src/tty.rs`, add small prompt helpers:

```rust
use std::io::{self, Write};

pub fn print_logo() {
    println!("Welcome to pop_dm!");
}

pub fn prompt_line(label: &str) -> io::Result<String> {
    print!("{label}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_owned())
}
```

Then add password input.

Suggested dependency in `src/pop_dm/Cargo.toml`:

```toml
rpassword = "7"
```

Then:

```rust
pub fn prompt_password(label: &str) -> io::Result<String> {
    rpassword::prompt_password(label)
}
```

Why `rpassword`:

1. It hides terminal echo for you.
2. It restores terminal state on normal errors.
3. It lets you postpone raw terminal handling.

### 3A.2 Add Config For A Fixed Session

In `pop_dm/src/config.rs`, add a fixed command for now:

```rust
pub struct Config {
    pub session_dirs: Vec<PathBuf>,
    pub default_session: Option<String>,
    pub tty: u32,
    pub failed_login_delay_seconds: u64,
    pub fixed_session_command: Vec<String>,
}
```

Default:

```rust
fixed_session_command: vec!["sway".to_owned()],
```

For safer early tests, you can temporarily use:

```rust
fixed_session_command: vec!["/usr/bin/env".to_owned()],
```

or:

```rust
fixed_session_command: vec!["/bin/sh".to_owned()],
```

Do not parse config files yet. Just hardcode defaults.

### 3A.3 Define Session Command

In `pop_dm_lib/src/session.rs`:

```rust
use crate::Result;
use std::process::{Command, ExitStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl SessionCommand {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }

    pub fn from_words(words: &[String]) -> Result<Self> {
        let Some((program, args)) = words.split_first() else {
            return Err(crate::PopDMLibError::InvalidSession(
                "session command is empty".to_owned(),
            ));
        };

        Ok(Self {
            program: program.clone(),
            args: args.to_vec(),
        })
    }
}
```

You will need an `InvalidSession` variant in `error.rs`:

```rust
#[error("invalid session: {0}")]
InvalidSession(String),
```

### 3A.4 Launch A Session Without Privilege Dropping Yet

Still in `session.rs`:

```rust
pub fn run_session(command: &SessionCommand) -> Result<ExitStatus> {
    let mut child = Command::new(&command.program)
        .args(&command.args)
        .spawn()?;

    Ok(child.wait()?)
}
```

This does not make a real display-manager user session yet. That is fine for Step 3A.

This lets you test the control flow:

1. prompt
2. authenticate
3. spawn command
4. wait
5. return to prompt

Privilege dropping comes later.

### 3A.5 Add Fake Authentication

In `pop_dm_lib/src/auth.rs`, define the API first:

```rust
use crate::Result;

#[derive(Debug, Clone)]
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
```

Then add a dev-only authenticator:

```rust
#[derive(Debug, Default, Clone, Copy)]
pub struct DevAuthenticator;

impl Authenticator for DevAuthenticator {
    fn authenticate(&self, credentials: Credentials) -> Result<AuthenticatedUser> {
        if credentials.username.is_empty() {
            return Err(crate::PopDMLibError::AuthFailed);
        }

        if credentials.password == "dev" {
            Ok(AuthenticatedUser {
                username: credentials.username,
            })
        } else {
            Err(crate::PopDMLibError::AuthFailed)
        }
    }
}
```

Add error variant:

```rust
#[error("authentication failed")]
AuthFailed,
```

Important:

Do not keep this as the final auth system. It is only a scaffold for the login loop.

### 3A.6 Write The Login Loop

In `pop_dm/src/boot.rs`, make `run()` actually orchestrate:

```rust
use crate::config::Config;
use crate::tty;
use pop_dm_lib::auth::{Authenticator, Credentials, DevAuthenticator};
use pop_dm_lib::session::{run_session, SessionCommand};
use pop_dm_lib::Result;
use std::thread;
use std::time::Duration;

pub fn boot() -> Result<Config> {
    Ok(Config::default())
}

pub fn run(config: Config) -> Result<()> {
    tty::print_logo();

    let authenticator = DevAuthenticator;
    let session_command = SessionCommand::from_words(&config.fixed_session_command)?;

    loop {
        let username = tty::prompt_line("login: ")?;
        let password = tty::prompt_password("password: ")?;

        let credentials = Credentials { username, password };

        match authenticator.authenticate(credentials) {
            Ok(user) => {
                println!("starting session for {}", user.username);
                let status = run_session(&session_command)?;
                println!("session exited with: {status}");
            }
            Err(_) => {
                eprintln!("login failed");
                thread::sleep(Duration::from_secs(
                    config.failed_login_delay_seconds,
                ));
            }
        }
    }
}

pub fn boot_and_run() -> Result<()> {
    run(boot()?)
}
```

This loop is intentionally simple.

Test it with password:

```txt
dev
```

### 3A.7 Avoid Logging Passwords

Even in fake auth:

1. never print the password
2. never include it in debug logs
3. do not derive `Debug` for `Credentials` if it contains `password`

Better:

```rust
pub struct Credentials {
    pub username: String,
    pub password: String,
}
```

Do not derive `Debug` here.

## Step 3B: Replace Fake Auth With PAM

Once the loop works, move to PAM.

This is the hard part of Step 3. If it gets too big, it is okay to make PAM the start of Step 4.

### 3B.1 Choose PAM Approach

You have two options:

1. Use a PAM crate.
2. Use direct FFI to `libpam`.

Recommended for learning:

1. Try a crate first.
2. If the crate feels too limiting, switch to direct FFI later.
3. Keep the rest of your app behind the `Authenticator` trait either way.

The app should not care whether auth is fake, crate-backed PAM, or direct FFI PAM.

### 3B.2 PAM Service Name

Eventually your service name should be:

```txt
popdm
```

That means PAM will look for:

```txt
/etc/pam.d/popdm
```

For early testing, you may use an existing service like:

```txt
login
```

But be careful: different PAM service files behave differently.

### 3B.3 PAM Authenticator Shape

In `auth.rs`, the final-ish shape can be:

```rust
pub struct PamAuthenticator {
    service_name: String,
}

impl PamAuthenticator {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }
}

impl Authenticator for PamAuthenticator {
    fn authenticate(&self, credentials: Credentials) -> Result<AuthenticatedUser> {
        // PAM implementation goes here.
        todo!()
    }
}
```

Keep PAM internals private.

### 3B.4 PAM Calls You Need

Minimum real PAM flow:

```txt
pam_start
pam_authenticate
pam_acct_mgmt
pam_open_session
launch session
pam_close_session
```

For Step 3, you can start with:

```txt
pam_start
pam_authenticate
pam_acct_mgmt
```

Then move `pam_open_session` and `pam_close_session` into the session lifecycle.

Why:

Authentication answers: “is this user allowed?”

Opening a session answers: “create the login/session environment for this user.”

Those are related, but not the same.

### 3B.5 Password Handling

The cleanest PAM design is:

1. `tty.rs` prompts for password.
2. `auth.rs` passes that password to the PAM conversation.
3. `auth.rs` drops the password as soon as auth completes.
4. nothing logs it
5. nothing stores it in config or session structs

Later, you can improve this by letting PAM drive the conversation prompts directly.

For now, a password-supplying conversation is enough.

## Step 3C: User Lookup

After auth succeeds, look up the user.

In `user.rs`, add:

```rust
pub fn lookup_user(username: &str) -> Result<LoginUser> {
    todo!()
}
```

Recommended dependency:

```toml
nix = { version = "0.30", features = ["user"] }
```

The lookup should fill:

1. username
2. UID
3. primary GID
4. home directory
5. shell

Do not drop privileges yet unless you are ready to test carefully.

For Step 3, the first use of this function can simply print:

```txt
authenticated user pop uid=1000 gid=1000
```

## Step 3D: Fixed Session Launching

Your first session launching can be simple:

```rust
Command::new("sway").spawn()?.wait()?
```

But structure it as `SessionCommand` now so later `.desktop` parsing can reuse it.

Important rule:

Do not use a shell:

```rust
Command::new("sh").arg("-c").arg(exec)
```

Avoid that for this project unless you intentionally implement shell behavior. Shelling out creates injection risks and makes `.desktop` parsing more dangerous.

## Step 3E: Environment Basics

For Step 3, do not try to fully construct a perfect Wayland user environment.

But start learning the fields you will eventually set:

```txt
HOME
USER
LOGNAME
SHELL
PATH
XDG_SESSION_TYPE=wayland
XDG_SESSION_CLASS=user
XDG_SESSION_DESKTOP
XDG_CURRENT_DESKTOP
```

The earliest safe version can inherit your current environment when testing from a terminal.

When testing from a real TTY/display-manager context, environment handling becomes much more important.

## Step 3F: Failed Login Behavior

Failed login should be boring and generic:

```txt
login failed
```

Do not say:

```txt
user does not exist
wrong password
account expired
```

Why:

You do not want to leak which usernames are valid.

Use:

```rust
thread::sleep(Duration::from_secs(config.failed_login_delay_seconds));
```

Later, add stronger rate limiting.

## Step 3G: Manual Test Script

Start with harmless commands.

Set fixed command to:

```txt
/usr/bin/env
```

Expected behavior:

1. `pop_dm` prompts for login.
2. password `dev` succeeds in fake auth.
3. it prints environment.
4. it returns to login prompt.

Then test:

```txt
/bin/sh
```

Expected behavior:

1. `pop_dm` prompts for login.
2. password `dev` succeeds.
3. shell opens.
4. type `exit`.
5. `pop_dm` returns to login prompt.

Only after that test:

```txt
sway
```

Use a VM or spare TTY.

## Step 3H: Tests To Add

Good unit tests:

1. `SessionCommand::from_words` rejects empty commands.
2. `SessionCommand::from_words` accepts `["sway"]`.
3. `SessionCommand::from_words` accepts `["sway", "--debug"]`.
4. `DevAuthenticator` accepts non-empty username with password `dev`.
5. `DevAuthenticator` rejects wrong password.
6. `Config::default()` has a non-empty fixed session command.

Example:

```rust
#[test]
fn session_command_rejects_empty_words() {
    let err = SessionCommand::from_words(&[]).unwrap_err();
    assert!(err.to_string().contains("session command is empty"));
}
```

Do not write automated tests that depend on real PAM yet. PAM tests are integration tests and can vary by distro.

## Step 3 Done Checklist

You are done with Step 3A when:

1. `cargo fmt` passes.
2. `cargo check` passes.
3. `cargo test` passes.
4. `cargo clippy --all-targets --all-features` has no surprising warnings.
5. The app prompts for username.
6. The app prompts for hidden password.
7. Fake auth succeeds with a known dev password.
8. Fake auth fails generically.
9. A fixed command launches after successful fake auth.
10. The app waits for the command to exit.
11. The app returns to the login prompt.

You are done with Step 3B when:

1. PAM authentication works for a real local user.
2. wrong passwords fail.
3. nonexistent users fail generically.
4. password text is never printed.
5. auth logic is hidden behind the `Authenticator` trait.

You are done with full Step 3 when:

1. fake auth is removed or clearly feature-gated for development only.
2. real auth can launch a harmless fixed command.
3. the control flow is stable enough to begin proper PAM session handling in Step 4.

## Recommended Implementation Order

Use this exact order:

1. Fix Step 2 formatting and warnings.
2. Add `InvalidSession` and `AuthFailed` error variants.
3. Add `SessionCommand`.
4. Add `run_session`.
5. Add prompt helpers in `tty.rs`.
6. Add `rpassword`.
7. Add `fixed_session_command` to `Config`.
8. Add `Authenticator` trait.
9. Add `DevAuthenticator`.
10. Wire the login loop in `boot.rs`.
11. Test with `/usr/bin/env`.
12. Test with `/bin/sh`.
13. Add unit tests.
14. Start PAM implementation only after the loop feels solid.

## The Mental Model

Step 2 made boxes.

Step 3 makes the first wire run through those boxes:

```txt
TTY input -> credentials -> auth -> session command -> child process -> wait -> login prompt
```

Do not make it fancy. Make it understandable, testable, and hard to accidentally make unsafe.
