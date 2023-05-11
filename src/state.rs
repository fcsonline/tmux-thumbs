use regex::Regex;
use std::collections::HashMap;
use std::fmt;

const EXCLUDE_PATTERNS: [(&str, &str); 1] = [("bash", r"[[:cntrl:]]\[([0-9]{1,2};)?([0-9]{1,2})?m")];

const PATTERNS: [(&str, &str); 15] = [
  ("markdown_url", r"\[[^]]*\]\(([^)]+)\)"),
  ("url", r"(?P<match>(https?://|git@|git://|ssh://|ftp://|file:///)[^ ]+)"),
  (
    "diff_summary",
    r"diff --git a/([.\w\-@~\[\]]+?/[.\w\-@\[\]]++) b/([.\w\-@~\[\]]+?/[.\w\-@\[\]]++)",
  ),
  ("diff_a", r"--- a/([^ ]+)"),
  ("diff_b", r"\+\+\+ b/([^ ]+)"),
  ("docker", r"sha256:([0-9a-f]{64})"),
  ("path", r"(?P<match>([.\w\-@$~\[\]]+)?(/[.\w\-@$\[\]]+)+)"),
  ("color", r"#[0-9a-fA-F]{6}"),
  ("uid", r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"),
  ("ipfs", r"Qm[0-9a-zA-Z]{44}"),
  ("sha", r"[0-9a-f]{7,40}"),
  ("ip", r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"),
  ("ipv6", r"[A-f0-9:]+:+[A-f0-9:]+[%\w\d]+"),
  ("address", r"0x[0-9a-fA-F]+"),
  ("number", r"[0-9]{4,}"),
];

#[derive(Clone)]
pub struct Match<'a> {
  pub start: usize,
  pub end: usize,
  pub prev_end: usize,
  pub pattern: &'a str,
  pub text: &'a str,
  pub hint: Option<String>,
}

impl<'a> fmt::Debug for Match<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "Match {{ start: {}, end: {}, pattern: {}, text: {}, hint: <{}> }}",
      self.start,
      self.end,
      self.pattern,
      self.text,
      self.hint.clone().unwrap_or("<undefined>".to_string())
    )
  }
}

impl<'a> PartialEq for Match<'a> {
  fn eq(&self, other: &Match) -> bool {
    self.start == other.start && self.end == other.end
  }
}

pub struct State<'a> {
  pub output: &'a str,
  alphabet: &'a str,
  regexp: &'a Vec<&'a str>,
}

