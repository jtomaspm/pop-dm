# Learning Plan: Build a Text-Based Linux Display Manager in Rust

## Key Clarification

A Linux display manager is not the same thing as a Wayland compositor.

A display manager mainly does this:

1. Starts at boot.
2. Shows a login interface, called a greeter.
3. Authenticates the user with PAM.
4. Opens a user session through PAM and `systemd-logind`.
5. Starts the selected desktop/session, such as Sway, Hyprland, GNOME, KDE, etc.
6. Cleans up after logout and returns to the login prompt.

For your project, the greeter will be text-based and run on a TTY. That means you do not need to write a Wayland compositor or graphical greeter first.

Recommended path:

> Build a secure, boring, text-based Rust display manager that authenticates with PAM, discovers Wayland sessions, launches one, waits for it to exit, then returns to the prompt.

## Phase 0: Rust and Linux Foundations

Learn these before writing the display manager:

1. Rust basics for systems programming:
   - ownership and borrowing
   - `Result` and error propagation
   - lifetimes at a practical level
   - modules and crates
   - `Drop` for cleanup
   - `std::process::Command`
   - `std::os::unix`
   - safe wrappers around unsafe Unix calls

2. Linux process/session concepts:
   - users and groups
   - UID/GID
   - TTYs
   - sessions and process groups
   - environment variables
   - `/etc/passwd`, `/etc/shadow`
   - system services
   - file descriptors
   - signals

3. Rust tooling:
   - Cargo
   - `rustfmt`
   - `clippy`
   - `cargo test`
   - `cargo audit`
   - `pkg-config`
   - linking against system libraries such as PAM

Suggested crates to investigate:

1. `nix` for Unix process, user, signal, and terminal APIs.
2. `users` or `uzers` for user/group lookup.
3. `pam` or `pam-client` for PAM integration.
4. `rpassword` for hidden password input.
5. `rust-ini` or `freedesktop-desktop-entry` for parsing `.desktop` files.
6. `toml` and `serde` for later configuration.
7. `tracing` or `log` for structured logging.

Suggested mini-projects:

1. Write a Rust program that runs another command with `Command`.
2. Write a Rust program that looks up a user and prints their UID, GID, home, and shell.
3. Write a Rust program that drops from root to another user using `setgid`, `initgroups`, and `setuid`.
4. Write a Rust program that starts a child process and forwards/handles signals.
5. Write a tiny TTY prompt that hides password input.

## Phase 1: Understand Display Managers

Study existing projects, but do not copy them directly:

1. `greetd`
2. `tuigreet`
3. `ly`
4. `sddm`
5. `lightdm`

Focus on these questions:

1. How does the daemon start?
2. What runs as root?
3. What runs as the user?
4. Where does PAM happen?
5. How is the session selected?
6. How does it start Wayland sessions?
7. How does it clean up after logout?
8. How does it avoid leaking privileged state into the user session?

Important files/specs to learn:

1. `/usr/share/wayland-sessions/*.desktop`
2. `/usr/share/xsessions/*.desktop`
3. `/etc/pam.d/*`
4. systemd service units
5. Desktop Entry Specification

## Phase 2: Project Shape

Start with a single Rust binary, then split modules as the code becomes real.

Suggested structure:

```txt
popdm/
  Cargo.toml
  src/
    main.rs
    auth.rs
    session.rs
    desktop_entry.rs
    tty.rs
    user.rs
    config.rs
    error.rs
```

Initial responsibilities:

1. `main.rs`: login loop and top-level control flow.
2. `auth.rs`: PAM authentication and session open/close.
3. `session.rs`: launching and waiting for the user session.
4. `desktop_entry.rs`: discovering Wayland sessions.
5. `tty.rs`: prompt, password input, terminal cleanup.
6. `user.rs`: UID/GID/home/shell lookup.
7. `config.rs`: later config loading.
8. `error.rs`: shared error type if needed.

Keep the first version small. A display manager is security-sensitive, so boring code is good code.

## Phase 3: Build a Minimal TTY Login Manager

Goal: a simple text-mode display manager.

Features:

