#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavMode {
    Normal,
    Insert,
    Command,
    Visual,
    Help,
}

impl NavMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Command => "COMMAND",
            Self::Visual => "VISUAL",
            Self::Help => "HELP",
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Normal => "◆ NORMAL",
            Self::Insert => "✎ INSERT",
            Self::Command => "▶ COMMAND",
            Self::Visual => "▣ VISUAL",
            Self::Help => "? HELP",
        }
    }
}

impl Default for NavMode {
    fn default() -> Self {
        Self::Insert
    }
}
