use super::App;

impl App {
    /// Toggle help mode
    #[allow(clippy::missing_const_for_fn)]
    pub fn toggle_help(&mut self) {
        self.help_mode = !self.help_mode;
    }

    /// Close help mode
    #[allow(clippy::missing_const_for_fn)]
    pub fn close_help(&mut self) {
        self.help_mode = false;
    }
}
