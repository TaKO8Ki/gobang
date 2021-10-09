use crate::config::KeyConfig;

static CMD_GROUP_GENERAL: &str = "-- General --";
static CMD_GROUP_TABLE: &str = "-- Table --";
static CMD_GROUP_DATABASES: &str = "-- Databases --";
static CMD_GROUP_PROPERTIES: &str = "-- Properties --";

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct CommandText {
    pub name: String,
    pub group: &'static str,
    pub hide_help: bool,
}

impl CommandText {
    pub const fn new(name: String, group: &'static str) -> Self {
        Self {
            name,
            group,
            hide_help: false,
        }
    }
}

pub struct CommandInfo {
    pub text: CommandText,
}

impl CommandInfo {
    pub const fn new(text: CommandText) -> Self {
        Self { text }
    }
}

pub fn scroll(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Scroll up/down/left/right [{},{},{},{}]",
            key.scroll_up, key.scroll_down, key.scroll_left, key.scroll_right
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn scroll_up_down_multiple_lines(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Scroll up/down multiple lines [{},{}]",
            key.scroll_up_multiple_lines, key.scroll_down_multiple_lines,
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn scroll_to_top_bottom(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Scroll to top/bottom [{},{}]",
            key.scroll_to_top, key.scroll_to_bottom,
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn expand_collapse(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Expand/Collapse [{},{}]", key.scroll_right, key.scroll_left,),
        CMD_GROUP_DATABASES,
    )
}

pub fn filter(key: &KeyConfig) -> CommandText {
    CommandText::new(format!("Filter [{}]", key.filter), CMD_GROUP_GENERAL)
}

pub fn move_focus(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Move focus to left/right [{},{}]",
            key.focus_left, key.focus_right
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn extend_selection_by_one_cell(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Extend selection by one cell up/down/left/right [{},{},{},{}]",
            key.extend_selection_by_one_cell_up,
            key.extend_selection_by_one_cell_down,
            key.extend_selection_by_one_cell_left,
            key.extend_selection_by_one_cell_right
        ),
        CMD_GROUP_TABLE,
    )
}

pub fn extend_or_shorten_widget_width(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Extend/shorten widget width to left/right [{},{}]",
            key.extend_or_shorten_widget_width_to_left, key.extend_or_shorten_widget_width_to_right
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn tab_records(key: &KeyConfig) -> CommandText {
    CommandText::new(format!("Records [{}]", key.tab_records), CMD_GROUP_TABLE)
}

pub fn tab_columns(key: &KeyConfig) -> CommandText {
    CommandText::new(format!("Columns [{}]", key.tab_columns), CMD_GROUP_TABLE)
}

pub fn tab_constraints(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Constraints [{}]", key.tab_constraints),
        CMD_GROUP_TABLE,
    )
}

pub fn tab_foreign_keys(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Foreign keys [{}]", key.tab_foreign_keys),
        CMD_GROUP_TABLE,
    )
}

pub fn tab_indexes(key: &KeyConfig) -> CommandText {
    CommandText::new(format!("Indexes [{}]", key.tab_indexes), CMD_GROUP_TABLE)
}

pub fn tab_sql_editor(key: &KeyConfig) -> CommandText {
    CommandText::new(format!("SQL [{}]", key.tab_sql_editor), CMD_GROUP_TABLE)
}

pub fn tab_properties(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Properties [{}]", key.tab_properties),
        CMD_GROUP_TABLE,
    )
}

pub fn toggle_tabs(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Tab [{},{},{}]",
            key_config.tab_records, key_config.tab_properties, key_config.tab_sql_editor
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn toggle_property_tabs(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Tab [{},{},{},{}]",
            key_config.tab_columns,
            key_config.tab_constraints,
            key_config.tab_foreign_keys,
            key_config.tab_indexes
        ),
        CMD_GROUP_PROPERTIES,
    )
}

pub fn help(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Help [{}]", key_config.open_help),
        CMD_GROUP_GENERAL,
    )
}
