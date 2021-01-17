use super::*;
use std::char;
use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{color, cursor};

use unicode_width::UnicodeWidthStr;

pub struct View<'a> {
  state: &'a mut state::State<'a>,
  skip: usize,
  multi: bool,
  contrast: bool,
  position: &'a str,
  matches: Vec<state::Match<'a>>,
  select_foreground_color: Box<dyn color::Color>,
  select_background_color: Box<dyn color::Color>,
  foreground_color: Box<dyn color::Color>,
  background_color: Box<dyn color::Color>,
  hint_background_color: Box<dyn color::Color>,
  hint_foreground_color: Box<dyn color::Color>,
}

enum CaptureEvent {
  Exit,
  Hint(Vec<(String, bool)>),
}

impl<'a> View<'a> {
  pub fn new(
    state: &'a mut state::State<'a>,
    multi: bool,
    reverse: bool,
    unique: bool,
    contrast: bool,
    position: &'a str,
    select_foreground_color: Box<dyn color::Color>,
    select_background_color: Box<dyn color::Color>,
    foreground_color: Box<dyn color::Color>,
    background_color: Box<dyn color::Color>,
    hint_foreground_color: Box<dyn color::Color>,
    hint_background_color: Box<dyn color::Color>,
  ) -> View<'a> {
    let matches = state.matches(reverse, unique);
    let skip = if reverse { matches.len() - 1 } else { 0 };

