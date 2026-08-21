use super::Style;
use bevy::prelude::*;

pub struct TextColorStyle(Color);

impl Style for TextColorStyle {
	fn apply(&self, cmds: &mut EntityCommands) {
		cmds.insert(TextColor(self.0));
	}
}

pub struct BackgroundColorStyle(Color);

impl Style for BackgroundColorStyle {
	fn apply(&self, cmds: &mut EntityCommands) {
		cmds.insert(BackgroundColor(self.0));
	}
}

pub struct TextColorWithBackgroundStyle(Color, Color);

impl Style for TextColorWithBackgroundStyle {
	fn apply(&self, cmds: &mut EntityCommands) {
		cmds.insert((TextColor(self.0), BackgroundColor(self.1)));
	}
}

pub struct ApTextStyle(Color, FontWeight);

impl Style for ApTextStyle {
	fn apply(&self, cmds: &mut EntityCommands) {
		cmds.insert((TextColor(self.0), TextFont::from_font_weight(self.1)));
	}
}

pub struct BoldStyle;

impl Style for BoldStyle {
	fn apply(&self, cmds: &mut EntityCommands) {
		cmds.insert(TextFont::from_font_weight(FontWeight::BOLD));
	}
}

pub struct UnderlineStyle;

impl Style for UnderlineStyle {
	fn apply(&self, cmds: &mut EntityCommands) {
		cmds.insert(Underline);
	}
}

// generic text
pub const BOLD: BoldStyle = BoldStyle;
pub const UNDERLINE: UnderlineStyle = UnderlineStyle;
pub const PROGRESSION: ApTextStyle = ApTextStyle(Color::srgb_u8(168, 147, 228), FontWeight::BOLD);
pub const USEFUL: ApTextStyle = ApTextStyle(Color::srgb_u8(98, 122, 198), FontWeight::BOLD);
pub const TRAP: ApTextStyle = ApTextStyle(Color::srgb_u8(211, 113, 102), FontWeight::BOLD);
pub const LOCATION: ApTextStyle = ApTextStyle(Color::srgb_u8(0, 255, 127), FontWeight::NORMAL);
pub const ENTRANCE: ApTextStyle = ApTextStyle(Color::srgb(0.17, 0.01, 0.46), FontWeight::NORMAL);
pub const SELF: ApTextStyle = ApTextStyle(Color::srgb_u8(238, 0, 238), FontWeight::BOLD);
pub const PLAYER: ApTextStyle = ApTextStyle(Color::srgb_u8(150, 150, 110), FontWeight::BOLD);

// colors
pub const BLACK: TextColorStyle = TextColorStyle(Color::BLACK);
pub const BLACK_BG: TextColorWithBackgroundStyle =
	TextColorWithBackgroundStyle(Color::srgb_u8(0xcc, 0xcc, 0xcc), Color::BLACK);
pub const BLUE: TextColorStyle = TextColorStyle(Color::srgb_u8(63, 64, 255));
pub const BLUE_BG: TextColorWithBackgroundStyle = TextColorWithBackgroundStyle(
	Color::srgb_u8(0xcc, 0xcc, 0xcc),
	Color::srgb_u8(63, 64, 255),
);
pub const CYAN: TextColorWithBackgroundStyle = TextColorWithBackgroundStyle(
	Color::srgb_u8(127, 255, 255),
	Color::srgba(0.0, 0.0, 0.0, 0.5),
);
pub const CYAN_BG: BackgroundColorStyle = BackgroundColorStyle(Color::srgb_u8(127, 255, 255));
pub const GREEN: TextColorWithBackgroundStyle = TextColorWithBackgroundStyle(
	Color::srgb_u8(0, 255, 127),
	Color::srgba(0.0, 0.0, 0.0, 0.25),
);
pub const GREEN_BG: BackgroundColorStyle = BackgroundColorStyle(Color::srgb_u8(0, 255, 127));
pub const MAGENTA: TextColorStyle = TextColorStyle(Color::srgb_u8(255, 127, 255));
pub const MAGENTA_BG: BackgroundColorStyle = BackgroundColorStyle(Color::srgb_u8(255, 127, 255));
pub const RED: TextColorStyle = TextColorStyle(Color::srgb_u8(255, 64, 63));
pub const RED_BG: BackgroundColorStyle = BackgroundColorStyle(Color::srgb_u8(255, 64, 63));
pub const WHITE: TextColorWithBackgroundStyle =
	TextColorWithBackgroundStyle(Color::WHITE, Color::srgba(0.0, 0.0, 0.0, 0.5));
pub const WHITE_BG: BackgroundColorStyle = BackgroundColorStyle(Color::WHITE);
pub const YELLOW: TextColorWithBackgroundStyle = TextColorWithBackgroundStyle(
	Color::srgb_u8(255, 255, 127),
	Color::srgba(0.0, 0.0, 0.0, 0.5),
);
pub const YELLOW_BG: BackgroundColorStyle = BackgroundColorStyle(Color::srgb_u8(255, 255, 127));
