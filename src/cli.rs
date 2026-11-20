//! Command-line parsing for ed-rs, matching GNU ed 1.22.4.
//!
//! # Invocation name
//!
//! `parse_args` takes the full argument vector, including `argv[0]`.
//! Diagnostics use two different strings, on purpose:
//!
//! * The `ed-rs:` tag is the **basename** of `argv[0]`. GNU ed hardcodes
//!   its program name (`ed`), which is that basename when the binary is
//!   invoked under its real name. A path such as `/usr/bin/ed-rs` therefore
//!   still prints `ed-rs:`. An empty basename falls back to `ed-rs`.
//! * The hint `Try '… --help'` and the help `Usage:` line use the
//!   **invocation string unchanged** (`argv[0]` as passed). That matches
//!   GNU ed, whose hint is `Try '%s --help'` with `argv[0]`.
//!
//! # Options and operands
//!
//! Long options accept unambiguous prefixes (`--scr` is `--script`).
//! `--silent` is an alias of `-q` / `--quiet`, not of `-s` / `--script`.
//! Short flags combine (`-qs`). `-p` consumes the rest of its cluster or
//! the next argument; an empty prompt string is rejected.
//!
//! Options may appear after operands. `--` stops option parsing. A later
//! non-option is not an option. After the file name, further non-option
//! operands are ignored.
//!
//! A `+` operand is a start address, not a file name:
//!
//! * `+/RE` / `+?RE` (the expression may be empty) sets [`Options::start_regexp`].
//! * A second character that is an ASCII digit is a line number. Parsing
//!   matches C `strtol` base 10: trailing non-digits are ignored (`+12abc`
//!   is line 12) and the value must lie in `1..=i32::MAX` (GNU ed stores
//!   the address in a C `int`). `+0` and out-of-range numbers are errors.
//! * Anything else (`+foo`) is an error.
//!
//! The last numeric operand and the last regexp operand are both kept.
//! GNU ed applies the numeric address when both are set; this seat only
//! records them.
//!
//! An operand that is exactly `-` sets script mode ([`Options::silent`]) and
//! is not a file name, even after `--`. It does not suppress diagnostics;
//! that is [`Options::quiet`]. `-q` does suppress the start-address
//! diagnostics, but those paths still fail. Option errors are never
//! suppressed, and `-l` does not turn them into success.
//!
//! Help and version are printed by the caller and exit 0. The first of
//! `-h` / `--help` and `-V` / `--version` in argv order wins. A parse
//! error anywhere in the options is reported instead of help or version.

/// What `main` should do with a parsed command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Enter the editor with these options.
    Run(Options),
    /// Write this help text to stdout and exit 0.
    Help(String),
    /// Write this version text to stdout and exit 0.
    Version(String),
    /// Write this diagnostic to stderr (it may be empty) and exit 1.
    ///
    /// Empty means `-q` suppressed a start-address diagnostic. The process
    /// still exits 1.
    Error(String),
}

/// Startup options accepted by `commands::run`.
///
/// `silent` is the historical field name for script mode. It is **not**
/// `-q`. See [`Options::quiet`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Options {
    /// `-s` / `--script`, and a `-` operand.
    ///
    /// Suppresses byte counts and the `!` prompt. Distinct from [`Options::quiet`].
    pub silent: bool,
    /// `-q` / `--quiet` / `--silent`.
    ///
    /// Suppresses diagnostics written to stderr. Distinct from [`Options::silent`].
    pub quiet: bool,
    /// `-p` / `--prompt`. `None` when the flag was not given.
    ///
    /// An empty string is not stored: GNU ed rejects it as a missing argument.
    pub prompt: Option<String>,
    /// `-G` / `--traditional`: compatibility mode.
    pub traditional: bool,
    /// `-l` / `--loose-exit-status`: exit 0 even if a command fails.
    ///
    /// This does not apply to invalid options or a bad `+` address. Those
    /// still exit 1.
    pub loose_exit: bool,
    /// `-v` / `--verbose`: explain errors, equivalent to the `H` command.
    pub verbose: bool,
    /// `-r` / `--restricted`.
    pub restricted: bool,
    /// `-E` / `--extended-regexp`.
    pub extended_regexp: bool,
    /// `--strip-trailing-cr`.
    pub strip_trailing_cr: bool,
    /// `--unsafe-names`: allow control characters in file names.
    pub unsafe_names: bool,
    /// File to read, if one was given. A leading `!` is preserved for the
    /// I/O seat (read the output of a shell command). Not a `-` operand.
    pub file: Option<String>,
    /// Line from the last `+<digits>` operand, in `1..=i32::MAX`.
    ///
    /// When this is set together with [`Options::start_regexp`], GNU ed
    /// uses the line and does not run the search.
    pub start_line: Option<i32>,
    /// Last `+/RE` or `+?RE` operand.
    pub start_regexp: Option<StartRegexp>,
}

