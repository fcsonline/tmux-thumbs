use rustbox::Color;
use std::collections::HashMap;

const COLORS: [(&'static str, Color); 9] = [
  ("black", Color::Black),
  ("red", Color::Red),
  ("green", Color::Green),
  ("yellow", Color::Yellow),
  ("blue", Color::Blue),
  ("magenta", Color::Magenta),
  ("cyan", Color::Cyan),
  ("white", Color::White),
  ("default", Color::Default),
];

pub fn get_color(color_name: &str) -> Color {
  let available_colors: HashMap<&str, Color> = COLORS.iter().cloned().collect();

  available_colors[&color_name]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn match_color () {
    assert_eq!(get_color("green"), Color::Green);
  }
}
