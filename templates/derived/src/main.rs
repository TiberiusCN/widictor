use template::*;
use serde_derive::*;
use std::collections::HashMap;

fn main() {
  let mut params: Params = serde_json::from_reader(std::io::stdin()).unwrap();
  let (lang, lemma, mut alt, gloss, valueable) = match params.com.as_str() {
    "inflection of" => {
      let lang = params.args.remove("1").unwrap()[0].to_owned();
      let lemma = params.args.remove("2").unwrap()[0].to_owned();
      let alt = params.args.remove("3").map(|v| v[0].to_owned());
      (lang, lemma, alt, None, false)
    },
    "noncognate" | "noncog" | "cog" | "cognate" | "desc" | "descendant" | "link" | "l" | "mention" | "m" | "l-self" | "m-self" | "ll" => {
      let lang = params.args.remove("1").unwrap()[0].to_owned();
      let lemma = params.args.remove("2").unwrap()[0].to_owned();
      let alt = params.args.remove("3").or_else(|| params.args.remove("alt")).map(|v| v[0].to_owned());
      let gloss = params.args.remove("4").or_else(|| params.args.remove("gloss")).or_else(|| params.args.remove("t")).map(|v| v[0].to_owned());
      (lang, lemma, alt, gloss, true)
    },
    "inherited" | "inh" | "derived" | "der" | "bor" | "borrowed" => {
      let lang = params.args.remove("2").unwrap()[0].to_owned();
      let lemma = params.args.remove("3").unwrap()[0].to_owned();
      let alt = params.args.remove("4").or_else(|| params.args.remove("alt")).map(|v| v[0].to_owned());
      let gloss = params.args.remove("5").or_else(|| params.args.remove("gloss")).or_else(|| params.args.remove("t")).map(|v| v[0].to_owned());
      (lang, lemma, alt, gloss, true)
    },
    x => panic!("unsupported template: {}", x),
  };
  if let Some(v) = alt.as_ref() {
    if v.is_empty() {
      alt = None;
    }
  }
  let conv_lemma = convert_lemma(&lang, &lemma);
  let (subwords, lemma) = if let Some(conv_lemma) = conv_lemma {
    (vec![conv_lemma.clone()], conv_lemma)
  } else {
    (Vec::new(), lemma)
  };
  let value = if lemma.is_empty() { None } else { Some(lemma.clone()) };
  let value = alt.or_else(|| value);
  let gloss = gloss.map(|v| format!(": }}{}{{", v)).unwrap_or_default();
  let value = value.map(|value|format!("{{{} ({}{})}}", value, lang, gloss));
  serde_json::to_writer(std::io::stdout(), &Word {
    subwords,
    value: if valueable { value } else { None },
    .. Default::default()
  }).unwrap()
}

fn convert_lemma(lang: &str, lemma: &str) -> Option<String> {
  // DECOMPOSED!
  // &hex-id; = char(hex-id)
  // && = &
  // https://en.wiktionary.org/wiki/Module:languages/data2
  use unicode_normalization::*;
  let mut lemma: String = lemma.nfd().collect();
  let config_path = directories::ProjectDirs::from("com", "apqm", "widictor").unwrap().config_dir().join("derived").join("language.json");
  let f = std::fs::File::open(config_path).unwrap();
  let langs: Languages = serde_json::from_reader(f).unwrap();
  let lang = if let Some(lang) = langs.languages.get(lang) {
    Some(lang)
  } else {
    eprintln!("Language {} not found", lang);
    None
  }?;
  let mut replaces = HashMap::new();
  for (c, replace) in lang.convert.iter() {
    let mut out = String::new();
    let mut hex_id = false;
    let mut hex = 0u32;
    for c in c.chars() {
      match c {
        ';' if hex_id => {
          use std::convert::TryFrom;
          out.push(char::try_from(hex).unwrap());
          hex_id = false;
        },
        '&' if hex_id && hex == 0 => {
          hex_id = false;
          out.push('&');
        },
        '&' if !hex_id => {
          hex_id = true;
          hex = 0;
        },
        x if hex_id  => {
          hex *= 16;
          hex += match x {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'a' | 'A' => 10,
            'b' | 'B' => 11,
            'c' | 'C' => 12,
            'd' | 'D' => 13,
            'e' | 'E' => 14,
            'f' | 'F' => 15,
            x => panic!("bad hex digit: {}", x),
          };
        },
        x => out.push(x),
      }
    }
    replaces.insert(out, replace);
  }
  for (from, to) in replaces.iter() {
    lemma = lemma.replace(from, to);
  }
  lemma = lemma.nfc().collect();
  Some(lemma)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Languages {
  languages: HashMap<String, Language>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Language {
  convert: HashMap<String, String>,
}
