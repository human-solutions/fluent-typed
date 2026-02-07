#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    #[default]
    String,
    Pattern,
    Both,
}
