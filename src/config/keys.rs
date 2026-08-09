use crossterm::event::{KeyCode, KeyModifiers};
use serde::Deserialize;
use std::collections::HashMap;

/// Represents a key binding with optional modifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub const fn key(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    pub const fn ctrl(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::CONTROL)
    }

    pub const fn alt(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::ALT)
    }
}

/// Parse a key string like "ctrl+c", "alt+s", "shift+H", "q", "enter", etc.
fn parse_key_string(s: &str) -> Option<KeyBinding> {
    let s = s.trim().to_lowercase();
    let parts: Vec<&str> = s.split('+').collect();

    let (modifiers, key_part) = if parts.len() == 1 {
        (KeyModifiers::NONE, parts[0])
    } else {
        // Process modifiers
        let mut mods = KeyModifiers::NONE;
        for &part in &parts[..parts.len() - 1] {
            match part {
                "ctrl" | "control" => mods |= KeyModifiers::CONTROL,
                "alt" => mods |= KeyModifiers::ALT,
                "shift" => mods |= KeyModifiers::SHIFT,
                _ => return None, // Unknown modifier
            }
        }
        (mods, parts[parts.len() - 1])
    };

    let code = match key_part {
        "enter" | "return" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "backspace" | "bs" => KeyCode::Backspace,
        "tab" => KeyCode::Tab,
        "space" | " " => KeyCode::Char(' '),
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "pageup" | "pgup" => KeyCode::PageUp,
        "pagedown" | "pgdown" => KeyCode::PageDown,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "insert" | "ins" => KeyCode::Insert,
        "delete" | "del" => KeyCode::Delete,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "/" => KeyCode::Char('/'),
        "?" => KeyCode::Char('?'),
        s if s.len() == 1 => {
            let c = s.chars().next()?;
            // Handle shift modifier for uppercase letters
            if c.is_ascii_alphabetic() && modifiers.contains(KeyModifiers::SHIFT) {
                KeyCode::Char(c.to_ascii_uppercase())
            } else {
                KeyCode::Char(c)
            }
        }
        _ => return None,
    };

    Some(KeyBinding::new(code, modifiers))
}

/// Raw keybinding configuration from TOML
#[derive(Deserialize, Default, Debug, Clone)]
pub struct KeyBindingsConfig {
    // Normal mode
    pub quit: Option<Vec<String>>,
    pub navigate_up: Option<Vec<String>>,
    pub navigate_down: Option<Vec<String>>,
    pub enter: Option<Vec<String>>,
    pub go_back: Option<Vec<String>>,
    pub scroll_preview_up: Option<Vec<String>>,
    pub scroll_preview_down: Option<Vec<String>>,
    pub scroll_preview_page_up: Option<Vec<String>>,
    pub scroll_preview_page_down: Option<Vec<String>>,
    pub shrink_file_list: Option<Vec<String>>,
    pub grow_file_list: Option<Vec<String>>,
    pub open_search: Option<Vec<String>>,
    pub open_find: Option<Vec<String>>,
    pub open_symbol_search: Option<Vec<String>>,
    pub toggle_theme_preview: Option<Vec<String>>,
    pub toggle_help: Option<Vec<String>>,
    pub cycle_pane: Option<Vec<String>>,
    pub copy_selection: Option<Vec<String>>,
    pub select_all: Option<Vec<String>>,
    pub toggle_wrap: Option<Vec<String>>,

    // Split pane controls (Alt-based by default)
    pub swap_split_orientation: Option<Vec<String>>,
    pub close_active_pane: Option<Vec<String>>,
    pub toggle_file_list: Option<Vec<String>>,
    pub split_up: Option<Vec<String>>,
    pub split_down: Option<Vec<String>>,
    pub split_left: Option<Vec<String>>,
    pub split_right: Option<Vec<String>>,
    pub resize_split_left: Option<Vec<String>>,
    pub resize_split_right: Option<Vec<String>>,

