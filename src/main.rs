mod address;
mod buffer;
mod cli;
mod commands;
mod io;
mod regex;

fn main() {
    let opts = cli::parse_args(std::env::args().skip(1));
    let code = commands::run(opts);
    std::process::exit(code);
}
