extern crate rustbox;
extern crate clap;

mod state;
mod alphabets;
mod colors;
mod view;

use self::clap::{Arg, App};
use std::process::Command;
use clap::crate_version;

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
    .arg(Arg::with_name("tmux_pane")
                .help("Get this tmux pane as reference pane")
                .long("tmux-pane")
                .takes_value(true))
    .arg(Arg::with_name("command")
                .help("Pick command")
                .long("command")
                .default_value("tmux set-buffer {}"))
    .arg(Arg::with_name("upcase_command")
                .help("Upcase command")
                .long("upcase-command")
                .default_value("tmux paste-buffer"))
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

  let command = args.value_of("command").unwrap();
  let upcase_command = args.value_of("upcase_command").unwrap();
  let tmux_subcommand = if let Some(pane) = args.value_of("tmux_pane") {
    format!(" -t {}", pane)
  } else {
    "".to_string()
  };

  let execution = exec_command(format!("tmux capture-pane -e -J -p{}", tmux_subcommand));
  let output = String::from_utf8_lossy(&execution.stdout);
  let lines = output.split("\n").collect::<Vec<&str>>();

  let mut state = state::State::new(&lines, alphabet);

  let selected = {
    let mut viewbox = view::View::new(
      &mut state,
      reverse,
      unique,
      position,
      select_foreground_color,
      foreground_color,
      background_color,
      hint_foreground_color,
      hint_background_color
    );

    viewbox.present()
  };

  if let Some(pane) = args.value_of("tmux_pane") {
    exec_command(format!("tmux swap-pane -t {}", pane));
  };


  if let Some((text, paste)) = selected {
    exec_command(str::replace(command, "{}", text.as_str()));

    if paste {
      exec_command(upcase_command.to_string());
    }
  }

}