    // Search mode
    pub search_close: Option<Vec<String>>,
    pub search_confirm: Option<Vec<String>>,
    pub search_backspace: Option<Vec<String>>,
    pub search_navigate_up: Option<Vec<String>>,
    pub search_navigate_down: Option<Vec<String>>,

    // PDF navigation
    pub pdf_next_page: Option<Vec<String>>,
    pub pdf_prev_page: Option<Vec<String>>,
    pub pdf_first_page: Option<Vec<String>>,
    pub pdf_last_page: Option<Vec<String>>,

    // Popup
    pub toggle_file_tree_popup: Option<Vec<String>>,

    // Git mode
    pub toggle_git_mode: Option<Vec<String>>,
}

/// Parsed keybindings ready for use
#[derive(Debug, Clone)]
pub struct KeyBindings {
    // Normal mode
    pub quit: Vec<KeyBinding>,
    pub navigate_up: Vec<KeyBinding>,
    pub navigate_down: Vec<KeyBinding>,
    pub enter: Vec<KeyBinding>,
    pub go_back: Vec<KeyBinding>,
    pub scroll_preview_up: Vec<KeyBinding>,
    pub scroll_preview_down: Vec<KeyBinding>,
    pub scroll_preview_page_up: Vec<KeyBinding>,
    pub scroll_preview_page_down: Vec<KeyBinding>,
    pub shrink_file_list: Vec<KeyBinding>,
    pub grow_file_list: Vec<KeyBinding>,
    pub open_search: Vec<KeyBinding>,
    pub open_find: Vec<KeyBinding>,
    pub open_symbol_search: Vec<KeyBinding>,
    pub toggle_theme_preview: Vec<KeyBinding>,
    pub toggle_help: Vec<KeyBinding>,
    pub cycle_pane: Vec<KeyBinding>,
    pub copy_selection: Vec<KeyBinding>,
    pub select_all: Vec<KeyBinding>,
    pub toggle_wrap: Vec<KeyBinding>,

    // Split pane controls
    pub swap_split_orientation: Vec<KeyBinding>,
    pub close_active_pane: Vec<KeyBinding>,
    pub toggle_file_list: Vec<KeyBinding>,
    pub split_up: Vec<KeyBinding>,
    pub split_down: Vec<KeyBinding>,
    pub split_left: Vec<KeyBinding>,
    pub split_right: Vec<KeyBinding>,
    pub resize_split_left: Vec<KeyBinding>,
    pub resize_split_right: Vec<KeyBinding>,

    // Search mode
    pub search_close: Vec<KeyBinding>,
    pub search_confirm: Vec<KeyBinding>,
    pub search_backspace: Vec<KeyBinding>,
    pub search_navigate_up: Vec<KeyBinding>,
    pub search_navigate_down: Vec<KeyBinding>,

    // PDF navigation
    pub pdf_next_page: Vec<KeyBinding>,
    pub pdf_prev_page: Vec<KeyBinding>,
    pub pdf_first_page: Vec<KeyBinding>,
    pub pdf_last_page: Vec<KeyBinding>,

    // Popup
    pub toggle_file_tree_popup: Vec<KeyBinding>,

    // Git mode
    pub toggle_git_mode: Vec<KeyBinding>,

    // Lookup table for quick matching
    normal_mode_map: HashMap<KeyBinding, Action>,
    search_mode_map: HashMap<KeyBinding, Action>,
}

