# Learning Plan: Build a Linux Wayland Display Manager in C++

## Key Clarification

A Linux display manager is not the same thing as a Wayland compositor.

A display manager mainly does this:

1. Starts at boot.
2. Shows a login UI, called a greeter.
3. Authenticates the user with PAM.
4. Creates a user session through `systemd-logind` or seat management.
5. Starts the selected desktop/session, such as Sway, Hyprland, GNOME, KDE, etc.

For Wayland, the hard part is that the greeter itself needs somewhere to render. That means either:

1. Run a text greeter on the TTY first.
2. Run a graphical greeter inside an existing small compositor like `cage`.
3. Write your own Wayland compositor for the greeter, which is much harder.

Recommended path:

> Build a minimal text-based display manager first, then add Wayland session launching, then later build a graphical Wayland greeter.

## Phase 0: Foundations

Learn these before writing the display manager:

1. C++ basics for systems programming:
   - RAII
   - file descriptors
   - `fork`, `exec`, `waitpid`
   - signals
   - environment variables
   - privilege dropping
   - error handling

2. Linux process/session concepts:
   - users and groups
   - UID/GID
   - TTYs
   - sessions and process groups
   - environment variables
   - `/etc/passwd`, `/etc/shadow`
   - system services

3. Build tooling:
   - CMake or Meson
   - compiler warnings
   - sanitizers
   - `pkg-config`

Suggested mini-projects:

1. Write a C++ program that runs another command with `fork` + `exec`.
2. Write a program that drops from root to another user using `setuid`/`setgid`.
3. Write a program that starts a child process and correctly handles signals.

## Phase 1: Understand Display Managers

Study existing projects, but do not copy them directly:

1. `greetd`
2. `ly`
3. `sddm`
4. `lightdm`
5. `tuigreet`

Focus on these questions:

1. How does the daemon start?
2. What runs as root?
3. What runs as the user?
4. Where does PAM happen?
5. How is the session selected?
6. How does it start Wayland sessions?
7. How does it clean up after logout?

Important files/specs to learn:

1. `/usr/share/wayland-sessions/*.desktop`
2. `/usr/share/xsessions/*.desktop`
3. `/etc/pam.d/*`
4. systemd service units
5. Desktop Entry Specification

## Phase 2: Build a Minimal TTY Login Manager

Goal: a simple text-mode display manager.

Features:

1. Runs from a terminal or TTY.
2. Prompts for username.
3. Prompts for password.
4. Authenticates with PAM.
5. Starts a fixed command after login, for example:

```sh
sway
```

Concepts to learn:

1. PAM conversations
2. hiding password input
3. launching child processes
4. setting `HOME`, `USER`, `LOGNAME`, `SHELL`
5. switching UID/GID
6. waiting for the session to exit

At this point, do not worry about graphics.

First milestone:

> Boot into a TTY, log in through your program, and launch `sway`.

Test this in a VM, not on your main system.

## Phase 3: PAM Properly

PAM is central to a real display manager.

Learn:

1. `pam_start`
2. `pam_authenticate`
3. `pam_acct_mgmt`
4. `pam_open_session`
5. `pam_close_session`
6. PAM conversation callbacks
7. PAM environment variables

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

## Phase 4: Session Discovery

Instead of hardcoding `sway`, teach your display manager to discover installed sessions.

Read:

```txt
/usr/share/wayland-sessions/*.desktop
/usr/share/xsessions/*.desktop
```

For Wayland, start with only:

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

> Show a list of installed Wayland sessions and allow the user to select one.

Be careful with `Exec`.

Do not pass it through a shell unless you intentionally support shell parsing. Prefer tokenizing it carefully or initially support only simple commands.

## Phase 5: systemd-logind and Seats

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

## Phase 6: Wayland Session Launching

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

> Your display manager can launch at least Sway, Hyprland, or another Wayland compositor from a clean boot.

## Phase 7: Turn It Into a Real Service

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

> The machine boots into your display manager automatically.

Test this in a VM.

## Phase 8: Graphical Greeter Options

Once the text version works, decide how graphical you want to go.

Option A: Use an existing compositor for the greeter.

Example architecture:

```txt
popdm daemon
  -> starts cage/labwc/tinywl-like compositor
      -> starts popdm-greeter Wayland client
```

Pros:

1. Much easier.
2. You can write the greeter with Qt, GTK, SDL, or another toolkit.
3. You avoid writing a compositor early.

Cons:

1. Not completely from scratch.

Option B: Write your own minimal Wayland compositor.

Pros:

1. Deep learning.
2. Full control.

Cons:

1. Much harder.
2. Requires learning `wlroots`, DRM/KMS, input, rendering, seats, outputs.
3. `wlroots` is C-first, so C++ integration needs care.

Recommended path:

> Use `cage` or another minimal compositor first. Write your own compositor later as a separate learning project.

## Phase 9: Security Hardening

Security matters a lot because a display manager touches login and privileges.

Learn and implement:

1. never log passwords
2. sanitize environment variables
3. avoid shell injection
4. drop privileges as early as possible
5. separate daemon and greeter processes
6. keep root-owned code minimal
7. handle failed logins safely
8. rate-limit login attempts
9. lock down config file permissions
10. close unnecessary file descriptors
11. handle signals cleanly
12. ensure sessions are cleaned up

Suggested architecture eventually:

```txt
root daemon
  -> authenticates using PAM
  -> manages sessions
  -> starts unprivileged greeter
  -> starts user session after login
```

## Phase 10: Configuration

Add a simple config format only after the core works.

Possible config options:

```txt
default_session=sway
default_user=
greeter_command=
session_dirs=/usr/share/wayland-sessions
log_file=
tty=1
```

Use something simple:

1. INI
2. TOML
3. plain key-value

Avoid making configuration complicated early.

## Phase 11: Polish

Later features:

1. user list
2. session picker
3. reboot/shutdown buttons
4. keyboard layout selection
5. HiDPI support
6. multi-monitor support
7. accessibility
8. theming
9. failed-login delay
10. automatic login
11. user avatars
12. logging
13. packaging

Do not start here.

## Suggested Learning Timeline

A realistic path:

1. Weeks 1-2: Linux process, users, TTY, C++ process management.
2. Weeks 3-4: PAM authentication prototype.
3. Weeks 5-6: launch a fixed Wayland session from TTY.
4. Weeks 7-8: session discovery from `.desktop` files.
5. Weeks 9-10: systemd service integration.
6. Weeks 11-12: cleanup, logging, config, reliability.
7. Weeks 13-16: graphical greeter using an existing compositor.
8. Later: write your own minimal compositor if you still want the deepest Wayland path.

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
9. It works as a systemd service.
10. It gets a graphical greeter.

## Resources

Start with:

1. Arch Wiki: Display Manager
2. Arch Wiki: Sway
3. Arch Wiki: systemd
4. Arch Wiki: PAM
5. `man pam`
6. `man pam_start`
7. `man pam_open_session`
8. `man systemd.service`
9. `man loginctl`
10. `man setuid`
11. `man fork`
12. `man execve`
13. The Wayland Book: https://wayland-book.com/
14. Desktop Entry Specification: https://specifications.freedesktop.org/desktop-entry-spec/latest/
15. `greetd` source code
16. `ly` source code
17. `sddm` source code

## Strong Recommendation

Do not start by writing a Wayland compositor.

Start by writing a secure, boring, text-based display manager that can authenticate with PAM and launch Sway. Once that works, you will understand the real job of a display manager. Then build the graphical Wayland greeter on top.
