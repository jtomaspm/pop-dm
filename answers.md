# Ly Daemon And Session Flow

## Short Version

Ly is a foreground TTY display manager started by the init system as root. It keeps a root supervisor/UI process, forks a root authentication/PAM process, then forks the actual user session process and drops to the target user before executing the session command.

## How The Daemon Starts

Ly is not really daemonizing itself; it is supervised by init.

- systemd starts `ly@ttyN.service` with `ExecStart=$PREFIX_DIRECTORY/bin/$EXECUTABLE_NAME`, `StandardInput=tty`, and `TTYPath=/dev/%I`: `res/ly@.service:7-13`.
- The KMSCON variant starts `kmscon ... -- ly --use-kmscon-vt`: `res/ly-kmsconvt@.service:7-14`.
- Other init systems either run `ly` directly or via `getty` as the login program, for example OpenRC/runit/s6: `res/ly-openrc:22-29`, `res/ly-runit-service/run:13`, `res/ly-s6/run:2`.
- The binary entrypoint is `main()` in `src/main.zig:133`.

On startup it loads config, runs optional `start_cmd`, initializes the terminal UI, discovers users and sessions, switches to the active TTY, then enters the UI event loop: `src/main.zig:366-404`, `src/main.zig:981-1010`, `src/main.zig:1323-1333`.

## What Runs As Root

Runs as root:

- The init-started `ly` process, because the service files do not set a non-root `User=`.
- The TUI/main supervisor process.
- Startup commands like `start_cmd`: `src/main.zig:366-379`.
- Pre-login UI commands/custom keybinds: `src/main.zig:1429-1439`.
- PAM authentication and session open/close: `src/auth.zig:66-93`.
- The root auth supervisor that waits for the user session and maintains utmp: `src/auth.zig:129-158`.
- `logout_cmd`, after the user session exits: `src/main.zig:1621-1626`.
- Shutdown/restart/sleep/brightness commands from the greeter also run from the root greeter process.

## What Runs As The User

The actual session child drops privileges before running session commands:

- `startSession()` calls `interop.setUserContext(...)`: `src/auth.zig:171-174`.
- On Linux that does `initgroups`, `setgid`, then `setuid`: `ly-core/src/interop.zig:45-51`.
- After that, it sets `HOME`, `PWD`, `SHELL`, `USER`, `LOGNAME`, `PATH`, XDG variables, PAM environment variables, and `chdir`s to the user home: `src/auth.zig:175-200`.
- Then it executes the selected session through the user's shell: `src/auth.zig:206-216`.

So these run as the logged-in user:

- `setup_cmd`: default `/etc/ly/setup.sh`.
- Optional `login_cmd`.
- The selected Wayland/X11/custom/shell command.
- X11 helper/session commands too, because X11 execution is reached after `setUserContext()`.

## Where PAM Happens

PAM happens in `src/auth.zig` inside `auth.authenticate()`.

Flow:

- UI calls local `authenticate()` on Enter/autologin: `src/main.zig:1275`, `src/main.zig:1311`, `src/main.zig:1442`.
- That forks a child `session_pid`: `src/main.zig:1539-1540`.
- The child calls `auth.authenticate(...)`: `src/main.zig:1571-1579`.
- PAM starts with the configured service name: `src/auth.zig:66-69`.
- It sets `PAM_TTY`: `src/auth.zig:71-74`.
- It calls `pam_authenticate`, `pam_acct_mgmt`, `pam_setcred`, and `pam_open_session`: `src/auth.zig:76-93`.
- PAM env vars are later copied into the user session process with `pam_getenvlist`: `src/auth.zig:184-193`.
- PAM cleanup happens by defers: `pam_close_session`, `pam_setcred(PAM_DELETE_CRED)`, and `pam_end`: `src/auth.zig:88-93`, `src/auth.zig:69`.

PAM service files are installed as `/etc/pam.d/ly` and `/etc/pam.d/ly-autologin`: `build.zig:229-242`. The Linux PAM file includes normal `login` auth/account/session stacks and optional keyring modules: `res/pam.d/ly-linux:1-16`. Autologin uses `pam_permit` for auth: `res/pam.d/ly-linux-autologin:1-16`.

## How The Session Is Selected