/// Actions that can be triggered by key bindings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    // Normal mode
    Quit,
    NavigateUp,
    NavigateDown,
    Enter,
    GoBack,
    ScrollPreviewUp,
    ScrollPreviewDown,
    ScrollPreviewPageUp,
    ScrollPreviewPageDown,
    ShrinkFileList,
    GrowFileList,
    OpenSearch,
    OpenFind,
    OpenSymbolSearch,
    ToggleThemePreview,
    ToggleHelp,
    CyclePane,
    CopySelection,
    SelectAll,
    ToggleWrap,

    // Split pane controls
    SwapSplitOrientation,
    CloseActivePane,
    ToggleFileList,
    SplitUp,
    SplitDown,
    SplitLeft,
    SplitRight,
    ResizeSplitLeft,
    ResizeSplitRight,

    // Search mode
    SearchClose,
    SearchConfirm,
    SearchBackspace,
    SearchNavigateUp,
    SearchNavigateDown,

    // PDF navigation
    PdfNextPage,
    PdfPrevPage,
    PdfFirstPage,
    PdfLastPage,

    // Popup
    ToggleFileTreePopup,

    // Git mode
    ToggleGitMode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut bindings = Self {
            // Normal mode defaults
            quit: vec![
                KeyBinding::key(KeyCode::Char('q')),
                KeyBinding::key(KeyCode::Esc),
            ],
            navigate_up: vec![KeyBinding::key(KeyCode::Up)],
            navigate_down: vec![KeyBinding::key(KeyCode::Down)],
            enter: vec![
                KeyBinding::key(KeyCode::Enter),
                KeyBinding::key(KeyCode::Char('l')),
                KeyBinding::key(KeyCode::Right),
            ],
            go_back: vec![
                KeyBinding::key(KeyCode::Backspace),
                KeyBinding::key(KeyCode::Char('h')),
                KeyBinding::key(KeyCode::Left),
            ],
            scroll_preview_up: vec![KeyBinding::key(KeyCode::Char('k'))],
            scroll_preview_down: vec![KeyBinding::key(KeyCode::Char('j'))],
            scroll_preview_page_up: vec![KeyBinding::key(KeyCode::PageUp)],
            scroll_preview_page_down: vec![KeyBinding::key(KeyCode::PageDown)],
            shrink_file_list: vec![
                KeyBinding::new(KeyCode::Char('H'), KeyModifiers::SHIFT),
                KeyBinding::new(KeyCode::Char('h'), KeyModifiers::SHIFT),
                KeyBinding::key(KeyCode::Char('H')),
            ],
            grow_file_list: vec![
                KeyBinding::new(KeyCode::Char('L'), KeyModifiers::SHIFT),
                KeyBinding::new(KeyCode::Char('l'), KeyModifiers::SHIFT),
                KeyBinding::key(KeyCode::Char('L')),
            ],
            open_search: vec![KeyBinding::key(KeyCode::Char('/'))],
            open_find: vec![
                KeyBinding::key(KeyCode::Char('\\')),
                KeyBinding::key(KeyCode::Char('F')),
                KeyBinding::new(KeyCode::Char('F'), KeyModifiers::SHIFT),
                KeyBinding::new(KeyCode::Char('f'), KeyModifiers::SHIFT),
            ],
            open_symbol_search: vec![KeyBinding::key(KeyCode::Char('f'))],
            toggle_theme_preview: vec![KeyBinding::key(KeyCode::Char('t'))],
            toggle_help: vec![KeyBinding::key(KeyCode::Char('?'))],
            cycle_pane: vec![KeyBinding::key(KeyCode::Tab)],
            copy_selection: vec![KeyBinding::ctrl(KeyCode::Char('c'))],
            select_all: vec![KeyBinding::ctrl(KeyCode::Char('a'))],
            toggle_wrap: vec![KeyBinding::key(KeyCode::Char('w'))],

            // Split pane controls (Alt-based)
            swap_split_orientation: vec![KeyBinding::alt(KeyCode::Char('s'))],
            close_active_pane: vec![KeyBinding::alt(KeyCode::Char('q'))],
            toggle_file_list: vec![
                KeyBinding::ctrl(KeyCode::Char('b')),
                KeyBinding::alt(KeyCode::Char('p')),
            ],
            split_up: vec![KeyBinding::alt(KeyCode::Up)],
            split_down: vec![KeyBinding::alt(KeyCode::Down)],
            split_left: vec![KeyBinding::alt(KeyCode::Left)],
            split_right: vec![KeyBinding::alt(KeyCode::Right)],
            resize_split_left: vec![KeyBinding::alt(KeyCode::Char('h'))],
            resize_split_right: vec![KeyBinding::alt(KeyCode::Char('l'))],

            // Search mode defaults
            search_close: vec![KeyBinding::key(KeyCode::Esc)],
            search_confirm: vec![KeyBinding::key(KeyCode::Enter)],
            search_backspace: vec![KeyBinding::key(KeyCode::Backspace)],
            search_navigate_up: vec![KeyBinding::key(KeyCode::Up)],
            search_navigate_down: vec![KeyBinding::key(KeyCode::Down)],

            // PDF navigation defaults (n/p for next/prev, g/G for first/last)
            pdf_next_page: vec![KeyBinding::key(KeyCode::Char('n'))],
            pdf_prev_page: vec![KeyBinding::key(KeyCode::Char('p'))],
            pdf_first_page: vec![KeyBinding::key(KeyCode::Home)],
            pdf_last_page: vec![KeyBinding::key(KeyCode::End)],

            // Popup defaults
            toggle_file_tree_popup: vec![KeyBinding::ctrl(KeyCode::Char('t'))],

            // Git mode default (Shift+G toggles history browsing)
            toggle_git_mode: vec![
                KeyBinding::new(KeyCode::Char('G'), KeyModifiers::SHIFT),
                KeyBinding::key(KeyCode::Char('G')),
            ],

            // Initialize empty maps
            normal_mode_map: HashMap::new(),
            search_mode_map: HashMap::new(),
        };
        bindings.rebuild_maps();
        bindings
    }
}

