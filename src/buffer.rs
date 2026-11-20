#[derive(Debug, Default)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub current: usize,
    pub modified: bool,
}
