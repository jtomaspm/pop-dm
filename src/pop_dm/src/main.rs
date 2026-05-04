mod config;
mod runtime;
mod tty;

fn main() {
    if let Err(err) = runtime::boot_and_run() {
        eprintln!("pop_dm: {err}");
        std::process::exit(1);
    }
}