/// Helper to parse config keys and update binding if valid
fn apply_config_keys(target: &mut Vec<KeyBinding>, config_keys: Option<Vec<String>>) {
    if let Some(keys) = config_keys {
        let parsed: Vec<KeyBinding> = keys.iter().filter_map(|s| parse_key_string(s)).collect();
        if !parsed.is_empty() {
            *target = parsed;
        }
    }
}

/// Helper to insert bindings into a map
fn insert_bindings(map: &mut HashMap<KeyBinding, Action>, keys: &[KeyBinding], action: Action) {
    for key in keys {
        map.insert(key.clone(), action);
    }
}

impl KeyBindings {
    /// Create keybindings from config, using defaults for unspecified keys
    pub fn from_config(config: Option<KeyBindingsConfig>) -> Self {
        let mut bindings = Self::default();

        if let Some(cfg) = config {
            Self::apply_normal_mode_config(&mut bindings, &cfg);
            Self::apply_split_config(&mut bindings, &cfg);
            Self::apply_search_config(&mut bindings, &cfg);
            Self::apply_pdf_config(&mut bindings, &cfg);
            Self::apply_popup_config(&mut bindings, &cfg);
            apply_config_keys(&mut bindings.toggle_git_mode, cfg.toggle_git_mode.clone());
        }

        bindings.rebuild_maps();
        bindings
    }

    fn apply_normal_mode_config(bindings: &mut Self, cfg: &KeyBindingsConfig) {
        apply_config_keys(&mut bindings.quit, cfg.quit.clone());
        apply_config_keys(&mut bindings.navigate_up, cfg.navigate_up.clone());
        apply_config_keys(&mut bindings.navigate_down, cfg.navigate_down.clone());
        apply_config_keys(&mut bindings.enter, cfg.enter.clone());
        apply_config_keys(&mut bindings.go_back, cfg.go_back.clone());
        apply_config_keys(
            &mut bindings.scroll_preview_up,
            cfg.scroll_preview_up.clone(),
        );
        apply_config_keys(
            &mut bindings.scroll_preview_down,
            cfg.scroll_preview_down.clone(),
        );
        apply_config_keys(
            &mut bindings.scroll_preview_page_up,
            cfg.scroll_preview_page_up.clone(),
        );
        apply_config_keys(
            &mut bindings.scroll_preview_page_down,
            cfg.scroll_preview_page_down.clone(),
        );
        apply_config_keys(&mut bindings.shrink_file_list, cfg.shrink_file_list.clone());
        apply_config_keys(&mut bindings.grow_file_list, cfg.grow_file_list.clone());
        apply_config_keys(&mut bindings.open_search, cfg.open_search.clone());
        apply_config_keys(&mut bindings.open_find, cfg.open_find.clone());
        apply_config_keys(
            &mut bindings.open_symbol_search,
            cfg.open_symbol_search.clone(),
        );
        apply_config_keys(
            &mut bindings.toggle_theme_preview,
            cfg.toggle_theme_preview.clone(),
        );
        apply_config_keys(&mut bindings.toggle_help, cfg.toggle_help.clone());
        apply_config_keys(&mut bindings.cycle_pane, cfg.cycle_pane.clone());
        apply_config_keys(&mut bindings.copy_selection, cfg.copy_selection.clone());
        apply_config_keys(&mut bindings.select_all, cfg.select_all.clone());
        apply_config_keys(&mut bindings.toggle_wrap, cfg.toggle_wrap.clone());
    }

