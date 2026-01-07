use super::App;

impl App {
    pub fn theme_search_navigate_up(&mut self) {
        let current_idx = self
            .available_themes
            .iter()
            .position(|t| t == &self.current_theme_name)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.available_themes.len() - 1
        } else {
            current_idx - 1
        };
        let theme_name = self.available_themes[prev_idx].clone();
        let _ = self.switch_theme(&theme_name);
    }

    pub fn theme_search_navigate_down(&mut self) {
        let current_idx = self
            .available_themes
            .iter()
            .position(|t| t == &self.current_theme_name)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.available_themes.len();
        let theme_name = self.available_themes[next_idx].clone();
        let _ = self.switch_theme(&theme_name);
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn theme_search_confirm(&mut self) {
        self.theme_preview_mode = false;
    }
}
