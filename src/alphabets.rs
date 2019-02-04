use std::collections::HashMap;

const ALPHABETS: [(&'static str, &'static str); 21] = [
  ("abcd", "abcd"),
  ("qwerty", "asdfqwerzxcvjklmiuopghtybn"),
  ("qwerty-homerow", "asdfjklgh"),
  ("qwerty-left-hand", "asdfqwerzcxv"),
  ("qwerty-right-hand", "jkluiopmyhn"),
  ("azerty", "qsdfazerwxcvjklmuiopghtybn"),
  ("azerty-homerow", "qsdfjkmgh"),
  ("azerty-left-hand", "qsdfazerwxcv"),
  ("azerty-right-hand", "jklmuiophyn"),
  ("qwertz", "asdfqweryxcvjkluiopmghtzbn"),
  ("qwertz-homerow", "asdfghjkl"),
  ("qwertz-left-hand", "asdfqweryxcv"),
  ("qwertz-right-hand", "jkluiopmhzn"),
  ("dvorak", "aoeuqjkxpyhtnsgcrlmwvzfidb"),
  ("dvorak-homerow", "aoeuhtnsid"),
  ("dvorak-left-hand", "aoeupqjkyix"),
  ("dvorak-right-hand", "htnsgcrlmwvz"),
  ("colemak", "arstqwfpzxcvneioluymdhgjbk"),
  ("colemak-homerow", "arstneiodh"),
  ("colemak-left-hand", "arstqwfpzxcv"),
  ("colemak-right-hand", "neioluymjhk"),
];

pub struct Alphabet<'a> {
  letters: &'a str
}

impl<'a> Alphabet<'a> {
  fn new(letters: &'a str) -> Alphabet {
    Alphabet{
      letters: letters
    }
  }

  pub fn hints(&self, matches: usize) -> Vec<String> {
    let letters: Vec<String> = self.letters.chars().map(|s| s.to_string()).collect();

    if matches <= letters.len() {
      letters.iter().take(matches).map(|x| x.clone()).collect::<Vec<String>>()
    } else {
      // TODO
      let mut f = letters.iter().take(letters.len() - 1).map(|x| x.clone()).collect::<Vec<String>>();
      let l = letters.iter().last().unwrap();
      let mut g = letters.iter().take(matches - (letters.len() - 1)).map(|s| l.clone() + s).collect();

      f.append(&mut g);

      f
    }
  }
}

pub fn get_alphabet(alphabet_name: &str) -> Alphabet {
  let alphabets: HashMap<&str, &str> = ALPHABETS.iter().cloned().collect();

  Alphabet::new(alphabets[alphabet_name])
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn simple_matches () {
    let alphabet = Alphabet::new("abcd");
    let hints = alphabet.hints(3);
    assert_eq!(hints, ["a", "b", "c"]);
  }

  #[test]
  fn composed_matches () {
    let alphabet = Alphabet::new("abcd");
    let hints = alphabet.hints(6);
    assert_eq!(hints, ["a", "b", "c", "da", "db", "dc"]);
  }
}
