use super::*;
use std::char;
use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{color, cursor, terminal_size};

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
  multi_foreground_color: Box<dyn color::Color>,
  multi_background_color: Box<dyn color::Color>,
  foreground_color: Box<dyn color::Color>,
  background_color: Box<dyn color::Color>,
  hint_background_color: Box<dyn color::Color>,
  hint_foreground_color: Box<dyn color::Color>,
  chosen: Vec<(String, bool)>,
}

enum CaptureEvent {
  Exit,
  Hint,
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
    multi_foreground_color: Box<dyn color::Color>,
    multi_background_color: Box<dyn color::Color>,
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
      multi_foreground_color,
      multi_background_color,
      foreground_color,
      background_color,
      hint_foreground_color,
      hint_background_color,
      chosen: vec![],
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

  fn render(&self, stdout: &mut dyn Write, typed_hint: &str) -> () {
    write!(stdout, "{}", cursor::Hide).unwrap();

    let mut printed_row_sum = 0;
    let column_size = terminal_size().unwrap().0;

    for (index, line) in self.state.lines.iter().enumerate() {
      let clean = line.trim_end_matches(|c: char| c.is_whitespace());
      if clean.is_empty() {
        printed_row_sum += 1;
        continue;
      }

      let mut output_line = (*line).to_string();
      let selected = self.matches.get(self.skip);
      let cur_line_matches = self.matches.iter().filter(|m| m.y == index as i32);

      // process in reverse order so mat.x stays valid
      for mat in cur_line_matches.rev() {
        // split line into prefix, match, suffix
        let mut prefix = &output_line[..mat.x as usize];
        let mut m = &output_line[mat.x as usize..mat.x as usize + mat.text.width()];
        let mut suffix = &output_line[mat.x as usize + mat.text.width()..];

        /*
         *  off_left: <prefix[..hint.len] + hint> <hint> <text> <suffix>
         *      left: <prefix> <hint + text[hint.len..]> <suffix>
         *     right: <prefix> <text[..hint.len] + hint> <suffix>
         * off_right: <prefix> <text> <hint + suffix[hint.len..]>
         * respect the unicode while slicing
         */
        if let Some(ref hint) = mat.hint {
          if self.position == "off_left" {
            prefix = if prefix.width() < hint.width() {
              &prefix[..0]
            } else {
              let max_idx = prefix.char_indices().map(|(x, _)| x).max().unwrap_or(0);
              let max_lower_than_hint = prefix
                .char_indices()
                .filter(|&(x, _)| x < max_idx - hint.width())
                .map(|(x, _)| x)
                .max()
                .unwrap_or(0);
              &prefix[..max_lower_than_hint]
            };
          } else if self.position == "left" {
            m = &m[hint.width()..]
          } else if self.position == "right" {
            m = &m[..m.width() - hint.width()]
          } else {
            suffix = if suffix.width() < hint.width() {
              &suffix[..0]
            } else {
              let min_idx = suffix.char_indices().map(|(x, _)| x).min().unwrap_or(0);
              let max_lower_than_hint = suffix
                .char_indices()
                .filter(|&(x, _)| x > min_idx + hint.width())
                .map(|(x, _)| x)
                .max()
                .unwrap_or(0);
              &suffix[max_lower_than_hint..]
            }
          }
        }

        let chosen_hint = self.chosen.iter().any(|(hint, _)| hint == mat.text);
        let selected_color = if chosen_hint {
          &self.multi_foreground_color
        } else if selected == Some(mat) {
          &self.select_foreground_color
        } else {
          &self.foreground_color
        };
        let selected_background_color = if chosen_hint {
          &self.multi_background_color
        } else if selected == Some(mat) {
          &self.select_background_color
        } else {
          &self.background_color
        };

        let match_bg_color = color::Bg(&**selected_background_color).to_string();
        let match_bg_color = match_bg_color.as_str();
        let match_fg_color = color::Fg(&**selected_color).to_string();
        let match_fg_color = match_fg_color.as_str();
        let bg_reset = color::Bg(color::Reset).to_string();
        let bg_reset = bg_reset.as_str();
        let fg_reset = color::Fg(color::Reset).to_string();
        let fg_reset = fg_reset.as_str();

        output_line = if let Some(ref hint) = mat.hint {
          let (hint_bg_color, hint_fg_color) = if hint.starts_with(typed_hint) {
            let bg = color::Bg(&*self.multi_background_color).to_string();
            let fg = color::Fg(&*self.multi_foreground_color).to_string();
            (bg, fg)
          } else {
            let bg = color::Bg(&*self.hint_background_color).to_string();
            let fg = color::Fg(&*self.hint_foreground_color).to_string();
            (bg, fg)
          };

          if self.position == "left" || self.position == "off_left" {
            String::from("")
              + prefix
              + hint_bg_color.as_str()
              + hint_fg_color.as_str()
              + hint
              + fg_reset
              + bg_reset
              + match_bg_color
              + match_fg_color
              + m
              + fg_reset
              + bg_reset
              + suffix
          } else {
            String::from("")
              + prefix
              + match_bg_color
              + match_fg_color
              + m
              + fg_reset
              + bg_reset
              + hint_bg_color.as_str()
              + hint_fg_color.as_str()
              + hint
              + fg_reset
              + bg_reset
              + suffix
          }
        } else {
          String::from("") + prefix + match_bg_color + match_fg_color + m + fg_reset + bg_reset + suffix
        };
      }

      print!(
        "{goto}{text}",
        goto = cursor::Goto(1, printed_row_sum as u16 + 1),
        text = output_line
      );
      stdout.flush().unwrap();
      let printed_rows = line.width() / (column_size + 1) as usize + 1;
      printed_row_sum += printed_rows;
    }

    stdout.flush().unwrap();
  }