/// A `+/RE` or `+?RE` startup search.
///
/// The pattern is everything after the delimiter and may be empty. It is
/// not checked for a closing delimiter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartRegexp {
    /// `+/RE`: first match, searching forward.
    Forward(String),
    /// `+?RE`: last match, searching backward.
    Backward(String),
}

/// Parse `argv`, including the invocation name in `argv[0]`.
///
/// An empty iterator is treated as invocation `ed-rs` with no arguments.
pub fn parse_args<I>(args: I) -> Action
where
    I: IntoIterator<Item = String>,
{
    let argv: Vec<String> = args.into_iter().collect();
    parse_argv(&argv)
}

fn parse_argv(argv: &[String]) -> Action {
    let invocation = argv.first().map(String::as_str).unwrap_or("ed-rs");
    let (parsed_options, operands) = match split_args(argv) {
        Ok(parts) => parts,
        Err(message) => return Action::Error(option_error(invocation, &message)),
    };

    let mut opts = Options::default();
    for opt in parsed_options {
        match opt.code {
            code::HELP => return Action::Help(help_text(invocation)),
            code::VERSION => return Action::Version(version_text()),
            code::EXTENDED => opts.extended_regexp = true,
            code::TRADITIONAL => opts.traditional = true,
            code::LOOSE => opts.loose_exit = true,
            code::PROMPT => {
                if let Some(text) = opt.argument {
                    opts.prompt = Some(text);
                }
            }
            code::QUIET => opts.quiet = true,
            code::RESTRICTED => opts.restricted = true,
            code::SCRIPT => opts.silent = true,
            code::VERBOSE => opts.verbose = true,
            code::STRIP_CR => opts.strip_trailing_cr = true,
            code::UNSAFE_NAMES => opts.unsafe_names = true,
            _ => {
                return Action::Error(tagged(invocation, "internal error: uncaught option."));
            }
        }
    }

    for arg in operands {
        if arg == "-" {
            opts.silent = true;
            continue;
        }
        if let Some(rest) = arg.strip_prefix('+') {
            match classify_start(rest) {
                StartKind::Regexp(kind) => opts.start_regexp = Some(kind),
                StartKind::Line(line) => opts.start_line = Some(line),
                StartKind::BadNumber => {
                    return line_error(
                        invocation,
                        opts.quiet,
                        rest,
                        "Invalid line number; must be >= 1.",
                    );
                }
                StartKind::BadAddress => {
                    return line_error(
                        invocation,
                        opts.quiet,
                        &arg,
                        "Invalid line number or regular expression.",
                    );
                }
            }
            continue;
        }
        opts.file = Some(arg);
        break;
    }
    Action::Run(opts)
}

/// Basename of the invocation, used as the `name:` diagnostic tag.
fn program_tag(invocation: &str) -> &str {
    match invocation.rsplit('/').next() {
        Some(base) if !base.is_empty() => base,
        _ => "ed-rs",
    }
}

fn tagged(invocation: &str, message: &str) -> String {
    format!("{}: {message}\n", program_tag(invocation))
}

fn option_error(invocation: &str, message: &str) -> String {
    format!(
        "{}Try '{invocation} --help' for more information.\n",
        tagged(invocation, message)
    )
}

fn line_error(invocation: &str, quiet: bool, subject: &str, detail: &str) -> Action {
    if quiet {
        Action::Error(String::new())
    } else {
        Action::Error(tagged(invocation, &format!("{subject}: {detail}")))
    }
}

