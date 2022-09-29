use crate::config::KeyConfig;
use crate::event::Key;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct KeyBind {
    pub scroll_up: Option<Key>,
    pub scroll_down: Option<Key>,
    pub scroll_right: Option<Key>,
    pub scroll_left: Option<Key>,
    pub move_up: Option<Key>,
    pub move_down: Option<Key>,
    pub copy: Option<Key>,
    pub enter: Option<Key>,
    pub exit: Option<Key>,
    pub quit: Option<Key>,
    pub exit_popup: Option<Key>,
    pub focus_right: Option<Key>,
    pub focus_left: Option<Key>,
    pub focus_above: Option<Key>,
    pub focus_connections: Option<Key>,
    pub open_help: Option<Key>,
    pub filter: Option<Key>,
    pub scroll_down_multiple_lines: Option<Key>,
    pub scroll_up_multiple_lines: Option<Key>,
    pub scroll_to_top: Option<Key>,
    pub scroll_to_bottom: Option<Key>,
    pub extend_selection_by_one_cell_left: Option<Key>,
    pub extend_selection_by_one_cell_right: Option<Key>,
    pub extend_selection_by_one_cell_up: Option<Key>,
    pub extend_selection_by_one_cell_down: Option<Key>,
    pub tab_records: Option<Key>,
    pub tab_columns: Option<Key>,
    pub tab_constraints: Option<Key>,
    pub tab_foreign_keys: Option<Key>,
    pub tab_indexes: Option<Key>,
    pub tab_sql_editor: Option<Key>,
    pub tab_properties: Option<Key>,
    pub extend_or_shorten_widget_width_to_right: Option<Key>,
    pub extend_or_shorten_widget_width_to_left: Option<Key>,
}

impl KeyBind {
    pub fn load(config_path: PathBuf) -> anyhow::Result<Self> {
        if let Ok(file) = File::open(config_path) {
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;
            let key_bind: Result<KeyBind, ron::Error> = ron::from_str(&contents);
            match key_bind {
                Ok(key_bind) => return Ok(key_bind),
                Err(e) => panic!("fail to parse key bind file: {}", e),
            }
        }

        return Ok(Self::default());
    }
}

macro_rules! merge {
    ($kc:expr, $kt:expr) => {
        $kc = $kt.unwrap_or_else(|| $kc)
    };
}

impl From<KeyBind> for KeyConfig {
    #[allow(clippy::field_reassign_with_default)]
    fn from(kb: KeyBind) -> Self {
        let mut kc = KeyConfig::default();
        merge!(kc.scroll_up, kb.scroll_up);
        merge!(kc.scroll_down, kb.scroll_down);
        merge!(kc.scroll_right, kb.scroll_right);
        merge!(kc.scroll_left, kb.scroll_left);
        merge!(kc.scroll_down, kb.scroll_down);
        merge!(kc.move_up, kb.move_up);
        merge!(kc.move_down, kb.move_down);
        merge!(kc.copy, kb.copy);
        merge!(kc.enter, kb.enter);
        merge!(kc.exit, kb.exit);
        merge!(kc.quit, kb.quit);
        merge!(kc.exit_popup, kb.exit_popup);
        merge!(kc.focus_right, kb.focus_right);
        merge!(kc.focus_left, kb.focus_left);
        merge!(kc.focus_above, kb.focus_above);
        merge!(kc.focus_connections, kb.focus_connections);
        merge!(kc.open_help, kb.open_help);
        merge!(kc.filter, kb.filter);
        merge!(kc.scroll_down_multiple_lines, kb.scroll_down_multiple_lines);
        merge!(kc.scroll_up_multiple_lines, kb.scroll_up_multiple_lines);
        merge!(kc.scroll_to_top, kb.scroll_to_top);
        merge!(kc.scroll_to_bottom, kb.scroll_to_bottom);
        merge!(
            kc.extend_selection_by_one_cell_left,
            kb.extend_selection_by_one_cell_left
        );
        merge!(
            kc.extend_selection_by_one_cell_right,
            kb.extend_selection_by_one_cell_right
        );
        merge!(
            kc.extend_selection_by_one_cell_down,
            kb.extend_selection_by_one_cell_down
        );
        merge!(
            kc.extend_selection_by_one_cell_up,
            kb.extend_selection_by_one_cell_up
        );
        merge!(kc.tab_records, kb.tab_records);
        merge!(kc.tab_properties, kb.tab_properties);
        merge!(kc.tab_sql_editor, kb.tab_sql_editor);
        merge!(kc.tab_columns, kb.tab_columns);
        merge!(kc.tab_constraints, kb.tab_constraints);
        merge!(kc.tab_foreign_keys, kb.tab_foreign_keys);
        merge!(kc.tab_indexes, kb.tab_indexes);
        merge!(
            kc.extend_or_shorten_widget_width_to_right,
            kb.extend_or_shorten_widget_width_to_right
        );
        merge!(
            kc.extend_or_shorten_widget_width_to_left,
            kb.extend_or_shorten_widget_width_to_left
        );
        kc
    }
}

#[cfg(test)]
mod test {
    use super::KeyBind;
    use crate::config::KeyConfig;
    use crate::event::Key;
    use std::path::Path;

    #[test]
    fn test_exist_file() {
        let config_path = Path::new("examples/key_bind.ron").to_path_buf();
        assert_eq!(config_path.exists(), true);
        assert_eq!(KeyBind::load(config_path).is_ok(), true);
    }

    #[test]
    fn test_not_exist_file() {
        let config_path = Path::new("examples/not_exist.ron").to_path_buf();
        assert_eq!(config_path.exists(), false);
        assert_eq!(KeyBind::load(config_path).is_ok(), true);
    }

    #[test]
    fn test_key_config_from_key_bind() {
        // Default Config
        let empty_kb = KeyBind::default();
        let kc = KeyConfig::default();
        assert_eq!(KeyConfig::from(empty_kb), kc);

        // Merged Config
        let mut kb = KeyBind::default();
        kb.scroll_up = Some(Key::Char('M'));
        let build_kc = KeyConfig::from(kb);
        assert_eq!(build_kc.scroll_up, Key::Char('M'));
    }
}
