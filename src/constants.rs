use std::time::Duration;

use crate::funds::FundsAmount;

pub const STARTING_FUNDS: FundsAmount = 5000;

pub const AUTOSAVE_INTERVAL: Duration = Duration::from_mins(5);

pub mod files {
    pub const PROJECT_DIR_QUALIFIER: &str = "";
    pub const PROJECT_DIR_ORGANIZATION: &str = "amtep";
    pub const PROJECT_DIR_APPLICATION: &str = "Apocalyptosis";
}

pub mod ui {
    use bevy::color::Srgba;

    pub const TEXTURE_EARTH_BACKGROUND: &str = "textures/earth_night.jpg";

    pub const FONT_DISPLAY_PATH: &str = "fonts/DancingScript-Variable.ttf";
    pub const FONT_PATH: &str = "fonts/Lora-Variable.ttf";
    // A font spanning more unicode code points than usual
    pub const UNICODE_FONT_PATH: &str = "fonts/DejaVuSans.ttf";

    pub const THEME_DARK_PURPLE: Srgba = Srgba::rgb(0.102, 0.055, 0.243); // #1A0E3E
    pub const THEME_INDIGO: Srgba = Srgba::rgb(0.122, 0.102, 0.439); // #1F1A70
    #[expect(dead_code)]
    pub const THEME_MAGENTA: Srgba = Srgba::rgb(0.859, 0.282, 0.545); // #DB488B
    #[expect(dead_code)]
    pub const THEME_LIGHT_PINK: Srgba = Srgba::rgb(1.000, 0.514, 0.965); // #FF83F6
    #[expect(dead_code)]
    pub const THEME_CYAN: Srgba = Srgba::rgb(0.243, 0.816, 0.922); // #3ED0EB

    pub const WHITE: Srgba = Srgba::rgb(0.878, 0.878, 0.878);
    pub const DARK_GREY: Srgba = Srgba::rgb(0.149, 0.145, 0.153);
    pub const GREY: Srgba = Srgba::rgb(0.5, 0.5, 0.5);
    pub const BLACK: Srgba = Srgba::rgb(0.071, 0.071, 0.071);
    pub const YELLOW: Srgba = Srgba::rgb(1.00, 1.00, 0.384);
    pub const GREEN: Srgba = Srgba::rgb(0.694, 1.00, 0.384);
    pub const ORANGE: Srgba = Srgba::rgb(1.00, 0.694, 0.384);
    pub const RED: Srgba = Srgba::rgb(1.00, 0.384, 0.384);
    pub const BLUE: Srgba = Srgba::rgb(0.384, 0.384, 1.00);

    pub const BUTTON_BACKGROUND: Srgba = THEME_DARK_PURPLE;
    pub const BUTTON_HOVER_BACKGROUND: Srgba = THEME_INDIGO;
    pub const BUTTON_PRESSED_BACKGROUND: Srgba = BLACK;

    pub const DIALOG_BACKGROUND: Srgba = THEME_DARK_PURPLE;

    pub const MENU_BACKGROUND: Srgba = THEME_DARK_PURPLE;

    pub const TOOLTIP_BACKGROUND: Srgba = DARK_GREY;

    pub const TEXT: Srgba = WHITE;
    pub const BORDER: Srgba = WHITE;
    pub const TEXT_HIGHLIGHT: Srgba = YELLOW;
    pub const BORDER_HIGHLIGHT: Srgba = YELLOW;
    pub const TEXT_POSITIVE: Srgba = GREEN;
    pub const TEXT_MIXED: Srgba = ORANGE;
    pub const TEXT_NEGATIVE: Srgba = RED;
    pub const TEXT_NEUTRAL: Srgba = BLUE;
    pub const TEXT_DISABLED: Srgba = GREY;

    pub const HEADING: f32 = 24.0;
    pub const SUB_HEADING: f32 = 20.0;
    pub const LARGE: f32 = 16.0;
    pub const NORMAL: f32 = 13.0;
    pub const SMALL: f32 = 10.0;

    pub const ICON_PAUSE: &str = "textures/pause.png";
    pub const ICON_PLAY: &str = "textures/play.png";
    pub const ICON_FAST: &str = "textures/fast.png";
    pub const ICON_FASTEST: &str = "textures/fastest.png";

    pub const ZINDEX_MENU: i32 = 50;
    pub const ZINDEX_DIALOG: i32 = 100;
    pub const ZINDEX_TOOLTIP: i32 = 150;

    pub fn color(color: &str) -> Srgba {
        match color {
            "white" => WHITE,
            "dark grey" => DARK_GREY,
            "grey" => GREY,
            "black" => BLACK,
            "yellow" => YELLOW,
            "green" => GREEN,
            "orange" => ORANGE,
            "red" => RED,
            "blue" => BLUE,
            _ => unreachable!(),
        }
    }
}