fn help_text(invocation: &str) -> String {
    format!(
        "\
ed-rs is a clean-room Rust port of the ed line editor. It applies
commands from standard input to a buffer, interactively or from a script.
Credited to Bee, Grok Bot, and Nori. Licensed under GPL-3.0-or-later.

Usage: {invocation} [options] [[+line] file]

'+line', '+/RE', or '+?RE' before the file name selects the starting
current line: that line number, or the first or last line matching RE.
If a file is given, ed-rs reads it before accepting commands. A file
name that begins with '!' reads the output of a shell command. An
operand of '-' turns on script mode, the same as -s, and is not a file
name. Operands after the file name are ignored. Options may be mixed
with operands; a standalone '--' stops option parsing.

Options:
  -h, --help                 show this help and exit
  -V, --version              show the version and exit
  -E, --extended-regexp      use extended regular expressions
  -G, --traditional          run in compatibility mode
  -l, --loose-exit-status    exit 0 even if a command fails
  -p, --prompt=STRING        use STRING as the interactive prompt
  -q, --quiet, --silent      suppress diagnostics written to stderr
  -r, --restricted           run in restricted mode
  -s, --script               suppress byte counts and the '!' prompt
  -v, --verbose              be verbose; equivalent to the H command
      --strip-trailing-cr    strip carriage returns at the end of text lines
      --unsafe-names         allow control characters in file names

Exit status:
  0  normal exit
  1  environmental problems (invalid command-line options, memory
     exhausted, a command failed, and similar failures)
  2  problems with the input file (file not found, buffer modified,
     I/O errors)
  3  internal consistency error (for example a bug) which caused
     ed-rs to panic
"
    )
}

fn version_text() -> String {
    format!(
        "\
ed-rs {}
Clean-room Rust port of the GNU ed line editor.
Credited to Bee, Grok Bot, and Nori.
License GPL-3.0-or-later.
",
        env!("CARGO_PKG_VERSION")
    )
}

mod code {
    pub const EXTENDED: i32 = b'E' as i32;
    pub const TRADITIONAL: i32 = b'G' as i32;
    pub const HELP: i32 = b'h' as i32;
    pub const LOOSE: i32 = b'l' as i32;
    pub const PROMPT: i32 = b'p' as i32;
    pub const QUIET: i32 = b'q' as i32;
    pub const RESTRICTED: i32 = b'r' as i32;
    pub const SCRIPT: i32 = b's' as i32;
    pub const VERBOSE: i32 = b'v' as i32;
    pub const VERSION: i32 = b'V' as i32;
    pub const STRIP_CR: i32 = 256;
    pub const UNSAFE_NAMES: i32 = 257;
}

struct Spec {
    code: i32,
    long: &'static str,
    takes_arg: bool,
}

/// GNU ed 1.22.4 option table, in the same order.
const SPECS: &[Spec] = &[
    Spec {
        code: code::EXTENDED,
        long: "extended-regexp",
        takes_arg: false,
    },
    Spec {
        code: code::TRADITIONAL,
        long: "traditional",
        takes_arg: false,
    },
    Spec {
        code: code::HELP,
        long: "help",
        takes_arg: false,
    },
    Spec {
        code: code::LOOSE,
        long: "loose-exit-status",
        takes_arg: false,
    },
    Spec {
        code: code::PROMPT,
        long: "prompt",
        takes_arg: true,
    },
    Spec {
        code: code::QUIET,
        long: "quiet",
        takes_arg: false,
    },
    Spec {
        code: code::QUIET,
        long: "silent",
        takes_arg: false,
    },
    Spec {
        code: code::RESTRICTED,
        long: "restricted",
        takes_arg: false,
    },
    Spec {
        code: code::SCRIPT,
        long: "script",
        takes_arg: false,
    },
    Spec {
        code: code::VERBOSE,
        long: "verbose",
        takes_arg: false,
    },
    Spec {
        code: code::VERSION,
        long: "version",
        takes_arg: false,
    },
    Spec {
        code: code::STRIP_CR,
        long: "strip-trailing-cr",
        takes_arg: false,
    },
    Spec {
        code: code::UNSAFE_NAMES,
        long: "unsafe-names",
        takes_arg: false,
    },
];

struct OptRec {
    code: i32,
    argument: Option<String>,
}

