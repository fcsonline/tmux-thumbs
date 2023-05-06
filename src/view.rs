use super::*;
use std::io::{stdout, Read, Write};
use itertools::Itertools;
use termion::async_stdin;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{color, cursor};

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

  fn make_hint_text(&self, hint: &str) -> String {
    if self.contrast {
      format!("[{}]", hint)
    } else {
      hint.to_string()
    }
  }

  fn render(&self, stdout: &mut dyn Write, typed_hint: &str) {
    write!(stdout, "{}", cursor::Hide).unwrap();

    let mut output = self.state.output.to_string();
    let selected = self.matches.get(self.skip);
    let mut next_start = usize::MAX;

    for mat in self.matches.iter().sorted_by_key(|x| usize::MAX - x.start) {
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

      let matched_text = self.make_hint_text(mat.text);

      if let Some(ref hint) = mat.hint {
        let hint_text = self.make_hint_text(hint.as_str());
        let matched_text_len = matched_text.chars().count();
        let hint_text_len = hint_text.len();
        let (extra_position, extra_position_end, start, end) = match self.position {
          "right" => {
            let extra = if matched_text_len > hint_text_len {
              matched_text_len - hint_text_len
            } else {
              0
            };
            (
              extra,
              extra + hint_text_len,
              output[0..mat.start].to_string(),
              output[mat.end..].to_string(),
            )
          },
          "left" => (
            0,
            hint_text_len,
            output[0..mat.start].to_string(),
            output[mat.end..].to_string(),
          ),
          "off_right" => {
            let hint_text_contrast_len = hint_text_len +
              if self.contrast { 2 } else { 0 };
            if hint_text_contrast_len + mat.end > next_start {
              let extra = if matched_text_len > hint_text_len {
                matched_text_len - hint_text_len
              } else {
                0
              };
              (
                extra,
                extra + hint_text_len,
                output[0..mat.start].to_string(),
                output[mat.end..].to_string(),
              )
            } else {
              let mut tmp = output[mat.end..].chars();
              let mut newlines = false;
              for _ in 0..hint_text_contrast_len {
                match tmp.next() {
                  Some('\n') => {
                    newlines = true;
                    break;
                  },
                  Some('\r') => {
                    newlines = true;
                    break;
                  },
                  _ => (),
                };
              }
              if newlines {
                let extra = if matched_text_len > hint_text_len {
                  matched_text_len - hint_text_len
                } else {
                  0
                };
                (
                  extra,
                  extra + hint_text_len,
                  output[0..mat.start].to_string(),
                  output[mat.end..].to_string(),
                )
              } else {
                (
                  matched_text_len,
                  matched_text_len,
                  output[0..mat.start].to_string(),
                  tmp.collect::<String>(),
                )
              }
            }
          },
          "off_left" => {
            let hint_text_contrast_len = hint_text_len +
              if self.contrast { 2 } else { 0 };
            if hint_text_contrast_len + mat.prev_end > mat.start {
              (
                0,
                hint_text_len,
                output[0..mat.start].to_string(),
                output[mat.end..].to_string(),
              )
            } else {
              let mut tmp = output[0..mat.start].chars();
              let mut newlines = false;
              for _ in 0..hint_text_contrast_len {
                match tmp.next_back() {
                  Some('\n') => {
                    newlines = true;
                    break;
                  },
                  Some('\r') => {
                    newlines = true;
                    break;
                  },
                  _ => (),
                };
              }
              if newlines {
                (
                  0,
                  hint_text_len,
                  output[0..mat.start].to_string(),
                  output[mat.end..].to_string(),
                )
              } else {
                (
                  0,
                  0,
                  tmp.collect::<String>(),
                  output[mat.end..].to_string(),
                )
              }
            }
          },
          _ => {
            panic!("Unknown position: {}", self.position);
          }
        };

        if !typed_hint.is_empty() && hint.starts_with(typed_hint) {
          output = format!(
            "{start}{b}{f}{text_start}{typed_b}{typed_f}{typed}{hint_b}{hint_f}{hint}{b}{f}{text_end}{resetf}{resetb}{end}",
            start = &start,
            f = color::Fg(&**selected_color),
            b = color::Bg(&**selected_background_color),
            typed_f = color::Fg(&*self.multi_foreground_color),
            typed_b = color::Bg(&*self.multi_background_color),
            typed = &typed_hint,
            hint_f = color::Fg(&*self.hint_foreground_color),
            hint_b = color::Bg(&*self.hint_background_color),
            hint = &hint_text[typed_hint.len()..],
            resetf = color::Fg(color::Reset),
            resetb = color::Bg(color::Reset),
            text_start = &matched_text.chars().take(extra_position).collect::<String>(),
            text_end = &matched_text.chars().skip(extra_position_end).collect::<String>(),
            end = &end,
          );
        } else {
          output = format!(
            "{start}{b}{f}{text_start}{hint_b}{hint_f}{hint}{b}{f}{text_end}{resetf}{resetb}{end}",
            start = &start,
            f = color::Fg(&**selected_color),
            b = color::Bg(&**selected_background_color),
            hint_f = color::Fg(&*self.hint_foreground_color),
            hint_b = color::Bg(&*self.hint_background_color),
            hint = &hint_text,
            resetf = color::Fg(color::Reset),
            resetb = color::Bg(color::Reset),
            text_start = &matched_text.chars().take(extra_position).collect::<String>(),
            text_end = &matched_text.chars().skip(extra_position_end).collect::<String>(),
            end = &end,
          );
        };
      } else {
        output = format!(
          "{start}{b}{f}{text}{resetf}{resetb}{end}",
          start = &output[0..mat.start],
          f = color::Fg(&**selected_color),
          b = color::Bg(&**selected_background_color),
          resetf = color::Fg(color::Reset),
          resetb = color::Bg(color::Reset),
          text = &matched_text,
          end = &output[mat.end..],
        );
      }
      next_start = mat.start;
    }

    if output.ends_with("\r\n") {
      output.pop();
      output.pop();
    } else if output.ends_with('\n') {
      output.pop();
      output = output.replace('\n', "\r\n");
    };

    print!("\r\n{}", output);
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
      .unwrap();

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

  #[test]
  fn hint_text() {
    let output = "lorem 127.0.0.1 lorem";
    let custom = [].to_vec();
    let mut state = state::State::new(output, "abcd", &custom);
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
