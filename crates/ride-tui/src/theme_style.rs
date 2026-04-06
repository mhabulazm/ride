use ratatui::style::{Color, Modifier, Style};
use ride_core::theme::ColorStyle;

/// Parse a color string into a ratatui Color.
/// Supports named colors and #RRGGBB hex.
pub fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" | "dark_gray" | "dark_grey" => Color::DarkGray,
        "lightred" | "light_red" => Color::LightRed,
        "lightgreen" | "light_green" => Color::LightGreen,
        "lightyellow" | "light_yellow" => Color::LightYellow,
        "lightblue" | "light_blue" => Color::LightBlue,
        "lightmagenta" | "light_magenta" => Color::LightMagenta,
        "lightcyan" | "light_cyan" => Color::LightCyan,
        "white" => Color::White,
        hex if hex.starts_with('#') && hex.len() == 7 => {
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);
            Color::Rgb(r, g, b)
        }
        _ => Color::White,
    }
}

/// Convert a ColorStyle to a ratatui Style.
pub fn to_style(cs: &ColorStyle) -> Style {
    let mut style = Style::default();
    if let Some(ref fg) = cs.fg {
        style = style.fg(parse_color(fg));
    }
    if let Some(ref bg) = cs.bg {
        style = style.bg(parse_color(bg));
    }
    if cs.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cs.italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cs.underline {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}