  fn listen(&mut self, stdin: &mut dyn Read, stdout: &mut dyn Write) -> CaptureEvent {
    if self.matches.is_empty() {
      return CaptureEvent::Exit;
    }

    let mut typed_hint: String = "".to_owned();
    let longest_hint = self
      .matches
      .iter()
      .filter_map(|m| m.hint.clone())
      .max_by(|x, y| x.len().cmp(&y.len()))
      .unwrap()
      .clone();

    self.render(stdout, &typed_hint);

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
                Key::Backspace => {
                  typed_hint.pop();
                }
                Key::Char(ch) => {
                  match ch {
                    '\n' => match self.matches.iter().enumerate().find(|&h| h.0 == self.skip) {
                      Some(hm) => {
                        self.chosen.push((hm.1.text.to_string(), false));

                        if !self.multi {
                          return CaptureEvent::Hint;
                        }
                      }
                      _ => panic!("Match not found?"),
                    },
                    ' ' => {
                      if self.multi {
                        // Finalize the multi selection
                        return CaptureEvent::Hint;
                      } else {
                        // Enable the multi selection
                        self.multi = true;
                      }
                    }
                    key => {
                      let key = key.to_string();
                      let lower_key = key.to_lowercase();

                      typed_hint.push_str(lower_key.as_str());

                      let selection = self.matches.iter().find(|mat| mat.hint == Some(typed_hint.clone()));

                      match selection {
                        Some(mat) => {
                          self.chosen.push((mat.text.to_string(), key != lower_key));

                          if self.multi {
                            typed_hint.clear();
                          } else {
                            return CaptureEvent::Hint;
                          }
                        }
                        None => {
                          if !self.multi && typed_hint.len() >= longest_hint.len() {
                            break;
                          }
                        }
                      }
                    }
                  }
                }
                _ => {
                  // Unknown key
                }
              }
            }
            Err(err) => panic!("{}", err),
          }

          stdin.keys().for_each(|_| { /* Skip the rest of stdin buffer */ })
        }
        _ => {
          // Nothing in the buffer. Wait for a bit...
          std::thread::sleep(std::time::Duration::from_millis(50));
          continue; // don't render again if nothing new to show
        }
      }

      self.render(stdout, &typed_hint);
    }

    CaptureEvent::Exit
  }

  pub fn present(&mut self) -> Vec<(String, bool)> {
    let mut stdin = async_stdin();
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());

    let hints = match self.listen(&mut stdin, &mut stdout) {
      CaptureEvent::Exit => vec![],
      CaptureEvent::Hint => self.chosen.clone(),
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
      multi_foreground_color: colors::get_color("default"),
      multi_background_color: colors::get_color("default"),
      foreground_color: colors::get_color("default"),
      background_color: colors::get_color("default"),
      hint_background_color: colors::get_color("default"),
      hint_foreground_color: colors::get_color("default"),
      chosen: vec![],
    };

    let result = view.make_hint_text("a");
    assert_eq!(result, "a".to_string());

    view.contrast = true;
    let result = view.make_hint_text("a");
    assert_eq!(result, "[a]".to_string());
  }
}