fn split_args(argv: &[String]) -> Result<(Vec<OptRec>, Vec<String>), String> {
    let mut index = 1;
    let mut options = Vec::new();
    let mut operands = Vec::new();
    if argv.is_empty() {
        return Ok((options, operands));
    }
    while index < argv.len() {
        let arg = &argv[index];
        if arg == "--" {
            index += 1;
            break;
        } else if arg.starts_with("--") {
            index = parse_long(argv, index, &mut options)?;
        } else if arg.starts_with('-') && arg.len() > 1 {
            index = parse_short(argv, index, &mut options)?;
        } else {
            operands.push(arg.clone());
            index += 1;
        }
    }
    while index < argv.len() {
        operands.push(argv[index].clone());
        index += 1;
    }
    Ok((options, operands))
}

fn lookup_long(name: &str) -> Result<&'static Spec, LongError> {
    let mut found: Option<&Spec> = None;
    let mut ambiguous = false;
    for spec in SPECS {
        if !spec.long.starts_with(name) {
            continue;
        }
        if spec.long == name {
            // An exact match wins, even if a shorter prefix was ambiguous.
            return Ok(spec);
        }
        match found {
            None => found = Some(spec),
            Some(prev) if prev.code != spec.code || prev.takes_arg != spec.takes_arg => {
                ambiguous = true;
            }
            Some(_) => {}
        }
    }
    if ambiguous {
        Err(LongError::Ambiguous)
    } else if let Some(spec) = found {
        Ok(spec)
    } else {
        Err(LongError::Unknown)
    }
}

enum LongError {
    Unknown,
    Ambiguous,
}

fn parse_long(argv: &[String], index: usize, out: &mut Vec<OptRec>) -> Result<usize, String> {
    let opt = &argv[index];
    let body = &opt[2..];
    let (name, inline) = match body.split_once('=') {
        Some((name, value)) => (name, Some(value)),
        None => (body, None),
    };
    let spec = match lookup_long(name) {
        Ok(spec) => spec,
        Err(LongError::Unknown) => {
            return Err(format!("unrecognized option '{opt}'"));
        }
        Err(LongError::Ambiguous) => {
            return Err(format!("option '{opt}' is ambiguous"));
        }
    };
    let (argument, next) = take_long_argument(spec, inline, argv, index)?;
    out.push(OptRec {
        code: spec.code,
        argument,
    });
    Ok(next)
}

fn take_long_argument(
    spec: &Spec,
    inline: Option<&str>,
    argv: &[String],
    index: usize,
) -> Result<(Option<String>, usize), String> {
    if let Some(value) = inline {
        if !spec.takes_arg {
            return Err(format!(
                "option '--{}' doesn't allow an argument",
                spec.long
            ));
        }
        if value.is_empty() {
            return Err(format!("option '--{}' requires an argument", spec.long));
        }
        return Ok((Some(value.to_string()), index + 1));
    }
    if !spec.takes_arg {
        return Ok((None, index + 1));
    }
    match argv.get(index + 1) {
        Some(value) if !value.is_empty() => Ok((Some(value.clone()), index + 2)),
        _ => Err(format!("option '--{}' requires an argument", spec.long)),
    }
}

fn parse_short(argv: &[String], index: usize, out: &mut Vec<OptRec>) -> Result<usize, String> {
    let opt = &argv[index];
    let bytes = opt.as_bytes();
    let mut cind: usize = 1;
    let mut argind = index;
    while cind > 0 {
        let letter = bytes[cind];
        let spec = match SPECS.iter().find(|spec| spec.code == i32::from(letter)) {
            Some(spec) => spec,
            None => {
                return Err(format!("invalid option -- '{}'", letter as char));
            }
        };
        let mut argument = None;
        if cind + 1 >= bytes.len() {
            argind += 1;
            cind = 0;
        } else {
            cind += 1;
        }
        if spec.takes_arg && cind > 0 {
            // The rest of this cluster is the argument (`-pSTRING`).
            argument = Some(opt[cind..].to_string());
            argind += 1;
            cind = 0;
        } else if spec.takes_arg {
            match argv.get(argind) {
                Some(value) if !value.is_empty() => {
                    argument = Some(value.clone());
                    argind += 1;
                }
                _ => {
                    return Err(format!(
                        "option requires an argument -- '{}'",
                        letter as char
                    ));
                }
            }
        }
        out.push(OptRec {
            code: spec.code,
            argument,
        });
    }
    Ok(argind)
}

enum StartKind {
    Regexp(StartRegexp),
    Line(i32),
    BadNumber,
    BadAddress,
}