    fn apply_split_config(bindings: &mut Self, cfg: &KeyBindingsConfig) {
        apply_config_keys(
            &mut bindings.swap_split_orientation,
            cfg.swap_split_orientation.clone(),
        );
        apply_config_keys(
            &mut bindings.close_active_pane,
            cfg.close_active_pane.clone(),
        );
        apply_config_keys(&mut bindings.toggle_file_list, cfg.toggle_file_list.clone());
        apply_config_keys(&mut bindings.split_up, cfg.split_up.clone());
        apply_config_keys(&mut bindings.split_down, cfg.split_down.clone());
        apply_config_keys(&mut bindings.split_left, cfg.split_left.clone());
        apply_config_keys(&mut bindings.split_right, cfg.split_right.clone());
        apply_config_keys(
            &mut bindings.resize_split_left,
            cfg.resize_split_left.clone(),
        );
        apply_config_keys(
            &mut bindings.resize_split_right,
            cfg.resize_split_right.clone(),
        );
    }

    fn apply_search_config(bindings: &mut Self, cfg: &KeyBindingsConfig) {
        apply_config_keys(&mut bindings.search_close, cfg.search_close.clone());
        apply_config_keys(&mut bindings.search_confirm, cfg.search_confirm.clone());
        apply_config_keys(&mut bindings.search_backspace, cfg.search_backspace.clone());
        apply_config_keys(
            &mut bindings.search_navigate_up,
            cfg.search_navigate_up.clone(),
        );
        apply_config_keys(
            &mut bindings.search_navigate_down,
            cfg.search_navigate_down.clone(),
        );
    }

    fn apply_pdf_config(bindings: &mut Self, cfg: &KeyBindingsConfig) {
        apply_config_keys(&mut bindings.pdf_next_page, cfg.pdf_next_page.clone());
        apply_config_keys(&mut bindings.pdf_prev_page, cfg.pdf_prev_page.clone());
        apply_config_keys(&mut bindings.pdf_first_page, cfg.pdf_first_page.clone());
        apply_config_keys(&mut bindings.pdf_last_page, cfg.pdf_last_page.clone());
    }

    fn apply_popup_config(bindings: &mut Self, cfg: &KeyBindingsConfig) {
        apply_config_keys(
            &mut bindings.toggle_file_tree_popup,
            cfg.toggle_file_tree_popup.clone(),
        );
    }

    /// Rebuild the lookup maps after modifying bindings
    fn rebuild_maps(&mut self) {
        self.normal_mode_map.clear();
        self.search_mode_map.clear();

        self.build_normal_mode_map();
        self.build_split_map();
        self.build_search_mode_map();
        self.build_pdf_map();
        self.build_popup_map();
        insert_bindings(
            &mut self.normal_mode_map,
            &self.toggle_git_mode,
            Action::ToggleGitMode,
        );
    }

