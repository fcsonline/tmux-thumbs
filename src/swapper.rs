extern crate clap;

mod view;

use self::clap::{App, Arg};
use clap::crate_version;
use regex::Regex;
use std::io::Write;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

trait Executor {
  fn execute(&mut self, args: Vec<String>) -> String;
  fn last_executed(&self) -> Option<Vec<String>>;
}

struct RealShell {
  executed: Option<Vec<String>>,
}

impl RealShell {
  fn new() -> RealShell {
    RealShell { executed: None }
  }
}

impl Executor for RealShell {
  fn execute(&mut self, args: Vec<String>) -> String {
    let execution = Command::new(args[0].as_str())
      .args(&args[1..])
      .output()
      .expect("Couldn't run it");

    self.executed = Some(args);

    let output: String = String::from_utf8_lossy(&execution.stdout).into();

    output.trim_end().to_string()
  }

  fn last_executed(&self) -> Option<Vec<String>> {
    self.executed.clone()
  }
}

const TMP_FILE: &str = "/tmp/thumbs-last";

#[allow(dead_code)]
fn dbg(msg: &str) {
  let mut file = std::fs::OpenOptions::new()
    .create(true)
    .write(true)
    .append(true)
    .open("/tmp/thumbs.log")
    .expect("Unable to open log file");

  writeln!(&mut file, "{}", msg).expect("Unable to write log file");
}

pub struct Swapper<'a> {
  executor: Box<&'a mut dyn Executor>,
  dir: String,
  main_action: String,
  shift_action: String,
  ctrl_action: String,
  alt_action: String,
  multi_action: String,
  osc52: bool,
  active_pane_id: Option<String>,
  active_pane_height: Option<i32>,
  active_pane_scroll_position: Option<i32>,
  active_pane_zoomed: Option<bool>,
  thumbs_pane_id: Option<String>,
  content: Option<String>,
  signal: String,
}

impl<'a> Swapper<'a> {
  fn new(
    executor: Box<&'a mut dyn Executor>,
    dir: String,
    main_action: String,
    shift_action: String,
    ctrl_action: String,
    alt_action: String,
    multi_action: String,
    osc52: bool,
  ) -> Swapper {
    let since_the_epoch = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards");
    let signal = format!("thumbs-finished-{}", since_the_epoch.as_secs());

    Swapper {
      executor,
      dir,
      main_action,
      shift_action,
      ctrl_action,
      alt_action,
      multi_action,
      osc52,
      active_pane_id: None,
      active_pane_height: None,
      active_pane_scroll_position: None,
      active_pane_zoomed: None,
      thumbs_pane_id: None,
      content: None,
      signal,
    }
  }

  pub fn capture_active_pane(&mut self) {
    let active_command = vec![
      "tmux",
      "list-panes",
      "-F",
      "#{pane_id}:#{?pane_in_mode,1,0}:#{pane_height}:#{scroll_position}:#{window_zoomed_flag}:#{?pane_active,active,nope}",
    ];

    let output = self
      .executor
      .execute(active_command.iter().map(|arg| arg.to_string()).collect());

    let lines: Vec<&str> = output.split('\n').collect();
    let chunks: Vec<Vec<&str>> = lines.into_iter().map(|line| line.split(':').collect()).collect();

    let active_pane = chunks
      .iter()
      .find(|&chunks| *chunks.get(5).unwrap() == "active")
      .expect("Unable to find active pane");

    let pane_id = active_pane.get(0).unwrap();

    self.active_pane_id = Some(pane_id.to_string());

    let pane_height = active_pane
      .get(2)
      .unwrap()
      .parse()
      .expect("Unable to retrieve pane height");

    self.active_pane_height = Some(pane_height);

    if active_pane.get(1).unwrap().to_string() == "1" {
      let pane_scroll_position = active_pane
        .get(3)
        .unwrap()
        .parse()
        .expect("Unable to retrieve pane scroll");

      self.active_pane_scroll_position = Some(pane_scroll_position);
    }

    let zoomed_pane = *active_pane.get(4).expect("Unable to retrieve zoom pane property") == "1";

    self.active_pane_zoomed = Some(zoomed_pane);
  }

