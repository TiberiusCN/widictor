use template::*;

fn main() {
  let mut params: Params = serde_json::from_reader(std::io::stdin()).unwrap();
  let lang = params.args.remove("1").unwrap()[0].to_owned();
  let mut lemma = params.args.remove("2").unwrap()[0].to_owned();
  match params.com.as_str() {
    "inflection of" => {},
    "link" | "l" | "mention" | "m" | "l-self" | "m-self" | "ll" => {
      // lemma is not required in fact
      lemma = convert_lemma(&lang, &lemma);
    },
    x => panic!("unsupported template: {}", x),
  }
  serde_json::to_writer(std::io::stdout(), &Word {
    subwords: vec![params.args.remove("2").unwrap()[0].to_owned()],
    .. Default::default()
  }).unwrap()
}

fn convert_lemma(lang: &str, lemma: &str) -> String {
  // https://en.wiktionary.org/wiki/Module:languages/data2
}