    View {
      state,
      skip,
      multi,
      contrast,
      position,
      matches,
      select_foreground_color,
      select_background_color,
      foreground_color,
      background_color,
      hint_foreground_color,
      hint_background_color,
    }
  }

  pub fn prev(&mut self) {
    if self.skip > 0 {
      self.skip -= 1;
    }
  }

  pub fn next(&mut self) {
    if self.skip < self.matches.len() - 1 {
      self.skip += 1;
    }
  }

  fn make_hint_text(&self, hint: &str) -> String {
    if self.contrast {
      format!("[{}]", hint)
    } else {
      hint.to_string()
    }
  }

  fn render(&self, stdout: &mut dyn Write) -> () {
    write!(stdout, "{}", cursor::Hide).unwrap();

    for (index, line) in self.state.lines.iter().enumerate() {
      let clean = line.trim_end_matches(|c: char| c.is_whitespace());

      if !clean.is_empty() {
        print!("{goto}{text}", goto = cursor::Goto(1, index as u16 + 1), text = line);
      }
    }

    let selected = self.matches.get(self.skip);

    for mat in self.matches.iter() {
      let selected_color = if selected == Some(mat) {
        &self.select_foreground_color
      } else {
        &self.foreground_color
      };
      let selected_background_color = if selected == Some(mat) {
        &self.select_background_color
      } else {
        &self.background_color
      };

      // Find long utf sequences and extract it from mat.x
      let line = &self.state.lines[mat.y as usize];
      let prefix = &line[0..mat.x as usize];
      let extra = prefix.width_cjk() - prefix.chars().count();
      let offset = (mat.x as u16) - (extra as u16);
      let text = self.make_hint_text(mat.text);

      print!(
        "{goto}{background}{foregroud}{text}{resetf}{resetb}",
        goto = cursor::Goto(offset + 1, mat.y as u16 + 1),
        foregroud = color::Fg(&**selected_color),
        background = color::Bg(&**selected_background_color),
        resetf = color::Fg(color::Reset),
        resetb = color::Bg(color::Reset),
        text = &text
      );

      if let Some(ref hint) = mat.hint {
        let extra_position = match self.position {
          "right" => text.width_cjk() - hint.len(),
          "off_left" => 0 - hint.len() - if self.contrast { 2 } else { 0 },
          "off_right" => text.width_cjk(),
          _ => 0,
        };

        let text = self.make_hint_text(hint.as_str());
        let final_position = std::cmp::max(offset as i16 + extra_position as i16, 0);

        print!(
          "{goto}{background}{foregroud}{text}{resetf}{resetb}",
          goto = cursor::Goto(final_position as u16 + 1, mat.y as u16 + 1),
          foregroud = color::Fg(&*self.hint_foreground_color),
          background = color::Bg(&*self.hint_background_color),
          resetf = color::Fg(color::Reset),
          resetb = color::Bg(color::Reset),
          text = &text
        );
      }
    }

    stdout.flush().unwrap();
  }

  fn listen(&mut self, stdin: &mut dyn Read, stdout: &mut dyn Write) -> CaptureEvent {
    if self.matches.is_empty() {
      return CaptureEvent::Exit;
    }

    let mut chosen = vec![];
    let mut typed_hint: String = "".to_owned();
    let longest_hint = self
      .matches
      .iter()
      .filter_map(|m| m.hint.clone())
      .max_by(|x, y| x.len().cmp(&y.len()))
      .unwrap()
      .clone();

    self.render(stdout);

    loop {
      match stdin.keys().next() {
        Some(key) => {
          match key {
            Ok(key) => {
              match key {
                Key::Esc => {
                  if self.multi && !typed_hint.is_empty() {
                    typed_hint.clear();
                  } else {
                    break;
                  }
                }
                Key::Up => {
                  self.prev();
                }
                Key::Down => {
                  self.next();
                }
                Key::Left => {
                  self.prev();
                }
                Key::Right => {
                  self.next();
                }
                Key::Char(ch) => {
                  if ch == '\n' {
                    match self.matches.iter().enumerate().find(|&h| h.0 == self.skip) {
                      Some(hm) => {
                        chosen.push((hm.1.text.to_string(), false));

                        if !self.multi {
                          return CaptureEvent::Hint(chosen);
                        }
                      }
                      _ => panic!("Match not found?"),
                    }
                  }

                  if ch == ' ' && self.multi {
                    return CaptureEvent::Hint(chosen);
                  }

                  let key = ch.to_string();
                  let lower_key = key.to_lowercase();

                  typed_hint.push_str(lower_key.as_str());

                  let selection = self.matches.iter().find(|mat| mat.hint == Some(typed_hint.clone()));

                  match selection {
                    Some(mat) => {
                      chosen.push((mat.text.to_string(), key != lower_key));

                      if self.multi {
                        typed_hint.clear();
                      } else {
                        return CaptureEvent::Hint(chosen);
                      }
                    }
                    None => {
                      if !self.multi && typed_hint.len() >= longest_hint.len() {
                        break;
                      }
                    }
                  }
                }
                _ => {
                  // Unknown key
                }
              }
            }
            Err(err) => panic!(err),
          }

          stdin.keys().for_each(|_| { /* Skip the rest of stdin buffer */ })
        }
        _ => {
          // Nothing in the buffer. Wait for a bit...
          std::thread::sleep(std::time::Duration::from_millis(50));
          continue; // don't render again if nothing new to show
        }
      }

      self.render(stdout);
    }

    CaptureEvent::Exit
  }

  pub fn present(&mut self) -> Vec<(String, bool)> {
    let mut stdin = async_stdin();
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());

    let hints = match self.listen(&mut stdin, &mut stdout) {
      CaptureEvent::Exit => vec![],
      CaptureEvent::Hint(chosen) => chosen,
    };

    write!(stdout, "{}", cursor::Show).unwrap();

    hints
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn split(output: &str) -> Vec<&str> {
    output.split("\n").collect::<Vec<&str>>()
  }

  #[test]
  fn hint_text() {
    let lines = split("lorem 127.0.0.1 lorem");
    let custom = [].to_vec();
    let mut state = state::State::new(&lines, "abcd", &custom);
    let mut view = View {
      state: &mut state,
      skip: 0,
      multi: false,
      contrast: false,
      position: &"",
      matches: vec![],
      select_foreground_color: colors::get_color("default"),
      select_background_color: colors::get_color("default"),
      foreground_color: colors::get_color("default"),
      background_color: colors::get_color("default"),
      hint_background_color: colors::get_color("default"),
      hint_foreground_color: colors::get_color("default"),
    };

    let result = view.make_hint_text("a");
    assert_eq!(result, "a".to_string());

    view.contrast = true;
    let result = view.make_hint_text("a");
    assert_eq!(result, "[a]".to_string());
  }
}