    fn build_normal_mode_map(&mut self) {
        insert_bindings(&mut self.normal_mode_map, &self.quit, Action::Quit);
        insert_bindings(
            &mut self.normal_mode_map,
            &self.navigate_up,
            Action::NavigateUp,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.navigate_down,
            Action::NavigateDown,
        );
        insert_bindings(&mut self.normal_mode_map, &self.enter, Action::Enter);
        insert_bindings(&mut self.normal_mode_map, &self.go_back, Action::GoBack);
        insert_bindings(
            &mut self.normal_mode_map,
            &self.scroll_preview_up,
            Action::ScrollPreviewUp,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.scroll_preview_down,
            Action::ScrollPreviewDown,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.scroll_preview_page_up,
            Action::ScrollPreviewPageUp,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.scroll_preview_page_down,
            Action::ScrollPreviewPageDown,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.shrink_file_list,
            Action::ShrinkFileList,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.grow_file_list,
            Action::GrowFileList,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.open_search,
            Action::OpenSearch,
        );
        insert_bindings(&mut self.normal_mode_map, &self.open_find, Action::OpenFind);
        insert_bindings(
            &mut self.normal_mode_map,
            &self.open_symbol_search,
            Action::OpenSymbolSearch,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.toggle_theme_preview,
            Action::ToggleThemePreview,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.toggle_help,
            Action::ToggleHelp,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.cycle_pane,
            Action::CyclePane,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.copy_selection,
            Action::CopySelection,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.select_all,
            Action::SelectAll,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.toggle_wrap,
            Action::ToggleWrap,
        );
    }

    fn build_split_map(&mut self) {
        insert_bindings(
            &mut self.normal_mode_map,
            &self.swap_split_orientation,
            Action::SwapSplitOrientation,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.close_active_pane,
            Action::CloseActivePane,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.toggle_file_list,
            Action::ToggleFileList,
        );
        insert_bindings(&mut self.normal_mode_map, &self.split_up, Action::SplitUp);
        insert_bindings(
            &mut self.normal_mode_map,
            &self.split_down,
            Action::SplitDown,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.split_left,
            Action::SplitLeft,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.split_right,
            Action::SplitRight,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.resize_split_left,
            Action::ResizeSplitLeft,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.resize_split_right,
            Action::ResizeSplitRight,
        );
    }

    fn build_search_mode_map(&mut self) {
        insert_bindings(
            &mut self.search_mode_map,
            &self.search_close,
            Action::SearchClose,
        );
        insert_bindings(
            &mut self.search_mode_map,
            &self.search_confirm,
            Action::SearchConfirm,
        );
        insert_bindings(
            &mut self.search_mode_map,
            &self.search_backspace,
            Action::SearchBackspace,
        );
        insert_bindings(
            &mut self.search_mode_map,
            &self.search_navigate_up,
            Action::SearchNavigateUp,
        );
        insert_bindings(
            &mut self.search_mode_map,
            &self.search_navigate_down,
            Action::SearchNavigateDown,
        );
        // Copy selection and Select all work in both modes
        insert_bindings(
            &mut self.search_mode_map,
            &self.copy_selection,
            Action::CopySelection,
        );
        insert_bindings(
            &mut self.search_mode_map,
            &self.select_all,
            Action::SelectAll,
        );
    }

    fn build_pdf_map(&mut self) {
        insert_bindings(
            &mut self.normal_mode_map,
            &self.pdf_next_page,
            Action::PdfNextPage,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.pdf_prev_page,
            Action::PdfPrevPage,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.pdf_first_page,
            Action::PdfFirstPage,
        );
        insert_bindings(
            &mut self.normal_mode_map,
            &self.pdf_last_page,
            Action::PdfLastPage,
        );
    }

    fn build_popup_map(&mut self) {
        insert_bindings(
            &mut self.normal_mode_map,
            &self.toggle_file_tree_popup,
            Action::ToggleFileTreePopup,
        );
    }

    /// Look up an action for the given key in normal mode
    pub fn lookup_normal(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        let key = KeyBinding::new(code, modifiers);
        self.normal_mode_map.get(&key).copied()
    }

