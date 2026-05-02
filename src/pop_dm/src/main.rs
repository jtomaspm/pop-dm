mod boot;
mod config;
mod tty;

fn main() {
    if let Err(err) = boot::boot_and_run() {
        eprintln!("pop_dm: {err}");
        std::process::exit(1);
    }
}
