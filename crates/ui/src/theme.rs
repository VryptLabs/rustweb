use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub struct Theme {
    pub radius: String,
    pub color_primary: String,
    pub color_surface: String,
    pub color_text: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            radius: "8px".into(),
            color_primary: "#3b82f6".into(),
            color_surface: "#ffffff".into(),
            color_text: "#0f172a".into(),
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "--rw-radius:{};--rw-primary:{};--rw-surface:{};--rw-text:{}",
            self.radius, self.color_primary, self.color_surface, self.color_text
        )
    }
}
