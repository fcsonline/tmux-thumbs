extern crate clap;
extern crate rustbox;

mod alphabets;
mod colors;
mod state;
mod view;

use self::clap::{App, Arg};
use clap::crate_version;
use std::io::{self, Read};

fn app_args<'a>() -> clap::ArgMatches<'a> {
  return App::new("thumbs")
    .version(crate_version!())
    .about("A lightning fast version copy/pasting like vimium/vimperator")
    .arg(
      Arg::with_name("alphabet")
        .help("Sets the alphabet")
        .long("alphabet")
        .short("a")
        .default_value("qwerty"),
    )
    .arg(
      Arg::with_name("format")
        .help("Specifies the out format for the picked hint. (%U: Upcase, %H: Hint)")
        .long("format")
        .short("f")
        .default_value("%H"),
    )
    .arg(
      Arg::with_name("foreground_color")
        .help("Sets the foregroud color for matches")
        .long("fg-color")
        .default_value("green"),
    )
    .arg(
      Arg::with_name("background_color")
        .help("Sets the background color for matches")
        .long("bg-color")
        .default_value("black"),
    )
    .arg(
      Arg::with_name("hint_foreground_color")
        .help("Sets the foregroud color for hints")
        .long("hint-fg-color")
        .default_value("yellow"),
    )
    .arg(
      Arg::with_name("hint_background_color")
        .help("Sets the background color for hints")
        .long("hint-bg-color")
        .default_value("black"),
    )
    .arg(
      Arg::with_name("select_foreground_color")
        .help("Sets the foreground color for selection")
        .long("select-fg-color")
        .default_value("blue"),
    )
    .arg(
      Arg::with_name("select_background_color")
        .help("Sets the background color for selection")
        .long("select-bg-color")
        .default_value("black"),
    )
    .arg(
      Arg::with_name("multi")
        .help("Enable multi-selection")
        .long("multi")
        .short("m"),
    )
    .arg(
      Arg::with_name("reverse")
        .help("Reverse the order for assigned hints")
        .long("reverse")
        .short("r"),
    )
    .arg(
      Arg::with_name("unique")
        .help("Don't show duplicated hints for the same match")
        .long("unique")
        .short("u"),
    )
    .arg(
      Arg::with_name("position")
        .help("Hint position")
        .long("position")
        .default_value("left")
        .short("p"),
    )
    .arg(
      Arg::with_name("regexp")
        .help("Use this regexp as extra pattern to match")
        .long("regexp")
        .short("x")
        .takes_value(true)
        .multiple(true),
    )
    .arg(
      Arg::with_name("contrast")
        .help("Put square brackets around hint for visibility")
        .long("contrast")
        .short("c"),
    )
    .get_matches();
}

fn main() {
  let args = app_args();
  let format = args.value_of("format").unwrap();
  let alphabet = args.value_of("alphabet").unwrap();
  let position = args.value_of("position").unwrap();
  let multi = args.is_present("multi");
  let reverse = args.is_present("reverse");
  let unique = args.is_present("unique");
  let contrast = args.is_present("contrast");
  let regexp = if let Some(items) = args.values_of("regexp") {
    items.collect::<Vec<_>>()
  } else {
    [].to_vec()
  };

  let foreground_color = colors::get_color(args.value_of("foreground_color").unwrap());
  let background_color = colors::get_color(args.value_of("background_color").unwrap());
  let hint_foreground_color = colors::get_color(args.value_of("hint_foreground_color").unwrap());
  let hint_background_color = colors::get_color(args.value_of("hint_background_color").unwrap());
  let select_foreground_color =
    colors::get_color(args.value_of("select_foreground_color").unwrap());
  let select_background_color =
    colors::get_color(args.value_of("select_background_color").unwrap());

  let stdin = io::stdin();
  let mut handle = stdin.lock();
  let mut output = String::new();

  handle.read_to_string(&mut output).unwrap();

  let lines = output.split("\n").collect::<Vec<&str>>();

  let mut state = state::State::new(&lines, alphabet, &regexp);

  let selected = {
    let mut viewbox = view::View::new(
      &mut state,
      multi,
      reverse,
      unique,
      contrast,
      position,
      select_foreground_color,
      select_background_color,
      foreground_color,
      background_color,
      hint_foreground_color,
      hint_background_color,
    );

    viewbox.present()
  };

  if !selected.is_empty() {
    for (text, upcase) in selected.iter() {
      let mut output = format.to_string();

      let upcase_value = if *upcase { "true" } else { "false" };

      output = str::replace(&output, "%U", upcase_value);
      output = str::replace(&output, "%H", text.as_str());

      if multi {
        println!("{}", output);
      } else {
        print!("{}", output);
      }
    }
  } else {
    ::std::process::exit(1);
  }
}
