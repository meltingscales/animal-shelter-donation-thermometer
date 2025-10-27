/// Color constants for thermometer rendering

// Light mode colors
pub mod light {
    // Background
    pub const BACKGROUND: &str = "white";

    // Title and text
    pub const TITLE_TEXT: &str = "#4A4A4A";
    pub const TEXT_PRIMARY: &str = "#4A4A4A";
    pub const TEXT_SECONDARY: &str = "#888888";

    // Thermometer structure
    pub const TUBE_FILL: &str = "white";
    pub const TUBE_STROKE: &str = "#6B6B6B";

    // Progress fill - Christmas red gradient
    pub const FILL_COLOR_1: &str = "#DC143C"; // Crimson
    pub const FILL_COLOR_2: &str = "#FF6B6B"; // Light red

    // Achieved amount text - Christmas red
    pub const ACHIEVED_TEXT: &str = "#DC143C";

    // Percentage markers
    pub const MARKER_STROKE: &str = "#888";
    pub const MARKER_TEXT: &str = "#888";
}

// Dark mode colors
pub mod dark {
    // Background
    pub const BACKGROUND: &str = "#1a1a1a";

    // Title and text
    pub const TITLE_TEXT: &str = "#E0E0E0";
    pub const TEXT_PRIMARY: &str = "#E0E0E0";
    pub const TEXT_SECONDARY: &str = "#AAAAAA";

    // Thermometer structure
    pub const TUBE_FILL: &str = "#2a2a2a";
    pub const TUBE_STROKE: &str = "#9B9B9B";

    // Progress fill - Brighter Christmas red for dark mode
    pub const FILL_COLOR_1: &str = "#FF4444"; // Bright red
    pub const FILL_COLOR_2: &str = "#FF7777"; // Light bright red

    // Achieved amount text - Bright Christmas red
    pub const ACHIEVED_TEXT: &str = "#FF6B6B";

    // Percentage markers
    pub const MARKER_STROKE: &str = "#AAAAAA";
    pub const MARKER_TEXT: &str = "#AAAAAA";
}
