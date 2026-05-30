mod cli;
mod hook;
mod storage;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
