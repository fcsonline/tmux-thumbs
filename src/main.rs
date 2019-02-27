extern crate rustbox;
extern crate clap;

mod state;
mod alphabets;
mod colors;

use self::clap::{Arg, App};
use std::char;
use std::default::Default;
use std::process::Command;
use clap::crate_version;
use rustbox::{Color, RustBox, OutputMode};
use rustbox::Key;

fn exec_command(command: String) -> std::process::Output {
  let args: Vec<_> = command.split(" ").collect();

  return Command::new(args[0])
    .args(&args[1..])
    .output()
    .expect("Couldn't run it");
}

fn app_args<'a> () -> clap::ArgMatches<'a> {
  return App::new("tmux-thumbs")
    .version(crate_version!())
    .about("A lightning fast version of tmux-fingers, copy/pasting tmux like vimium/vimperator")
    .arg(Arg::with_name("alphabet")
                .help("Sets the alphabet")
                .long("alphabet")
                .short("a")
                .default_value("qwerty"))
    .arg(Arg::with_name("foreground_color")
                .help("Sets the foregroud color for matches")
                .long("fg-color")
                .default_value("green"))
    .arg(Arg::with_name("background_color")
                .help("Sets the background color for matches")
                .long("bg-color")
                .default_value("black"))
    .arg(Arg::with_name("hint_foreground_color")
                .help("Sets the foregroud color for hints")
                .long("hint-fg-color")
                .default_value("yellow"))
    .arg(Arg::with_name("hint_background_color")
                .help("Sets the background color for hints")
                .long("hint-bg-color")
                .default_value("black"))
    .arg(Arg::with_name("select_foreground_color")
                .help("Sets the foregroud color for selection")
                .long("select-fg-color")
                .default_value("blue"))
    .arg(Arg::with_name("reverse")
                .help("Reverse the order for assigned hints")
                .long("reverse")
                .short("r"))
    .arg(Arg::with_name("unique")
                .help("Don't show duplicated hints for the same match")
                .long("unique")
                .short("u"))
    .arg(Arg::with_name("position")
                .help("Hint position")
                .long("position")
                .default_value("left")
                .short("p"))
    .get_matches();
}

fn main() {
  let args = app_args();
  let alphabet = args.value_of("alphabet").unwrap();
  let position = args.value_of("position").unwrap();
  let reverse = args.is_present("reverse");
  let unique = args.is_present("unique");

  let foreground_color = colors::get_color(args.value_of("foreground_color").unwrap());
  let background_color = colors::get_color(args.value_of("background_color").unwrap());
  let hint_foreground_color = colors::get_color(args.value_of("hint_foreground_color").unwrap());
  let hint_background_color = colors::get_color(args.value_of("hint_background_color").unwrap());
  let select_foreground_color = colors::get_color(args.value_of("select_foreground_color").unwrap());

  let execution = exec_command(format!("tmux capture-pane -e -J -p"));
  let output = String::from_utf8_lossy(&execution.stdout);
  let lines = output.split("\n").collect::<Vec<&str>>();

  let mut state = state::State::new(lines, alphabet);
  let mut paste = false;

  {
    let mut rustbox = match RustBox::init(Default::default()) {
      Result::Ok(v) => v,
      Result::Err(e) => panic!("{}", e),
    };

    rustbox.set_output_mode(OutputMode::EightBit);

    for (index, line) in state.lines.iter().enumerate() {
      let clean = line.trim_right_matches(|c: char| c.is_whitespace());

      if clean.len() > 0 {
        let formatted = format!("{}\n", line).to_string();
        rustbox.print(0, index, rustbox::RB_NORMAL, Color::White, Color::Black, formatted.as_str());
      }
    }

    let mut typed_hint: String = "".to_owned();
    let matches = state.matches(reverse, unique);
    let longest_hint = matches.iter().filter(|&m| m.hint.clone().is_some()).last().unwrap().hint.clone().expect("Unknown hint").len();

    loop {
      let mut selected = matches.last();

      match matches.iter().enumerate().find(|&h| h.0 == state.skip) {
        Some(hm) => {
          selected = Some(hm.1);
        }
        _ => {}
      }

      for mat in matches.iter() {
        let selected_color = if selected == Some(mat) {
          select_foreground_color
        } else {
          foreground_color
        };

        // TODO: Find long utf sequences and extract it from mat.x
        // let re = regex::bytes::Regex::new(r"127").unwrap();
        // let line = lines[mat.y as usize];
        // let extra = re
        //   .find_iter(line.as_bytes())
        //   .fold(0, |sum, item| sum + item.as_bytes().len());

        let extra = 0;

        let offset = (mat.x as usize) - extra;

        rustbox.print(offset, mat.y as usize, rustbox::RB_NORMAL, selected_color, background_color, mat.text);

        if let Some(ref hint) = mat.hint {
          let extra_position = if position == "left" { 0 } else { mat.text.len() - mat.hint.clone().unwrap().len() };

          rustbox.print(offset + extra_position, mat.y as usize, rustbox::RB_BOLD, hint_foreground_color, hint_background_color, hint.as_str());
        }
      }

      rustbox.present();

      match rustbox.poll_event(false) {
        Ok(rustbox::Event::KeyEvent(key)) => {
          match key {
            Key::Esc => { break; }
            Key::Enter => {
              let mut choosen = matches.first().unwrap();

              match matches.iter().enumerate().find(|&h| h.0 == state.skip) {
                Some(hm) => {
                  choosen = hm.1;
                }
                _ => {}
              }

              exec_command(format!("tmux set-buffer {}", choosen.text));

              break;
            }
            Key::Up => { state.prev(); }
            Key::Down => { state.next(); }
            Key::Left => { state.prev(); }
            Key::Right => { state.next(); }
            Key::Char(ch) => {
              let key = ch.to_string();
              let lower_key = key.to_lowercase();
              typed_hint.push_str(lower_key.as_str());
              match matches.iter().find(|mat| mat.hint == Some(typed_hint.clone())) {
                Some(mat) => {
                  exec_command(format!("tmux set-buffer {}", mat.text));

                  if key != lower_key {
                    paste = true;
                  }

                  break;
                },
                None => {
                  if typed_hint.len() > longest_hint {
                    break;
                  }
                }
              }
            }
            _ => {}
          }
        }
        Err(e) => panic!("{}", e),
        _ => { }
      }
    }
  }

  if paste {
    exec_command(format!("tmux paste-buffer"));
  }
}