1. Runs manually from a terminal or TTY.
2. Prompts for username.
3. Prompts for password.
4. Authenticates with PAM.
5. Starts a fixed command after login, for example:

```sh
sway
```

Concepts to learn:

1. PAM conversations from Rust
2. hiding password input
3. launching child processes
4. setting `HOME`, `USER`, `LOGNAME`, `SHELL`
5. setting `PATH` conservatively
6. initializing supplementary groups
7. switching GID/UID
8. waiting for the session to exit
9. returning to the login prompt

First milestone:

> Boot into a TTY, log in through your Rust program, and launch `sway`.

Test this in a VM, not on your main system.

## Phase 4: PAM Properly

PAM is central to a real display manager.

Learn:

1. `pam_start`
2. `pam_authenticate`
3. `pam_acct_mgmt`
4. `pam_open_session`
5. `pam_close_session`
6. PAM conversation callbacks
7. PAM environment variables
8. `pam_systemd`

Important rule:

> Never inspect, log, store, or reuse passwords outside the PAM conversation.

You should create a PAM service file eventually, for example:

```txt
/etc/pam.d/popdm
```

Study the PAM configs used by:

```txt
/etc/pam.d/login
/etc/pam.d/sddm
/etc/pam.d/gdm
/etc/pam.d/greetd
```

Rust-specific notes:

1. Prefer a maintained PAM crate if it handles the conversation model cleanly.
2. If you need direct FFI, isolate unsafe PAM calls in `auth.rs`.
3. Make PAM session lifetime explicit so `pam_close_session` runs even if the child exits badly.
4. Do not keep password strings around longer than needed.

## Phase 5: Session Discovery

Instead of hardcoding `sway`, teach your display manager to discover installed sessions.

Read:

```txt
/usr/share/wayland-sessions/*.desktop
/usr/share/xsessions/*.desktop
```

For your first target, support only:

```txt
/usr/share/wayland-sessions/*.desktop
```

Learn the Desktop Entry spec.

You need to parse:

1. `Name`
2. `Comment`
3. `Exec`
4. `TryExec`
5. `DesktopNames`

Milestone:

> Show a numbered list of installed Wayland sessions and allow the user to select one.

Be careful with `Exec`.

Do not pass it through a shell unless you intentionally support shell parsing. Prefer a Desktop Entry parser crate or initially support only simple commands. For the first version, it is acceptable to reject complex `Exec` lines and print a clear error.

## Phase 6: systemd-logind and Seats

A proper display manager should integrate with seat/session management.

Learn:

1. `systemd-logind`
2. seats
3. sessions
4. TTY ownership
5. active sessions
6. D-Bus basics

Relevant tools:

```sh
loginctl
loginctl seat-status seat0
loginctl session-status
```

Read about:

1. `pam_systemd`
2. `XDG_SESSION_TYPE`
3. `XDG_SESSION_CLASS`
4. `XDG_SESSION_DESKTOP`
5. `XDG_CURRENT_DESKTOP`
6. `XDG_RUNTIME_DIR`

For a first version, relying on PAM with `pam_systemd` is acceptable.

Milestone:

> After login, `loginctl` shows a proper user session.

## Phase 7: Wayland Session Launching

For Wayland sessions, your display manager should set the right environment.

Common variables:

```sh
XDG_SESSION_TYPE=wayland
XDG_SESSION_CLASS=user
XDG_CURRENT_DESKTOP=sway
XDG_SESSION_DESKTOP=sway
```

Also understand:

```sh
XDG_RUNTIME_DIR
WAYLAND_DISPLAY
DBUS_SESSION_BUS_ADDRESS
```

For many sessions, `pam_systemd` and the launched desktop handle most of this.

Milestone:

> Your Rust display manager can launch at least Sway, Hyprland, or another Wayland compositor from a clean boot.

## Phase 8: Turn It Into a Real Service

Create a systemd service only after the program works manually.

Learn:

1. systemd unit files
2. service restart behavior
3. dependencies
4. TTY allocation
5. conflicts with `getty`
6. boot targets

You will need to understand units like:

