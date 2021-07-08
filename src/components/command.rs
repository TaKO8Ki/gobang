#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct CommandText {
    ///
    pub name: String,
    ///
    pub desc: &'static str,
    ///
    pub group: &'static str,
    ///
    pub hide_help: bool,
}
pub struct CommandInfo {
    ///
    pub text: CommandText,
    /// available but not active in the context
    pub enabled: bool,
    /// will show up in the quick bar
    pub quick_bar: bool,

    /// available in current app state
    pub available: bool,
    /// used to order commands in quickbar
    pub order: i8,
}