Sessions are represented as `Environment` entries: `src/Environment.zig:16-24`.

Session discovery:

- Optional shell entry is added if `shell = true`: `src/main.zig:778-792`.
- Optional xinitrc entry is added if X11 support/config allows it: `src/main.zig:794-809`.
- Wayland `.desktop` files are crawled from configured `waylandsessions`: `src/main.zig:826-840`.
- X11 `.desktop` files are crawled from configured `xsessions`: `src/main.zig:842-857`.
- Custom sessions are crawled from `custom_sessions`: `src/main.zig:859-870`.

`.desktop` parsing pulls `Name`, `Exec`, `DesktopNames`, and `Terminal`: `src/main.zig:2158-2212`.

The UI uses a cyclable label. Left/right changes the selected item: `ly-ui/src/components/generic.zig:64-67`, `ly-ui/src/components/generic.zig:140-169`.

Per-user session selection is remembered:

- When the session changes, it writes that session index into the current user record: `src/components/Session.zig:100-104`.
- When the user changes, it restores that user's saved session index: `src/components/UserList.zig:124-128`.
- Saved user/session choices are loaded and restored: `src/main.zig:1109-1132`.
- Current choices are saved before login: `src/main.zig:1493-1523`.

Autologin bypasses manual selection by finding a configured session name via filename, display name, `XDG_SESSION_DESKTOP`, or desktop names: `src/main.zig:931-978`, `src/main.zig:2223-2235`.

## How Wayland Sessions Start

Wayland sessions are `.desktop` entries from `waylandsessions`, defaulting to `$PREFIX_DIRECTORY/share/wayland-sessions`: `src/config/Config.zig:99`, `res/config.ini:381-385`.

On login:

- The selected `Environment` is read from `state.session.label.current`: `src/main.zig:1541`.
- XDG variables are set before and after dropping privileges. For Wayland, `XDG_SESSION_TYPE=wayland`: `src/auth.zig:46-49`, `src/auth.zig:179-182`, `src/auth.zig:234-248`.
- `XDG_RUNTIME_DIR` is set to `/run/user/$uid` on non-FreeBSD: `src/auth.zig:250-260`.
- The session child releases the greeter's TTY control by signaling the parent: `src/auth.zig:202-204`.
- For `.wayland`, Ly calls `executeCmd(...)`: `src/auth.zig:206-209`.
- `executeCmd` optionally redirects stdout/stderr to `session_log`, then executes the command through the user shell: `src/auth.zig:533-562`.

The command shape is effectively:

```sh
$user_shell -c "$setup_cmd $login_cmd $desktop_exec"
```

The default `setup.sh` sources profile files and ends with `exec "$@"`, so it hands off to the session command: `res/setup.sh:13-107`.

For KMSCON non-terminal sessions, Ly prefixes with `kmscon-launch-gui`: `src/auth.zig:557`.

## How Cleanup After Logout Works

There are two supervisor layers.

Main Ly process:

- Forks auth/session child: `src/main.zig:1539-1588`.
- Waits for that child to finish: `src/main.zig:1590-1595`.
- Reinitializes logs and reclaims the terminal buffer: `src/main.zig:1597-1600`.
- If authentication/session ended cleanly, runs optional `logout_cmd`, clears password, disables autologin for future loops: `src/main.zig:1621-1629`.

Auth/PAM supervisor:

- Opens PAM session before launching the user session: `src/auth.zig:90-93`.
- Forks the actual user session process: `src/auth.zig:113-127`.
- Adds a utmp entry: `src/auth.zig:147`, `src/auth.zig:588-629`.
- Waits for the user session to exit: `src/auth.zig:149-151`.
- Removes the utmp entry: `src/auth.zig:153-156`, `src/auth.zig:631-638`.
- Then PAM defers close the session, delete credentials, and end PAM.

Signal cleanup:

- Main Ly forwards termination signals to the auth/session child: `src/main.zig:48-61`.
- Auth supervisor forwards `SIGTERM` to the user session child: `src/auth.zig:34-37`, `src/auth.zig:139-145`.
- X11 sessions additionally terminate the X server with `SIGTERM`, then `SIGKILL` if needed: `src/auth.zig:521-530`.