```txt
display-manager.service
getty@tty1.service
graphical.target
multi-user.target
```

Milestone:

> The machine boots into your text-based display manager automatically.

Test this in a VM.

## Phase 9: Security Hardening

Security matters a lot because a display manager touches login and privileges.

Learn and implement:

1. never log passwords
2. sanitize environment variables
3. avoid shell injection
4. drop privileges as early as possible
5. keep root-owned code minimal
6. handle failed logins safely
7. rate-limit login attempts
8. lock down config file permissions
9. close unnecessary file descriptors
10. handle signals cleanly
11. ensure sessions are cleaned up
12. make unsafe Rust blocks tiny and documented
13. avoid panics in privileged control flow
14. test error paths, not only successful login

Suggested architecture eventually:

```txt
root popdm process
  -> authenticates using PAM
  -> opens PAM session
  -> discovers selected session
  -> forks/execs user session with dropped privileges
  -> waits for session exit
  -> closes PAM session
  -> returns to text login prompt
```

## Phase 10: Configuration

Add a simple config format only after the core works.

Possible config options:

```toml
default_session = "sway"
default_user = ""
session_dirs = ["/usr/share/wayland-sessions"]
tty = 1
failed_login_delay_seconds = 2
```

Use TOML with `serde` once you need config. Avoid making configuration complicated early.

## Phase 11: Tests and Development Safety

Some parts of a display manager are hard to test automatically, but still test what you can.

Good test targets:

1. `.desktop` parsing
2. `Exec` tokenization/rejection
3. config parsing
4. user-facing selection logic
5. environment construction
6. failed-login delay behavior

Manual test matrix:

1. wrong password
2. nonexistent user
3. locked user
4. session command missing
5. session exits normally
6. session crashes
7. repeated login/logout
8. boot into VM service mode

Use a VM snapshot before testing service integration.

## Phase 12: Polish

Later features:

1. user list
2. session picker
3. reboot/shutdown commands
4. keyboard layout selection
5. failed-login delay
6. automatic login
7. logging
8. packaging
9. man page
10. distro-specific install notes

Do not start here.

## Suggested Learning Timeline

A realistic path:

1. Weeks 1-2: Rust, Linux process basics, users, TTYs, signals.
2. Weeks 3-4: PAM authentication prototype.
3. Weeks 5-6: launch a fixed Wayland session from TTY.
4. Weeks 7-8: session discovery from `.desktop` files.
5. Weeks 9-10: systemd service integration in a VM.
6. Weeks 11-12: cleanup, logging, config, reliability.
7. Later: optional graphical greeter or compositor work as a separate project.

## Recommended First Milestones

Build in this exact order:

1. `popdm-dev` runs manually in a TTY.
2. It asks for username/password.
3. It authenticates with PAM.
4. It launches `sway`.
5. It waits until `sway` exits.
6. It returns to the login prompt.
7. It lists installed Wayland sessions.
8. It launches the selected session.
9. `loginctl` shows a real user session.
10. It works as a systemd service in a VM.

## Resources

Start with:

1. The Rust Book: https://doc.rust-lang.org/book/
2. Rust `std::process::Command`: https://doc.rust-lang.org/std/process/struct.Command.html
3. Rust `std::os::unix`: https://doc.rust-lang.org/std/os/unix/
4. `nix` crate docs: https://docs.rs/nix/
5. Arch Wiki: Display Manager
6. Arch Wiki: Sway
7. Arch Wiki: systemd
8. Arch Wiki: PAM
9. `man pam`
10. `man pam_start`
11. `man pam_open_session`
12. `man systemd.service`
13. `man loginctl`
14. `man setuid`
15. `man execve`
16. Desktop Entry Specification: https://specifications.freedesktop.org/desktop-entry-spec/latest/
17. `greetd` source code
18. `tuigreet` source code
19. `ly` source code

## Strong Recommendation

Do not start by writing a Wayland compositor.

Start by writing a secure Rust text-based display manager that can authenticate with PAM and launch Sway. Once that works, you will understand the real job of a display manager. A graphical Wayland greeter can be a later project, not the foundation.
