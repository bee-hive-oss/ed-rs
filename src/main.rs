mod address;
mod buffer;
mod cli;
mod commands;
mod io;
mod regex;

fn main() {
    let code = apply(cli::parse_args(std::env::args()));
    // `process::exit` skips flushing Rust's stdio buffers.
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let _ = std::io::Write::flush(&mut std::io::stderr());
    std::process::exit(code);
}

/// Print help, version, or a parse error, and return the process status.
/// [`cli::Action::Run`] enters the editor.
fn apply(action: cli::Action) -> i32 {
    match action {
        cli::Action::Run(opts) => commands::run(opts),
        cli::Action::Help(text) => {
            print!("{text}");
            0
        }
        cli::Action::Version(text) => {
            print!("{text}");
            0
        }
        cli::Action::Error(text) => {
            eprint!("{text}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_and_version_exit_zero() {
        let help = cli::parse_args(["ed-rs".to_string(), "-h".to_string()]);
        assert!(matches!(help, cli::Action::Help(_)));
        assert_eq!(apply(help), 0);

        let version = cli::parse_args(["ed-rs".to_string(), "--version".to_string()]);
        assert!(matches!(version, cli::Action::Version(_)));
        assert_eq!(apply(version), 0);
    }

    #[test]
    fn option_errors_exit_one() {
        let bad = cli::parse_args(["ed-rs".to_string(), "--nope".to_string()]);
        assert!(matches!(bad, cli::Action::Error(_)));
        assert_eq!(apply(bad), 1);

        let missing = cli::parse_args(["ed-rs".to_string(), "-p".to_string()]);
        assert_eq!(apply(missing), 1);
    }
}
