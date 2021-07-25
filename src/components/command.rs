#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct CommandText {
    pub name: String,
    pub desc: &'static str,
    pub group: &'static str,
    pub hide_help: bool,
}
pub struct CommandInfo {
    pub text: CommandText,
    pub enabled: bool,
    pub quick_bar: bool,
    pub available: bool,
    pub order: i8,
}
