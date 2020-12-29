use termion::color;

pub fn get_color(color_name: &str) -> Box<dyn color::Color> {
  match color_name {
    "black" => Box::new(color::Black),
    "red" => Box::new(color::Red),
    "green" => Box::new(color::Green),
    "yellow" => Box::new(color::Yellow),
    "blue" => Box::new(color::Blue),
    "magenta" => Box::new(color::Magenta),
    "cyan" => Box::new(color::Cyan),
    "white" => Box::new(color::White),
    "default" => Box::new(color::Reset),
    _ => panic!("Unknown color: {}", color_name),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn match_color() {
    let text1 = println!("{}{}", color::Fg(&*get_color("green")), "foo");
    let text2 = println!("{}{}", color::Fg(color::Green), "foo");

    assert_eq!(text1, text2);
  }

  #[test]
  #[should_panic]
  fn no_match_color() {
    println!("{}{}", color::Fg(&*get_color("wat")), "foo");
  }
}
