use super::App;

impl App {
    /// Toggle help mode
    pub fn toggle_help(&mut self) {
        self.help_mode = !self.help_mode;
    }

    /// Close help mode
    pub fn close_help(&mut self) {
        self.help_mode = false;
    }
}