impl<'a> State<'a> {
  pub fn new(output: &'a str, alphabet: &'a str, regexp: &'a Vec<&'a str>) -> State<'a> {
    State {
      output,
      alphabet,
      regexp,
    }
  }

  pub fn matches(&self, reverse: bool, unique: bool) -> Vec<Match<'a>> {
    let mut matches = Vec::new();

    let exclude_patterns = EXCLUDE_PATTERNS
      .iter()
      .map(|tuple| (tuple.0, Regex::new(tuple.1).unwrap()))
      .collect::<Vec<_>>();

    let custom_patterns = self
      .regexp
      .iter()
      .map(|regexp| ("custom", Regex::new(regexp).expect("Invalid custom regexp")))
      .collect::<Vec<_>>();

    let patterns = PATTERNS
      .iter()
      .map(|tuple| (tuple.0, Regex::new(tuple.1).unwrap()))
      .collect::<Vec<_>>();

    // This order determines the priority of pattern matching
    let all_patterns = [exclude_patterns, custom_patterns, patterns].concat();

    let mut chunk: &str = self.output;
    let mut offset: usize = 0;
    let mut prev_end: usize = 0;

    loop {
      // For this line we search which patterns match, all of them.
      let submatches = all_patterns
        .iter()
        .filter_map(|tuple| tuple.1.captures_iter(chunk).next().map(
          |captures| {
            let m = captures.get(0).unwrap();
            (tuple.0, captures, m.start(), m.end())
          }
        ))
      .collect::<Vec<_>>();

      // Then, we search for the match with the lowest index
      let first_match_option = submatches.iter().min_by(|x, y| x.2.cmp(&y.2));

      if let Some((name, captures, _m_start, m_end)) = first_match_option {
        // Never hint or broke bash color sequences, but process it
        if *name != "bash" {
          let matches_start: Vec<(&str, usize)> = if let Some(mat) = captures.name("match") {
            [(mat.as_str(), mat.start())].to_vec()
          } else if captures.len() > 1 {
            captures
              .iter()
              .skip(1)
              .flatten()
              .map(|mat| (mat.as_str(), mat.start()))
              .collect::<Vec<(&str, usize)>>()
          } else {
            let mat = captures.get(0).unwrap();
            [(mat.as_str(), mat.start())].to_vec()
          };

          for (subtext, substart) in matches_start.iter() {
            let start = offset + *substart;
            let end = start + subtext.len();
            matches.push(Match {
              start,
              end,
              prev_end,
              pattern: name,
              text: subtext,
              hint: None,
            });
            prev_end = end;
          }
        }

        chunk = chunk.get(*m_end..).expect("Unknown chunk");
        offset += *m_end;
      } else {
        break;
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

    matches
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn match_reverse() {
    let output = "lorem 127.0.0.1 lorem 255.255.255.255 lorem 127.0.0.1 lorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 3);
    assert_eq!(results.first().unwrap().hint.clone().unwrap(), "a");
    assert_eq!(results.last().unwrap().hint.clone().unwrap(), "c");
  }

  #[test]
  fn match_unique() {
    let output = "lorem 127.0.0.1 lorem 255.255.255.255 lorem 127.0.0.1 lorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, true);

    assert_eq!(results.len(), 3);
    assert_eq!(results.first().unwrap().hint.clone().unwrap(), "a");
    assert_eq!(results.last().unwrap().hint.clone().unwrap(), "a");
  }

  #[test]
  fn match_docker() {
    let output = "latest sha256:30557a29d5abc51e5f1d5b472e79b7e296f595abcf19fe6b9199dbbc809c6ff4 20 hours ago";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 1);
    assert_eq!(
      results.get(0).unwrap().text,
      "30557a29d5abc51e5f1d5b472e79b7e296f595abcf19fe6b9199dbbc809c6ff4"
    );
  }

  #[test]
  fn match_bash() {
    let output = "path: [32m/var/log/nginx.log[m\npath: [32mtest/log/nginx-2.log:32[mfolder/.nginx@4df2.log";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 3);
    assert_eq!(results.get(0).unwrap().text, "/var/log/nginx.log");
    assert_eq!(results.get(1).unwrap().text, "test/log/nginx-2.log");
    assert_eq!(results.get(2).unwrap().text, "folder/.nginx@4df2.log");
  }

  #[test]
  fn match_paths() {
    let output = "Lorem /tmp/foo/bar_lol, lorem\n Lorem /var/log/boot-strap.log lorem ../log/kern.log lorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 3);
    assert_eq!(results.get(0).unwrap().text, "/tmp/foo/bar_lol");
    assert_eq!(results.get(1).unwrap().text, "/var/log/boot-strap.log");
    assert_eq!(results.get(2).unwrap().text, "../log/kern.log");
  }

  #[test]
  fn match_routes() {
    let output = "Lorem /app/routes/$routeId/$objectId, lorem\n Lorem /app/routes/$sectionId";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 2);
    assert_eq!(results.get(0).unwrap().text, "/app/routes/$routeId/$objectId");
    assert_eq!(results.get(1).unwrap().text, "/app/routes/$sectionId");
  }

  #[test]
  fn match_home() {
    let output = "Lorem ~/.gnu/.config.txt, lorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().text, "~/.gnu/.config.txt");
  }

  #[test]
  fn match_slugs() {
    let output = "Lorem dev/api/[slug]/foo, lorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().text, "dev/api/[slug]/foo");
  }

  #[test]
  fn match_uids() {
    let output = "Lorem ipsum 123e4567-e89b-12d3-a456-426655440000 lorem\n Lorem lorem lorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 1);
  }

  #[test]
  fn match_shas() {
    let output = "Lorem fd70b5695 5246ddf f924213 lorem\n Lorem 973113963b491874ab2e372ee60d4b4cb75f717c lorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 4);
    assert_eq!(results.get(0).unwrap().text, "fd70b5695");
    assert_eq!(results.get(1).unwrap().text, "5246ddf");
    assert_eq!(results.get(2).unwrap().text, "f924213");
    assert_eq!(
      results.get(3).unwrap().text,
      "973113963b491874ab2e372ee60d4b4cb75f717c"
    );
  }

  #[test]
  fn match_ips() {
    let output = "Lorem ipsum 127.0.0.1 lorem\n Lorem 255.255.10.255 lorem 127.0.0.1 lorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 3);
    assert_eq!(results.get(0).unwrap().text, "127.0.0.1");
    assert_eq!(results.get(1).unwrap().text, "255.255.10.255");
    assert_eq!(results.get(2).unwrap().text, "127.0.0.1");
  }

  #[test]
  fn match_ipv6s() {
    let output = "Lorem ipsum fe80::2:202:fe4 lorem\n Lorem 2001:67c:670:202:7ba8:5e41:1591:d723 lorem fe80::2:1 lorem ipsum fe80:22:312:fe::1%eth0";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 4);
    assert_eq!(results.get(0).unwrap().text, "fe80::2:202:fe4");
    assert_eq!(
      results.get(1).unwrap().text,
      "2001:67c:670:202:7ba8:5e41:1591:d723"
    );
    assert_eq!(results.get(2).unwrap().text, "fe80::2:1");
    assert_eq!(results.get(3).unwrap().text, "fe80:22:312:fe::1%eth0");
  }

  #[test]
  fn match_markdown_urls() {
    let output = "Lorem ipsum [link](https://github.io?foo=bar) ![](http://cdn.com/img.jpg) lorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 2);
    assert_eq!(results.get(0).unwrap().pattern, "markdown_url");
    assert_eq!(results.get(0).unwrap().text, "https://github.io?foo=bar");
    assert_eq!(results.get(1).unwrap().pattern, "markdown_url");
    assert_eq!(results.get(1).unwrap().text, "http://cdn.com/img.jpg");
  }

  #[test]
  fn match_urls() {
    let output = "Lorem ipsum https://www.rust-lang.org/tools lorem\n Lorem ipsumhttps://crates.io lorem https://github.io?foo=bar lorem ssh://github.io";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 4);
    assert_eq!(results.get(0).unwrap().text, "https://www.rust-lang.org/tools");
    assert_eq!(results.get(0).unwrap().pattern, "url");
    assert_eq!(results.get(1).unwrap().text, "https://crates.io");
    assert_eq!(results.get(1).unwrap().pattern, "url");
    assert_eq!(results.get(2).unwrap().text, "https://github.io?foo=bar");
    assert_eq!(results.get(2).unwrap().pattern, "url");
    assert_eq!(results.get(3).unwrap().text, "ssh://github.io");
    assert_eq!(results.get(3).unwrap().pattern, "url");
  }

  #[test]
  fn match_addresses() {
    let output = "Lorem 0xfd70b5695 0x5246ddf lorem\n Lorem 0x973113tlorem";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 3);
    assert_eq!(results.get(0).unwrap().text, "0xfd70b5695");
    assert_eq!(results.get(1).unwrap().text, "0x5246ddf");
    assert_eq!(results.get(2).unwrap().text, "0x973113");
  }

  #[test]
  fn match_hex_colors() {
    let output = "Lorem #fd7b56 lorem #FF00FF\n Lorem #00fF05 lorem #abcd00 lorem #afRR00";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 4);
    assert_eq!(results.get(0).unwrap().text, "#fd7b56");
    assert_eq!(results.get(1).unwrap().text, "#FF00FF");
    assert_eq!(results.get(2).unwrap().text, "#00fF05");
    assert_eq!(results.get(3).unwrap().text, "#abcd00");
  }

  #[test]
  fn match_ipfs() {
    let output = "Lorem QmRdbNSxDJBXmssAc9fvTtux4duptMvfSGiGuq6yHAQVKQ lorem Qmfoobar";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 1);
    assert_eq!(
      results.get(0).unwrap().text,
      "QmRdbNSxDJBXmssAc9fvTtux4duptMvfSGiGuq6yHAQVKQ"
    );
  }

  #[test]
  fn match_process_port() {
    let output =
      "Lorem 5695 52463 lorem\n Lorem 973113 lorem 99999 lorem 8888 lorem\n   23456 lorem 5432 lorem 23444";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 8);
  }

  #[test]
  fn match_diff_a() {
    let output = "Lorem lorem\n--- a/src/main.rs";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().text, "src/main.rs");
  }

  #[test]
  fn match_diff_b() {
    let output = "Lorem lorem\n+++ b/src/main.rs";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().text, "src/main.rs");
  }

  #[test]
  fn match_diff_summary() {
    let output = "diff --git a/samples/test1 b/samples/test2";
    let custom = [].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 2);
    assert_eq!(results.get(0).unwrap().text, "samples/test1");
    assert_eq!(results.get(1).unwrap().text, "samples/test2");
  }

  #[test]
  fn priority() {
    let output = "Lorem [link](http://foo.bar) ipsum CUSTOM-52463 lorem ISSUE-123 lorem\nLorem /var/fd70b569/9999.log 52463 lorem\n Lorem 973113 lorem 123e4567-e89b-12d3-a456-426655440000 lorem 8888 lorem\n  https://crates.io/23456/fd70b569 lorem";
    let custom = ["CUSTOM-[0-9]{4,}", "ISSUE-[0-9]{3}"].to_vec();
    let results = State::new(output, "abcd", &custom).matches(false, false);

    assert_eq!(results.len(), 9);
    assert_eq!(results.get(0).unwrap().text, "http://foo.bar");
    assert_eq!(results.get(1).unwrap().text, "CUSTOM-52463");
    assert_eq!(results.get(2).unwrap().text, "ISSUE-123");
    assert_eq!(results.get(3).unwrap().text, "/var/fd70b569/9999.log");
    assert_eq!(results.get(4).unwrap().text, "52463");
    assert_eq!(results.get(5).unwrap().text, "973113");
    assert_eq!(
      results.get(6).unwrap().text,
      "123e4567-e89b-12d3-a456-426655440000"
    );
    assert_eq!(results.get(7).unwrap().text, "8888");
    assert_eq!(results.get(8).unwrap().text, "https://crates.io/23456/fd70b569");
  }
}