  pub fn execute_thumbs(&mut self) {
    let options_command = vec!["tmux", "show", "-g"];
    let params: Vec<String> = options_command.iter().map(|arg| arg.to_string()).collect();
    let options = self.executor.execute(params);
    let lines: Vec<&str> = options.split('\n').collect();

    let pattern = Regex::new(r#"^@thumbs-([\w\-0-9]+)\s+"?([^"]+)"?$"#).unwrap();

    let args = lines
      .iter()
      .flat_map(|line| {
        if let Some(captures) = pattern.captures(line) {
          let name = captures.get(1).unwrap().as_str();
          let value = captures.get(2).unwrap().as_str();

          let boolean_params = vec!["reverse", "unique", "contrast"];

          if boolean_params.iter().any(|&x| x == name) {
            return vec![format!("--{}", name)];
          }

          let string_params = vec![
            "alphabet",
            "position",
            "fg-color",
            "bg-color",
            "hint-bg-color",
            "hint-fg-color",
            "select-fg-color",
            "select-bg-color",
            "multi-fg-color",
            "multi-bg-color",
          ];

          if string_params.iter().any(|&x| x == name) {
            return vec![format!("--{}", name), format!("'{}'", value)];
          }

          if name.starts_with("regexp") {
            return vec!["--regexp".to_string(), format!("'{}'", value.replace("\\\\", "\\"))];
          }

          vec![]
        } else {
          vec![]
        }
      })
      .collect::<Vec<String>>();

    let active_pane_id = self.active_pane_id.as_mut().unwrap().clone();

    let scroll_params =
      if let (Some(pane_height), Some(scroll_position)) = (self.active_pane_height, self.active_pane_scroll_position) {
        format!(" -S {} -E {}", -scroll_position, pane_height - scroll_position - 1)
      } else {
        "".to_string()
      };

    let active_pane_zoomed = self.active_pane_zoomed.as_mut().unwrap().clone();
    let zoom_command = if active_pane_zoomed {
      format!("tmux resize-pane -t {} -Z;", active_pane_id)
    } else {
      "".to_string()
    };

    let pane_command = format!(
        "tmux capture-pane -t {active_pane_id} -p{scroll_params} | tail -n {height} | {dir}/target/release/thumbs -f '%K:%H' -t {tmp} {args}; tmux swap-pane -t {active_pane_id}; {zoom_command} tmux wait-for -S {signal}",
        active_pane_id = active_pane_id,
        scroll_params = scroll_params,
        height = self.active_pane_height.unwrap_or(i32::MAX),
        dir = self.dir,
        tmp = TMP_FILE,
        args = args.join(" "),
        zoom_command = zoom_command,
        signal = self.signal
    );

    let thumbs_command = vec![
      "tmux",
      "new-window",
      "-P",
      "-F",
      "#{pane_id}",
      "-d",
      "-n",
      "[thumbs]",
      pane_command.as_str(),
    ];

    let params: Vec<String> = thumbs_command.iter().map(|arg| arg.to_string()).collect();

    self.thumbs_pane_id = Some(self.executor.execute(params));
  }

  pub fn swap_panes(&mut self) {
    let active_pane_id = self.active_pane_id.as_mut().unwrap().clone();
    let thumbs_pane_id = self.thumbs_pane_id.as_mut().unwrap().clone();

    let swap_command = vec![
      "tmux",
      "swap-pane",
      "-d",
      "-s",
      active_pane_id.as_str(),
      "-t",
      thumbs_pane_id.as_str(),
    ];

    let params = swap_command
      .iter()
      .filter(|&s| !s.is_empty())
      .map(|arg| arg.to_string())
      .collect();

    self.executor.execute(params);
  }

  pub fn resize_pane(&mut self) {
    let active_pane_zoomed = self.active_pane_zoomed.as_mut().unwrap().clone();

    if !active_pane_zoomed {
      return;
    }

    let thumbs_pane_id = self.thumbs_pane_id.as_mut().unwrap().clone();

    let resize_command = vec!["tmux", "resize-pane", "-t", thumbs_pane_id.as_str(), "-Z"];

    let params = resize_command
      .iter()
      .filter(|&s| !s.is_empty())
      .map(|arg| arg.to_string())
      .collect();

    self.executor.execute(params);
  }

  pub fn wait_thumbs(&mut self) {
    let wait_command = vec!["tmux", "wait-for", self.signal.as_str()];
    let params = wait_command.iter().map(|arg| arg.to_string()).collect();

    self.executor.execute(params);
  }

  pub fn retrieve_content(&mut self) {
    let retrieve_command = vec!["cat", TMP_FILE];
    let params = retrieve_command.iter().map(|arg| arg.to_string()).collect();

    self.content = Some(self.executor.execute(params));
  }

  pub fn destroy_content(&mut self) {
    let retrieve_command = vec!["rm", TMP_FILE];
    let params = retrieve_command.iter().map(|arg| arg.to_string()).collect();

    self.executor.execute(params);
  }

  pub fn send_osc52(&mut self) {}

  pub fn execute_command(&mut self) {
    let content = self.content.clone().unwrap();
    let items: Vec<&str> = content.split('\n').collect();

    if items.len() > 1 {
      let text = items
        .iter()
        .map(|item| item.splitn(2, ':').last().unwrap())
        .collect::<Vec<&str>>()
        .join(" ");

      // REVIEW: Not sure about the ModifierKeys here
      self.execute_final_command(&text, &view::ModifierKeys::Main.to_string(), &self.multi_action.clone());

      return;
    }

    // Only one item
    let item: &str = items.first().unwrap();

    let mut splitter = item.splitn(2, ':');

    if let Some(modkey) = splitter.next() {
      if let Some(text) = splitter.next() {
        if self.osc52 {
          let base64_text = base64::encode(text.as_bytes());
          let osc_seq = format!("\x1b]52;0;{}\x07", base64_text);
          let tmux_seq = format!("\x1bPtmux;{}\x1b\\", osc_seq.replace("\x1b", "\x1b\x1b"));

          // FIXME: Review if this comment is still rellevant
          //
          // When the user selects a match:
          // 1. The `rustbox` object created in the `viewbox` above is dropped.
          // 2. During its `drop`, the `rustbox` object sends a CSI 1049 escape
          //    sequence to tmux.
          // 3. This escape sequence causes the `window_pane_alternate_off` function
          //    in tmux to be called.
          // 4. In `window_pane_alternate_off`, tmux sets the needs-redraw flag in the
          //    pane.
          // 5. If we print the OSC copy escape sequence before the redraw is completed,
          //    tmux will *not* send the sequence to the host terminal. See the following
          //    call chain in tmux: `input_dcs_dispatch` -> `screen_write_rawstring`
          //    -> `tty_write` -> `tty_client_ready`. In this case, `tty_client_ready`
          //    will return false, thus preventing the escape sequence from being sent.
          //
          // Therefore, for now we wait a little bit here for the redraw to finish.
          std::thread::sleep(std::time::Duration::from_millis(100));

          std::io::stdout().write_all(tmux_seq.as_bytes()).unwrap();
          std::io::stdout().flush().unwrap();
        }

        let modkey = modkey.trim_end();

        let shift = view::ModifierKeys::Shift.to_string();
        let ctrl = view::ModifierKeys::Ctrl.to_string();
        let alt = view::ModifierKeys::Alt.to_string();

        let execute_command = match modkey {
          shift => self.shift_action.clone(),
          ctrl => self.ctrl_action.clone(),
          alt => self.alt_action.clone(),
          _ => self.main_action.clone(),
        };

        // The command we run has two arguments:
        //  * The first arg is the (trimmed) text. This gets stored in a variable, in order to
        //    preserve quoting and special characters.
        //
        //  * The second argument is the user's command, with the '{}' token replaced with an
        //    unquoted reference to the variable containing the text.
        //
        // The reference is unquoted, unfortunately, because the token may already have been
        // spliced into a string (e.g 'tmux display-message "Copied {}"'), and it's impossible (or
        // at least exceedingly difficult) to determine the correct quoting level.
        //
        // The alternative of literally splicing the text into the command is bad and it causes all
        // kinds of harmful escaping issues that the user cannot reasonable avoid.
        //
        // For example, imagine some pattern matched the text "foo;rm *" and the user's command was
        // an innocuous "echo {}". With literal splicing, we would run the command "echo foo;rm *".
        // That's BAD. Without splicing, instead we execute "echo ${THUMB}" which does mostly the
        // right thing regardless the contents of the text. (At worst, bash will word-separate the
        // unquoted variable; but it won't _execute_ those words in common scenarios).
        //
        // Ideally user commands would just use "${THUMB}" to begin with rather than having any
        // sort of ad-hoc string splicing here at all, and then they could specify the quoting they
        // want, but that would break backwards compatibility.
        self.execute_final_command(text.trim_end(), modkey, &execute_command);
      }
    }
  }

  pub fn execute_final_command(&mut self, text: &str, modkey: &str, execute_command: &str) {
    let final_command = str::replace(execute_command, "{}", "${THUMB}");
    let retrieve_command = vec![
      "bash",
      "-c",
      "THUMB=\"$1\"; MODIFIER=\"$2\"; eval \"$3\"",
      "--",
      text,
      modkey,
      final_command.as_str(),
    ];

    let params = retrieve_command.iter().map(|arg| arg.to_string()).collect();

    self.executor.execute(params);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct TestShell {
    outputs: Vec<String>,
    executed: Option<Vec<String>>,
  }

  impl TestShell {
    fn new(outputs: Vec<String>) -> TestShell {
      TestShell {
        executed: None,
        outputs,
      }
    }
  }

  impl Executor for TestShell {
    fn execute(&mut self, args: Vec<String>) -> String {
      self.executed = Some(args);
      self.outputs.pop().unwrap()
    }

    fn last_executed(&self) -> Option<Vec<String>> {
      self.executed.clone()
    }
  }

  #[test]
  fn retrieve_active_pane() {
    let last_command_outputs = vec!["%97:100:24:1:0:active\n%106:100:24:1:0:nope\n%107:100:24:1:0:nope\n".to_string()];
    let mut executor = TestShell::new(last_command_outputs);
    let mut swapper = Swapper::new(
      Box::new(&mut executor),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      false,
    );

    swapper.capture_active_pane();

    assert_eq!(swapper.active_pane_id.unwrap(), "%97");
  }

  #[test]
  fn swap_panes() {
    let last_command_outputs = vec![
      "".to_string(),
      "%100".to_string(),
      "".to_string(),
      "%106:100:24:1:0:nope\n%98:100:24:1:0:active\n%107:100:24:1:0:nope\n".to_string(),
    ];
    let mut executor = TestShell::new(last_command_outputs);
    let mut swapper = Swapper::new(
      Box::new(&mut executor),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      false,
    );

    swapper.capture_active_pane();
    swapper.execute_thumbs();
    swapper.swap_panes();

    let expectation = vec!["tmux", "swap-pane", "-d", "-s", "%98", "-t", "%100"];

    assert_eq!(executor.last_executed().unwrap(), expectation);
  }

  #[test]
  fn quoted_execution() {
    let last_command_outputs = vec!["Blah blah blah, the ignored user script output".to_string()];
    let mut executor = TestShell::new(last_command_outputs);

    let main_action = "echo \"{}\"".to_string();
    let shift_action = "open \"{}\"".to_string();
    let ctrl_action = "echo \"{}\"".to_string();
    let alt_action = "echo \"{}\"".to_string();
    let multi_action = "open \"{}\"".to_string();
    let mut swapper = Swapper::new(
      Box::new(&mut executor),
      "".to_string(),
      main_action,
      shift_action,
      ctrl_action,
      alt_action,
      multi_action,
      false,
    );

    swapper.content = Some(format!(
      "{do_upcase}:{thumb_text}",
      do_upcase = false,
      thumb_text = "foobar;rm *",
    ));
    swapper.execute_command();

    let expectation = vec![
      "bash",
      // The actual shell command:
      "-c",
      "THUMB=\"$1\"; eval \"$2\"",
      // $0: The non-existent program name.
      "--",
      // $1: The value assigned to THUMB above.
      //     Not interpreted as a shell expression!
      "foobar;rm *",
      // $2: The user script, with {} replaced with ${THUMB},
      //     and will be eval'd with THUMB in scope.
      "echo \"${THUMB}\"",
    ];

    assert_eq!(executor.last_executed().unwrap(), expectation);
  }
}

fn app_args<'a>() -> clap::ArgMatches<'a> {
  App::new("tmux-thumbs")
    .version(crate_version!())
    .about("A lightning fast version of tmux-fingers, copy/pasting tmux like vimium/vimperator")
    .arg(
      Arg::with_name("dir")
        .help("Directory where to execute thumbs")
        .long("dir")
        .default_value(""),
    )
    .arg(
      Arg::with_name("main-action")
        .help("Command to execute after choose a hint")
        .long("main-action")
        .default_value("tmux set-buffer -- \"{}\" && tmux display-message \"Copied {}\""),
    )
    .arg(
      Arg::with_name("shift-action")
        .help("Command to execute after choose a hint with SHIFT key pressed (Upcase)")
        .long("shift-action")
        .default_value("tmux set-buffer -- \"{}\" && tmux paste-buffer && tmux display-message \"Copied {}\""),
    )
    .arg(
      Arg::with_name("ctrl-action")
        .help("Command to execute after choose a hint with CTRL key pressed")
        .long("ctrl-action")
        .default_value("tmux display-message \"No CTRL action configured! Hint: {}\""),
    )
    .arg(
      Arg::with_name("alt-action")
        .help("Command to execute after choose a hint with ALT key pressed")
        .long("alt-action")
        .default_value("tmux display-message \"No ALT action configured! Hint: {}\" && echo \"FOO $MODIFIER ~ $HINT BAR\" > /tmp/tx.txt"),
    )
    .arg(
      Arg::with_name("multi-action")
        .help("Command to execute after choose multiple hints")
        .long("multi-action")
        .default_value("tmux set-buffer -- \"{}\" && tmux paste-buffer && tmux display-message \"Multi copied {}\""),
    )
    .arg(
      Arg::with_name("osc52")
        .help("Print OSC52 copy escape sequence in addition to running the pick command")
        .long("osc52")
        .short("o"),
    )
    .get_matches()
}

fn main() -> std::io::Result<()> {
  let args = app_args();
  let dir = args.value_of("dir").unwrap();
  let main_action = args.value_of("main-action").unwrap();
  let shift_action = args.value_of("shift-action").unwrap();
  let ctrl_action = args.value_of("ctrl-action").unwrap();
  let alt_action = args.value_of("alt-action").unwrap();
  let multi_action = args.value_of("multi-action").unwrap();
  let osc52 = args.is_present("osc52");

  if dir.is_empty() {
    panic!("Invalid tmux-thumbs execution. Are you trying to execute tmux-thumbs directly?")
  }

  let mut executor = RealShell::new();
  let mut swapper = Swapper::new(
    Box::new(&mut executor),
    dir.to_string(),
    main_action.to_string(),
    shift_action.to_string(),
    ctrl_action.to_string(),
    alt_action.to_string(),
    multi_action.to_string(),
    osc52,
  );

  swapper.capture_active_pane();
  swapper.execute_thumbs();
  swapper.swap_panes();
  swapper.resize_pane();
  swapper.wait_thumbs();
  swapper.retrieve_content();
  swapper.destroy_content();
  swapper.execute_command();

  Ok(())
}
