static CMD_GROUP_GENERAL: &str = "-- General --";

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct CommandText {
    pub name: String,
    pub desc: &'static str,
    pub group: &'static str,
    pub hide_help: bool,
}

impl CommandText {
    pub const fn new(name: String, desc: &'static str, group: &'static str) -> Self {
        Self {
            name,
            desc,
            group,
            hide_help: false,
        }
    }
}

pub struct CommandInfo {
    pub text: CommandText,
    pub enabled: bool,
    pub quick_bar: bool,
    pub available: bool,
    pub order: i8,
}

impl CommandInfo {
    pub const fn new(text: CommandText, enabled: bool, available: bool) -> Self {
        Self {
            text,
            enabled,
            quick_bar: true,
            available,
            order: 0,
        }
    }

    pub const fn order(self, order: i8) -> Self {
        let mut res = self;
        res.order = order;
        res
    }
}

pub fn move_down(key: &str) -> CommandText {
    CommandText::new(
        format!("Move down [{}]", key),
        "move down",
        CMD_GROUP_GENERAL,
    )
}

pub fn move_up(key: &str) -> CommandText {
    CommandText::new(format!("Move up [{}]", key), "move up", CMD_GROUP_GENERAL)
}

pub fn move_right(key: &str) -> CommandText {
    CommandText::new(
        format!("Move right [{}]", key),
        "move right",
        CMD_GROUP_GENERAL,
    )
}

pub fn move_left(key: &str) -> CommandText {
    CommandText::new(
        format!("Move left [{}]", key),
        "move left",
        CMD_GROUP_GENERAL,
    )
}

pub fn filter(key: &str) -> CommandText {
    CommandText::new(
        format!("Filter [{}]", key),
        "enter input for filter",
        CMD_GROUP_GENERAL,
    )
}