    /// Look up an action for the given key in search mode
    pub fn lookup_search(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        let key = KeyBinding::new(code, modifiers);
        self.search_mode_map.get(&key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_keys() {
        assert_eq!(
            parse_key_string("q"),
            Some(KeyBinding::key(KeyCode::Char('q')))
        );
        assert_eq!(
            parse_key_string("enter"),
            Some(KeyBinding::key(KeyCode::Enter))
        );
        assert_eq!(parse_key_string("esc"), Some(KeyBinding::key(KeyCode::Esc)));
        assert_eq!(parse_key_string("up"), Some(KeyBinding::key(KeyCode::Up)));
        assert_eq!(
            parse_key_string("pagedown"),
            Some(KeyBinding::key(KeyCode::PageDown))
        );
    }

    #[test]
    fn test_parse_modifier_keys() {
        assert_eq!(
            parse_key_string("ctrl+c"),
            Some(KeyBinding::ctrl(KeyCode::Char('c')))
        );
        assert_eq!(
            parse_key_string("alt+s"),
            Some(KeyBinding::alt(KeyCode::Char('s')))
        );
        assert_eq!(
            parse_key_string("alt+up"),
            Some(KeyBinding::alt(KeyCode::Up))
        );
    }

    #[test]
    fn test_parse_shift_key() {
        // shift+h should become 'H'
        assert_eq!(
            parse_key_string("shift+h"),
            Some(KeyBinding::new(KeyCode::Char('H'), KeyModifiers::SHIFT))
        );
    }

    #[test]
    fn test_parse_special_chars() {
        assert_eq!(
            parse_key_string("/"),
            Some(KeyBinding::key(KeyCode::Char('/')))
        );
        assert_eq!(
            parse_key_string("?"),
            Some(KeyBinding::key(KeyCode::Char('?')))
        );
    }

    #[test]
    fn test_default_keybindings() {
        let bindings = KeyBindings::default();

        // Test quit bindings
        assert!(bindings
            .lookup_normal(KeyCode::Char('q'), KeyModifiers::NONE)
            .is_some());
        assert!(bindings
            .lookup_normal(KeyCode::Esc, KeyModifiers::NONE)
            .is_some());

        // Test navigation
        assert_eq!(
            bindings.lookup_normal(KeyCode::Up, KeyModifiers::NONE),
            Some(Action::NavigateUp)
        );
        assert_eq!(
            bindings.lookup_normal(KeyCode::Down, KeyModifiers::NONE),
            Some(Action::NavigateDown)
        );

        // Test scroll
        assert_eq!(
            bindings.lookup_normal(KeyCode::Char('j'), KeyModifiers::NONE),
            Some(Action::ScrollPreviewDown)
        );
        assert_eq!(
            bindings.lookup_normal(KeyCode::Char('k'), KeyModifiers::NONE),
            Some(Action::ScrollPreviewUp)
        );

        // Test selection
        assert_eq!(
            bindings.lookup_normal(KeyCode::Char('a'), KeyModifiers::CONTROL),
            Some(Action::SelectAll)
        );

        // Test Alt bindings
        assert_eq!(
            bindings.lookup_normal(KeyCode::Char('s'), KeyModifiers::ALT),
            Some(Action::SwapSplitOrientation)
        );

        // Test search mode
        assert_eq!(
            bindings.lookup_search(KeyCode::Esc, KeyModifiers::NONE),
            Some(Action::SearchClose)
        );
        assert_eq!(
            bindings.lookup_search(KeyCode::Enter, KeyModifiers::NONE),
            Some(Action::SearchConfirm)
        );
    }

    #[test]
    fn test_from_config_override() {
        let config = KeyBindingsConfig {
            quit: Some(vec!["ctrl+q".to_string()]),
            ..Default::default()
        };

        let bindings = KeyBindings::from_config(Some(config));

        // Old quit binding should NOT work
        assert!(bindings
            .lookup_normal(KeyCode::Char('q'), KeyModifiers::NONE)
            .is_none());
        // New quit binding should work
        assert_eq!(
            bindings.lookup_normal(KeyCode::Char('q'), KeyModifiers::CONTROL),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_keybinding_equality() {
        let binding = KeyBinding::ctrl(KeyCode::Char('c'));
        assert_eq!(
            binding,
            KeyBinding::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
        );
        assert_ne!(binding, KeyBinding::key(KeyCode::Char('c')));
        assert_ne!(binding, KeyBinding::ctrl(KeyCode::Char('x')));
    }
}