fn classify_start(rest: &str) -> StartKind {
    match rest.as_bytes().first().copied() {
        Some(b'/') => StartKind::Regexp(StartRegexp::Forward(rest[1..].to_string())),
        Some(b'?') => StartKind::Regexp(StartRegexp::Backward(rest[1..].to_string())),
        Some(byte) if byte.is_ascii_digit() => match parse_line_number(rest) {
            Ok(line) => StartKind::Line(line),
            Err(()) => StartKind::BadNumber,
        },
        _ => StartKind::BadAddress,
    }
}

/// C `strtol` base 10 on text that already starts with a digit.
/// Trailing non-digits are ignored. Overflow and values outside
/// `1..=i32::MAX` are errors.
fn parse_line_number(text: &str) -> Result<i32, ()> {
    let mut value: i64 = 0;
    let mut saw_digit = false;
    for byte in text.bytes() {
        if !byte.is_ascii_digit() {
            break;
        }
        saw_digit = true;
        let digit = i64::from(byte - b'0');
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add(digit))
            .ok_or(())?;
    }
    if !saw_digit {
        return Err(());
    }
    if (1..=i64::from(i32::MAX)).contains(&value) {
        Ok(value as i32)
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(invocation: &str, args: &[&str]) -> Action {
        let argv =
            std::iter::once(invocation.to_string()).chain(args.iter().copied().map(str::to_string));
        parse_args(argv)
    }

    fn run(args: &[&str]) -> Options {
        match parse("ed-rs", args) {
            Action::Run(opts) => opts,
            other => panic!("expected Run, got {other:?}"),
        }
    }

    fn err(args: &[&str]) -> String {
        match parse("ed-rs", args) {
            Action::Error(text) => text,
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn no_arguments_are_defaults() {
        assert_eq!(run(&[]), Options::default());
    }

    #[test]
    fn short_and_long_options() {
        let opts = run(&[
            "-E",
            "-G",
            "-l",
            "-v",
            "-r",
            "--strip-trailing-cr",
            "--unsafe-names",
        ]);
        assert!(opts.extended_regexp);
        assert!(opts.traditional);
        assert!(opts.loose_exit);
        assert!(opts.verbose);
        assert!(opts.restricted);
        assert!(opts.strip_trailing_cr);
        assert!(opts.unsafe_names);
        assert!(!opts.silent);
        assert!(!opts.quiet);

        let long = run(&[
            "--extended-regexp",
            "--traditional",
            "--loose-exit-status",
            "--verbose",
            "--restricted",
            "--script",
            "--quiet",
        ]);
        assert!(long.extended_regexp);
        assert!(long.traditional);
        assert!(long.loose_exit);
        assert!(long.verbose);
        assert!(long.restricted);
        assert!(long.silent);
        assert!(long.quiet);
    }

    #[test]
    fn long_prefixes_and_ambiguity() {
        assert!(run(&["--scr"]).silent);
        assert!(!run(&["--scr"]).quiet);
        assert!(run(&["--sil"]).quiet);
        assert!(!run(&["--sil"]).silent);
        assert!(run(&["--qui"]).quiet);
        assert!(run(&["--stri"]).strip_trailing_cr);
        assert!(run(&["--uns"]).unsafe_names);
        assert!(run(&["--trad"]).traditional);
        assert!(run(&["--loose"]).loose_exit);
        assert!(run(&["--ext"]).extended_regexp);
        assert!(run(&["--rest"]).restricted);
        assert!(run(&["--verb"]).verbose);
        assert_eq!(
            err(&["--s"]),
            "ed-rs: option '--s' is ambiguous\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["--ver"]),
            "ed-rs: option '--ver' is ambiguous\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["--script=no"]),
            "ed-rs: option '--script' doesn't allow an argument\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["--stri=1"]),
            "ed-rs: option '--strip-trailing-cr' doesn't allow an argument\n\
             Try 'ed-rs --help' for more information.\n"
        );
    }

    #[test]
    fn combined_short_flags() {
        let opts = run(&["-qs"]);
        assert!(opts.quiet);
        assert!(opts.silent);
        let opts = run(&["-GlvrE"]);
        assert!(opts.traditional);
        assert!(opts.loose_exit);
        assert!(opts.verbose);
        assert!(opts.restricted);
        assert!(opts.extended_regexp);
        assert!(!opts.quiet);
        assert!(!opts.silent);
        // `-p` consumes the rest of the cluster, so `-ps` is a prompt.
        let opts = run(&["-ps"]);
        assert_eq!(opts.prompt.as_deref(), Some("s"));
        assert!(!opts.silent);
        let opts = run(&["-sqp>"]);
        assert!(opts.silent);
        assert!(opts.quiet);
        assert_eq!(opts.prompt.as_deref(), Some(">"));
    }

    #[test]
    fn prompt_forms() {
        assert_eq!(run(&["-p", "hello"]).prompt.as_deref(), Some("hello"));
        assert_eq!(run(&["-phello"]).prompt.as_deref(), Some("hello"));
        assert_eq!(run(&["--prompt", "hello"]).prompt.as_deref(), Some("hello"));
        assert_eq!(run(&["--prompt=hello"]).prompt.as_deref(), Some("hello"));
        assert_eq!(run(&["--pro=hi"]).prompt.as_deref(), Some("hi"));
        assert_eq!(run(&["-p", "-s"]).prompt.as_deref(), Some("-s"));
        assert!(!run(&["-p", "-s"]).silent);
        // The last prompt wins.
        assert_eq!(
            run(&["-p", "one", "--prompt=two"]).prompt.as_deref(),
            Some("two")
        );
    }

    #[test]
    fn missing_prompt_argument() {
        assert_eq!(
            err(&["-p"]),
            "ed-rs: option requires an argument -- 'p'\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["-qp"]),
            "ed-rs: option requires an argument -- 'p'\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["--prompt"]),
            "ed-rs: option '--prompt' requires an argument\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["--prompt="]),
            "ed-rs: option '--prompt' requires an argument\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["--p"]),
            "ed-rs: option '--prompt' requires an argument\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["-p", ""]),
            "ed-rs: option requires an argument -- 'p'\n\
             Try 'ed-rs --help' for more information.\n"
        );
    }

    #[test]
    fn unknown_options() {
        assert_eq!(
            err(&["-Z"]),
            "ed-rs: invalid option -- 'Z'\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["-qZ"]),
            "ed-rs: invalid option -- 'Z'\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["--nope"]),
            "ed-rs: unrecognized option '--nope'\n\
             Try 'ed-rs --help' for more information.\n"
        );
        assert_eq!(
            err(&["--nope=1"]),
            "ed-rs: unrecognized option '--nope=1'\n\
             Try 'ed-rs --help' for more information.\n"
        );
        // A bad option is reported even when help or -q appears beside it,
        // and -l does not turn the failure into success.
        assert!(matches!(
            parse("ed-rs", &["-h", "--nope"]),
            Action::Error(_)
        ));
        assert!(matches!(parse("ed-rs", &["-q", "-Z"]), Action::Error(_)));
        assert!(matches!(parse("ed-rs", &["-l", "-Z"]), Action::Error(_)));
    }

    #[test]
    fn invocation_name_in_diagnostics() {
        let text = match parse("/usr/local/bin/ed-rs", &["--nope"]) {
            Action::Error(text) => text,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            text,
            "ed-rs: unrecognized option '--nope'\n\
             Try '/usr/local/bin/ed-rs --help' for more information.\n"
        );
        let text = match parse("./ed-rs", &["-Z"]) {
            Action::Error(text) => text,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            text,
            "ed-rs: invalid option -- 'Z'\n\
             Try './ed-rs --help' for more information.\n"
        );
        let help = match parse("/opt/bin/ed-rs", &["-h"]) {
            Action::Help(text) => text,
            other => panic!("{other:?}"),
        };
        assert!(help.contains("Usage: /opt/bin/ed-rs [options] [[+line] file]"));
        let tagged = match parse("/opt/bin/ed-rs", &["+foo"]) {
            Action::Error(text) => text,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            tagged,
            "ed-rs: +foo: Invalid line number or regular expression.\n"
        );
    }

    #[test]
    fn help_and_version_exit_zero_and_list_flags() {
        let help = match parse("ed-rs", &["-h"]) {
            Action::Help(text) => text,
            other => panic!("{other:?}"),
        };
        let help_long = match parse("ed-rs", &["--help"]) {
            Action::Help(text) => text,
            other => panic!("{other:?}"),
        };
        assert_eq!(help, help_long);
        assert!(help.ends_with('\n'));
        for flag in [
            "-h",
            "--help",
            "-V",
            "--version",
            "-E",
            "--extended-regexp",
            "-G",
            "--traditional",
            "-l",
            "--loose-exit-status",
            "-p",
            "--prompt",
            "-q",
            "--quiet",
            "--silent",
            "-r",
            "--restricted",
            "-s",
            "--script",
            "-v",
            "--verbose",
            "--strip-trailing-cr",
            "--unsafe-names",
        ] {
            assert!(help.contains(flag), "help missing {flag}");
        }
        assert!(help.contains("ed-rs"));
        assert!(help.contains("GPL-3.0-or-later"));
        assert!(help.contains("Bee"));
        assert!(help.contains("Grok Bot"));
        assert!(help.contains("Nori"));
        assert!(help.contains("normal exit"));
        assert!(help.contains("invalid command-line options"));
        assert!(help.contains("command failed"));
        assert!(help.contains("problems with the input file"));
        assert!(help.contains("internal consistency error"));
        assert!(help.contains("panic"));
        assert!(!help.contains("Andrew"));
        assert!(!help.contains("Antonio"));
        assert!(!help.contains("bug-ed@gnu.org"));

        let version = match parse("ed-rs", &["-V"]) {
            Action::Version(text) => text,
            other => panic!("{other:?}"),
        };
        let version_long = match parse("ed-rs", &["--version"]) {
            Action::Version(text) => text,
            other => panic!("{other:?}"),
        };
        assert_eq!(version, version_long);
        assert!(version.contains(env!("CARGO_PKG_VERSION")));
        assert!(version.contains("ed-rs"));
        assert!(version.contains("GPL-3.0-or-later"));
        assert!(version.contains("Bee, Grok Bot, and Nori"));
        assert!(version.ends_with('\n'));
        assert!(!version.contains("Andrew"));
        assert!(!version.contains("Antonio"));

        // The first of help and version in argv order wins.
        assert!(matches!(parse("ed-rs", &["-hV"]), Action::Help(_)));
        assert!(matches!(parse("ed-rs", &["-Vh"]), Action::Version(_)));
        assert!(matches!(
            parse("ed-rs", &["--help", "--version"]),
            Action::Help(_)
        ));
        assert!(matches!(
            parse("ed-rs", &["--version", "--help"]),
            Action::Version(_)
        ));
        // Help is chosen before a later operand is validated.
        assert!(matches!(parse("ed-rs", &["-h", "+foo"]), Action::Help(_)));
        // `-v` is verbose, not version.
        assert!(run(&["-v"]).verbose);
        assert!(!matches!(parse("ed-rs", &["-v"]), Action::Version(_)));
    }

    #[test]
    fn file_start_address_and_bang() {
        assert_eq!(run(&["notes.txt"]).file.as_deref(), Some("notes.txt"));
        let opts = run(&["+42", "notes.txt"]);
        assert_eq!(opts.start_line, Some(42));
        assert_eq!(opts.file.as_deref(), Some("notes.txt"));
        assert!(opts.start_regexp.is_none());

        let opts = run(&["+/foo", "notes.txt"]);
        assert_eq!(
            opts.start_regexp,
            Some(StartRegexp::Forward("foo".to_string()))
        );
        assert_eq!(opts.file.as_deref(), Some("notes.txt"));

        let opts = run(&["+?bar", "notes.txt"]);
        assert_eq!(
            opts.start_regexp,
            Some(StartRegexp::Backward("bar".to_string()))
        );

        let opts = run(&["+/"]);
        assert_eq!(opts.start_regexp, Some(StartRegexp::Forward(String::new())));
        let opts = run(&["+?"]);
        assert_eq!(
            opts.start_regexp,
            Some(StartRegexp::Backward(String::new()))
        );

        // Trailing junk is ignored, as with strtol. Leading zeros are decimal.
        assert_eq!(run(&["+12abc"]).start_line, Some(12));
        assert_eq!(run(&["+007"]).start_line, Some(7));
        assert_eq!(run(&["+1a2"]).start_line, Some(1));
        assert_eq!(run(&["+2147483647"]).start_line, Some(i32::MAX));

        // Last of each kind wins, and neither clears the other.
        let opts = run(&["+/one", "+4", "+?two", "+9"]);
        assert_eq!(opts.start_line, Some(9));
        assert_eq!(
            opts.start_regexp,
            Some(StartRegexp::Backward("two".to_string()))
        );

        // Extra operands after the file are ignored, including a later `+`.
        let opts = run(&["+2", "notes.txt", "extra", "+3"]);
        assert_eq!(opts.start_line, Some(2));
        assert_eq!(opts.file.as_deref(), Some("notes.txt"));

        let opts = run(&["!ls"]);
        assert_eq!(opts.file.as_deref(), Some("!ls"));
        assert!(!opts.silent);
        let opts = run(&["-p", ">", "!ls"]);
        assert_eq!(opts.prompt.as_deref(), Some(">"));
        assert_eq!(opts.file.as_deref(), Some("!ls"));
    }

    #[test]
    fn invalid_start_address() {
        assert_eq!(
            err(&["+foo"]),
            "ed-rs: +foo: Invalid line number or regular expression.\n"
        );
        assert_eq!(
            err(&["+"]),
            "ed-rs: +: Invalid line number or regular expression.\n"
        );
        assert_eq!(
            err(&["+0"]),
            "ed-rs: 0: Invalid line number; must be >= 1.\n"
        );
        assert_eq!(
            err(&["+00"]),
            "ed-rs: 00: Invalid line number; must be >= 1.\n"
        );
        assert_eq!(
            err(&["+2147483648"]),
            "ed-rs: 2147483648: Invalid line number; must be >= 1.\n"
        );
        let huge = format!("+{}", "9".repeat(40));
        let message = err(&[huge.as_str()]);
        assert!(message.starts_with("ed-rs: "));
        assert!(message.contains("Invalid line number; must be >= 1."));
        // Quiet suppresses the text and still fails. Script mode does not.
        assert_eq!(err(&["-q", "+foo"]), "");
        assert_eq!(err(&["+foo", "-q"]), "");
        assert!(err(&["-s", "+foo"]).contains("Invalid line number"));
        assert!(matches!(parse("ed-rs", &["-l", "+foo"]), Action::Error(_)));
    }

    #[test]
    fn hyphen_operand_sets_script_mode() {
        let opts = run(&["-"]);
        assert!(opts.silent);
        assert!(!opts.quiet);
        assert!(opts.file.is_none());

        let opts = run(&["-", "notes.txt"]);
        assert!(opts.silent);
        assert_eq!(opts.file.as_deref(), Some("notes.txt"));

        let opts = run(&["+3", "-", "notes.txt"]);
        assert!(opts.silent);
        assert_eq!(opts.start_line, Some(3));
        assert_eq!(opts.file.as_deref(), Some("notes.txt"));

        // A hyphen after the file is an extra operand, not script mode.
        let opts = run(&["notes.txt", "-"]);
        assert!(!opts.silent);
        assert_eq!(opts.file.as_deref(), Some("notes.txt"));

        // `--` does not turn `-` into a file name.
        let opts = run(&["--", "-"]);
        assert!(opts.silent);
        assert!(opts.file.is_none());
        let opts = run(&["--", "-", "notes.txt"]);
        assert!(opts.silent);
        assert_eq!(opts.file.as_deref(), Some("notes.txt"));
    }

    #[test]
    fn quiet_and_script_are_distinct() {
        let quiet = run(&["-q"]);
        assert!(quiet.quiet);
        assert!(!quiet.silent);
        let script = run(&["-s"]);
        assert!(script.silent);
        assert!(!script.quiet);
        let alias = run(&["--silent"]);
        assert!(alias.quiet);
        assert!(!alias.silent);
        let both = run(&["-q", "-s"]);
        assert!(both.quiet);
        assert!(both.silent);
        let both = run(&["--quiet", "--script"]);
        assert!(both.quiet);
        assert!(both.silent);
    }

    #[test]
    fn options_permute_until_double_dash() {
        let opts = run(&["notes.txt", "-s"]);
        assert!(opts.silent);
        assert_eq!(opts.file.as_deref(), Some("notes.txt"));

        let opts = run(&["notes.txt", "--", "-s"]);
        assert!(!opts.silent);
        assert_eq!(opts.file.as_deref(), Some("notes.txt"));

        let opts = run(&["--", "--help"]);
        assert_eq!(opts.file.as_deref(), Some("--help"));
        let opts = run(&["--", "-q", "notes.txt"]);
        assert!(!opts.quiet);
        assert_eq!(opts.file.as_deref(), Some("-q"));
    }
}
