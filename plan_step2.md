# Step 2 Deep Plan: Shape the Rust Project

## Goal

Turn the repo from a Cargo skeleton into a clean Rust project shape for a text-based display manager.

This step is not about logging in yet. It is about creating the boundaries that will make login, PAM, session launching, and TTY work easier to build without everything ending up in `main.rs`.

By the end of this step:

1. The workspace builds with `cargo check`.
2. `pop_dm` is the small binary crate.
3. `pop_dm_lib` contains the core display-manager logic.
4. Each module has a clear job.
5. The placeholder `add()` example is gone.
6. The program can start, load/default a config, print a startup message, and exit cleanly.

## Current Repo Shape

You already have this useful split:

```txt
src/
  Cargo.toml
  pop_dm/
    Cargo.toml
    src/
      main.rs
      config.rs
      tty.rs
  pop_dm_lib/
    Cargo.toml
    src/
      lib.rs
      error.rs
      user.rs
      desktop_boot.rs
      session.rs
      auth.rs
```

Keep this split.

Use `pop_dm` for command-line app behavior:

1. reading config paths
2. choosing interactive vs later daemon/service mode
3. printing user-facing messages
4. wiring modules together
5. owning the top-level `main()`

Use `pop_dm_lib` for reusable core behavior:

1. error types
2. user lookup
3. authentication API
4. session model
5. desktop/session discovery
6. process launching
7. data structures that can be tested without a real TTY

## Design Rule

Keep `main.rs` boring.

The binary should look like orchestration:

```rust
fn main() {
    if let Err(err) = pop_dm::run() {
        eprintln!("popdm: {err}");
        std::process::exit(1);
    }
}
```

Most real logic should live in functions that return `Result`.

## Step 2.1: Fix Workspace Command Habits

The workspace root is:

```txt
src/Cargo.toml
```

So most Rust commands should run from `src/`:

```sh
cd src
cargo check
cargo test
cargo fmt
cargo clippy --all-targets --all-features
```

Later, you can add a root script if you want:

```sh
scripts/run.sh
```

For now, keep the commands simple.

## Step 2.2: Decide Module Ownership

Use this ownership map.

### `pop_dm/src/main.rs`

Job:

1. call a `run()` function
2. print top-level fatal errors
3. exit with nonzero status on failure

Avoid:

1. PAM logic
2. UID/GID switching
3. `.desktop` parsing
4. child process handling

### `pop_dm/src/config.rs`

Job:

1. define app-level config
2. provide safe defaults
3. later load TOML from disk

First useful type:

```rust
pub struct Config {
    pub session_dirs: Vec<PathBuf>,
    pub default_session: Option<String>,
    pub tty: Option<u32>,
    pub failed_login_delay_seconds: u64,
}
```

For step 2, only implement `Default`.

Do not add TOML parsing yet unless you want a small optional stretch.

### `pop_dm/src/tty.rs`

Job:

1. later read username
2. later read password without echo
3. later render session choices

For step 2, keep this tiny:

```rust
pub fn print_banner() {
    println!("popdm text display manager");
}
```

The real TTY behavior comes in step 3.

### `pop_dm_lib/src/lib.rs`

Job:

1. expose the library modules
2. re-export common result/error types
3. remove the default `add()` example

Shape:

```rust
pub mod auth;
pub mod desktop_boot;
pub mod error;
pub mod session;
pub mod user;

pub use error::{Error, Result};
```

### `pop_dm_lib/src/error.rs`

Job:

1. define one shared library error type
2. keep error messages readable
3. convert common errors with `From`

Start simple. You do not need `thiserror` yet, but it is nice once errors grow.

Dependency option:

```toml
thiserror = "2"
```

First useful shape:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("user not found: {0}")]
    UserNotFound(String),

    #[error("invalid session: {0}")]
    InvalidSession(String),

    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

If you do not want dependencies yet, implement `Display` and `std::error::Error` manually.

### `pop_dm_lib/src/user.rs`

Job:

1. represent a login user
2. later look up UID/GID/home/shell
3. later initialize groups and drop privileges

First useful type:

```rust
pub struct LoginUser {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub home: PathBuf,
    pub shell: PathBuf,
}
```

For step 2, you can define the type without real lookup.

In step 3 or 4, add real lookup using:

1. `nix`
2. `users`
3. direct libc calls, wrapped carefully

### `pop_dm_lib/src/auth.rs`

Job:

1. define the authentication boundary
2. later implement PAM behind this boundary

For step 2, do not wire PAM yet.

Define the shape:

```rust
pub struct Credentials {
    pub username: String,
}

pub struct AuthenticatedUser {
    pub username: String,
}
```

Later, this module will own:

1. `pam_start`
2. `pam_authenticate`
3. `pam_acct_mgmt`
4. `pam_open_session`
5. `pam_close_session`

The important design idea is that the rest of the program should not know the details of PAM.

### `pop_dm_lib/src/session.rs`

Job:

1. represent the selected session
2. later launch the session process
3. later wait for the session to exit

First useful types:

```rust
pub struct SessionCommand {
    pub program: String,
    pub args: Vec<String>,
}

pub struct SessionExit {
    pub code: Option<i32>,
}
```

For step 2, this module can just hold types.

Do not implement privilege dropping yet.

### `pop_dm_lib/src/desktop_boot.rs`

Job:

1. represent installed Wayland sessions
2. later parse `.desktop` files
3. later convert a selected entry into a `SessionCommand`

Consider renaming this module later to one of these:

1. `desktop_entry`
2. `session_discovery`
3. `sessions`

`desktop_boot` is not wrong, but `desktop_entry` is clearer if this module is mostly about reading `/usr/share/wayland-sessions/*.desktop`.

First useful type:

```rust
pub struct DesktopSession {
    pub name: String,
    pub comment: Option<String>,
    pub exec: String,
    pub desktop_names: Vec<String>,
}
```

For step 2, you can define the type and add a fake discovery function:

```rust
pub fn discover_wayland_sessions(_dirs: &[PathBuf]) -> Result<Vec<DesktopSession>> {
    Ok(Vec::new())
}
```

Real parsing comes later.

## Step 2.3: Add Minimal Dependencies

Keep dependencies light right now.

Suggested `pop_dm_lib/Cargo.toml`:

```toml
[dependencies]
thiserror = "2"
```

Suggested `pop_dm/Cargo.toml`:

```toml
[dependencies]
pop_dm_lib = { path = "../pop_dm_lib" }
```

Do not add PAM, `nix`, TOML, logging, or desktop-entry parsing until you actually start using them.

Good order for later dependencies:

1. `thiserror`
2. `nix`
3. `rpassword`
4. PAM crate or direct PAM FFI
5. `serde` and `toml`
6. desktop entry parser
7. `tracing`

## Step 2.4: Create a Thin App Runner

Instead of putting everything in `main.rs`, make a runner function.

Possible shape:

```txt
pop_dm/src/main.rs
pop_dm/src/config.rs
pop_dm/src/tty.rs
```

Since binary crates do not automatically expose modules to themselves through a separate library file, `main.rs` can declare:

```rust
mod config;
mod tty;
```

Then:

```rust
use config::Config;

fn main() {
    if let Err(err) = run() {
        eprintln!("popdm: {err}");
        std::process::exit(1);
    }
}

fn run() -> pop_dm_lib::Result<()> {
    let config = Config::default();
    tty::print_banner();
    println!("session dirs: {:?}", config.session_dirs);
    Ok(())
}
```

That gives you a working app frame without pretending login exists yet.

## Step 2.5: Add First Tests

Do not wait too long to add tests. Step 2 only needs tiny ones.

Good first tests:

1. `Config::default()` includes `/usr/share/wayland-sessions`.
2. `SessionCommand` can represent `sway`.
3. `discover_wayland_sessions` returns an empty list for now.
4. error messages display cleanly.

Example:

```rust
#[test]
fn default_config_has_wayland_session_dir() {
    let config = Config::default();
    assert!(config
        .session_dirs
        .iter()
        .any(|path| path == Path::new("/usr/share/wayland-sessions")));
}
```

The point is not coverage yet. The point is forcing the project shape to compile and stay honest.

## Step 2.6: Run Checks

From `src/`:

```sh
cargo fmt
cargo check
cargo test
cargo clippy --all-targets --all-features
```

Expected result:

1. no placeholder `add()` code
2. no dead imports
3. no broken module declarations
4. no warnings you do not understand

Some dead-code warnings are acceptable while you are laying out types, but prefer to use small tests to exercise the types.

## Suggested Commit for Step 2

Commit message:

```txt
Shape Rust workspace for text display manager
```

Commit should include:

1. module declarations in `pop_dm_lib`
2. shared error type
3. config defaults
4. initial user/session/desktop/auth data types
5. thin `main.rs`
6. first tiny tests

## What Not to Build Yet

Do not implement these in step 2:

1. real PAM login
2. password prompting
3. UID/GID switching
4. launching Sway
5. parsing real `.desktop` files
6. systemd service files
7. logging architecture
8. graphical UI

Those are real work, but they belong to later steps.

Step 2 succeeds when the project is shaped well enough that step 3 can focus only on the TTY login loop.

## Step 2 Done Checklist

1. `src/pop_dm_lib/src/lib.rs` exports real modules.
2. `src/pop_dm_lib/src/error.rs` defines `Error` and `Result`.
3. `src/pop_dm_lib/src/user.rs` defines a user model.
4. `src/pop_dm_lib/src/auth.rs` defines the authentication boundary.
5. `src/pop_dm_lib/src/session.rs` defines the session command model.
6. `src/pop_dm_lib/src/desktop_boot.rs` defines the desktop session model.
7. `src/pop_dm/src/config.rs` defines `Config::default()`.
8. `src/pop_dm/src/tty.rs` has a tiny banner or prompt helper.
9. `src/pop_dm/src/main.rs` has a clean `run() -> Result<()>`.
10. `cd src && cargo fmt && cargo test && cargo clippy --all-targets --all-features` passes.
