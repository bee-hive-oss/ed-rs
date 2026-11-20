#[derive(Debug, Default)]
pub struct Options {
    pub silent: bool,
    pub prompt: Option<String>,
    pub traditional: bool,
    pub loose_exit: bool,
    pub file: Option<String>,
}

pub fn parse_args<I: IntoIterator<Item = String>>(_args: I) -> Options {
    Options::default()
}
