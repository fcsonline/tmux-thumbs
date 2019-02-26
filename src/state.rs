use std::collections::HashMap;
use regex::Regex;

pub struct Match<'a> {
  pub x: i32,
  pub y: i32,
  pub text: &'a str,
  pub hint: Option<String>
}

impl<'a> PartialEq for Match<'a> {
  fn eq(&self, other: &Match) -> bool {
    self.x == other.x && self.y == other.y
  }
}

pub struct State<'a> {
  pub lines: Vec<&'a str>,
  alphabet: &'a str,
  pub skip: usize,
}

impl<'a> State<'a> {
  pub fn new(lines: Vec<&'a str>, alphabet: &'a str) -> State<'a> {
    State{
      lines: lines,
      alphabet: alphabet,
      skip: 0
    }
  }

  pub fn matches(&self, reverse: bool, unique: bool) -> Vec<Match<'a>> {
    let mut matches = Vec::new();
    let mut patterns = Vec::new();

    patterns.push(Regex::new(r"((https?://|git@|git://|ssh://|ftp://|file:///)[\w?=%/_.:,;~@!#$&()*+-]*)").unwrap()); // Urls
    patterns.push(Regex::new(r"[^ ]+/[^ ]+").unwrap()); // Paths
    patterns.push(Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap()); // Uid
    patterns.push(Regex::new(r"[0-9a-f]{7,40}").unwrap()); // Sha id
    patterns.push(Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap()); // Ip address
    patterns.push(Regex::new(r"0x[0-9a-fA-F]+").unwrap()); // Address
    patterns.push(Regex::new(r"[0-9]{4,}").unwrap()); // Process or ports

    for (index, line) in self.lines.iter().enumerate() {
      let mut chunk: &str = line;
      let mut offset: i32 = 0;

      loop {
        let submatches = patterns.iter().filter_map(|pattern| pattern.find_iter(chunk).nth(0)).collect::<Vec<_>>();
        let first_match_option = submatches.iter().min_by(|x, y| x.start().cmp(&y.start()));

        if let Some(first_match) = first_match_option {
          matches.push(Match{
            x: offset + first_match.start() as i32,
            y: index as i32,
            text: &chunk[first_match.start()..first_match.end()],
            hint: None
          });

          chunk = chunk.get(first_match.end()..).expect("Unknown chunk");
          offset = offset + first_match.end() as i32;
        } else {
          break;
        }
      }
    }

    let alphabet = super::alphabets::get_alphabet(self.alphabet);
    let mut hints = alphabet.hints(matches.len());

    // This looks wrong but we do a pop after
    if !reverse {
      hints.reverse();
    } else {
      matches.reverse();
      hints.reverse();
    }

    if unique {
      let mut previous: HashMap<&str, String> = HashMap::new();

      for mat in &mut matches {
        if let Some(previous_hint) = previous.get(mat.text) {
          mat.hint = Some(previous_hint.clone());
        } else if let Some(hint) = hints.pop() {
          mat.hint = Some(hint.to_string().clone());
          previous.insert(mat.text, hint.to_string().clone());
        }
      }
    } else {
      for mat in &mut matches {
        if let Some(hint) = hints.pop() {
          mat.hint = Some(hint.to_string().clone());
        }
      }
    }

    if reverse {
      matches.reverse();
    }

    return matches;
  }

  pub fn prev(&mut self) {
    if self.skip > 0 {
      self.skip = self.skip - 1;
    }
  }

  pub fn next(&mut self) {
    self.skip = self.skip + 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn match_lines_flags(output: &str, reverse: bool, unique: bool) -> Vec<Match> {
    let lines = output.split("\n").collect::<Vec<&str>>();
    let state = State::new(lines, "abcd");

    state.matches(reverse, unique)
  }

  fn match_lines(output: &str) -> Vec<Match> {
    match_lines_flags(output, false, false)
  }

  #[test]
  fn match_reverse () {
    let output = "lorem 127.0.0.1 lorem 255.255.255.255 lorem 127.0.0.1 lorem";

    let results = match_lines_flags(output, false, false);

    assert_eq!(results.len(), 3);
    assert_eq!(results.first().unwrap().hint.clone().unwrap(), "a");
    assert_eq!(results.last().unwrap().hint.clone().unwrap(), "c");
  }

  #[test]
  fn match_unique () {
    let output = "lorem 127.0.0.1 lorem 255.255.255.255 lorem 127.0.0.1 lorem";

    let results = match_lines_flags(output, false, true);

    assert_eq!(results.len(), 3);
    assert_eq!(results.first().unwrap().hint.clone().unwrap(), "a");
    assert_eq!(results.last().unwrap().hint.clone().unwrap(), "a");
  }

  #[test]
  fn match_paths () {
    let output = "Lorem /tmp/foo/bar lorem\n Lorem /var/log/bootstrap.log lorem ../log/kern.log lorem";

    assert_eq!(match_lines(output).len(), 3);
  }

  #[test]
  fn match_uids () {
    let output = "Lorem ipsum 123e4567-e89b-12d3-a456-426655440000 lorem\n Lorem lorem lorem";

    assert_eq!(match_lines(output).len(), 1);
  }

  #[test]
  fn match_shas () {
    let output = "Lorem fd70b5695 5246ddf f924213 lorem\n Lorem 973113963b491874ab2e372ee60d4b4cb75f717c lorem";

    assert_eq!(match_lines(output).len(), 4);
  }

  #[test]
  fn match_ips () {
    let output = "Lorem ipsum 127.0.0.1 lorem\n Lorem 255.255.10.255 lorem 127.0.0.1 lorem";

    assert_eq!(match_lines(output).len(), 3);
  }

  #[test]
  fn match_urls () {
    let output = "Lorem ipsum https://www.rust-lang.org/tools lorem\n Lorem https://crates.io lorem https://github.io lorem ssh://github.io";

    assert_eq!(match_lines(output).len(), 4);
  }

  #[test]
  fn match_addresses () {
    let output = "Lorem 0xfd70b5695 0x5246ddf lorem\n Lorem 0x973113 lorem";

    assert_eq!(match_lines(output).len(), 3);
  }

  #[test]
  fn match_process_port () {
    let output = "Lorem 5695 52463 lorem\n Lorem 973113 lorem 99999 lorem 8888 lorem\n   23456 lorem 5432 lorem 23444";

    assert_eq!(match_lines(output).len(), 8);
  }

  #[test]
  fn priority () {
    let output = "Lorem /var/fd70b569/9999.log 52463 lorem\n Lorem 973113 lorem 123e4567-e89b-12d3-a456-426655440000 lorem 8888 lorem\n  https://crates.io/23456/fd70b569 lorem";

    // Matches
    // /var/fd70b569/9999.log
    // 52463
    // 973113
    // 123e4567-e89b-12d3-a456-426655440000
    // 8888
    // https://crates.io/23456/fd70b569
    assert_eq!(match_lines(output).len(), 6);
  }
}
