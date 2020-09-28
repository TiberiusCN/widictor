use template::*;
use serde_json::to_writer;
use std::env::var as env_var;
use std::collections::{HashSet, HashMap};

fn main() {
  let word = env_var("ENV_MAINWORD").unwrap();
  let lang = env_var("ENV_0").unwrap();

  let data = match lang.as_str() {
    "la-verb" => unimplemented!(), //latina(&word),
    u => panic!("unsupported: {}", u),
  };

  to_writer(std::io::stdout(), &data).unwrap();
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
enum Rule {
  Set,
  Allow,
  Forbidden,
  Auto,
}
impl Default for Rule {
  fn default() -> Self { Self::Auto }
}
impl From<Rule> for bool {
  fn from(src: Rule) -> Self {
    src == Rule::Set
  }
}

#[derive(Debug, Clone)]
struct Conjugation {
  conj: ConjugationType,

  lemma: String,
  iv: bool, // + false, ++ true
  deponent: bool,
  semi_deponent: bool,

  infect_stem: Vec<String>,
  perfect_stem: Vec<String>,
  supine_stem: Vec<String>,

  // ToDo: no pass links in parameters in widictor

  table_generator: Vec<bool>,

  tags: HashSet<String>,
}

impl Conjugation {
  fn new(conj: ConjugationType, lemma: &str) -> Self {
    let mut table_generator = Vec::with_capacity(<usize as TableContext>::max());
    for _ in 0..<usize as TableContext>::max() {
      table_generator.push(true);
    }
    let lemma = lemma.to_owned();
    Self {
      conj,
      lemma,
      iv: false,
      deponent: false,
      semi_deponent: false,
      infect_stem: Vec::new(),
      perfect_stem: Vec::new(),
      supine_stem: Vec::new(),
      table_generator,
      tags: HashSet::new(),
    }
  }
}

#[derive(Debug, Clone)]
enum ConjugationType {
  C1, C2, C3, C4,
  Irreg(Vec<String>),
}

// TODO: suffix/prefix = subwords and no conjugation and tags

impl Conjugation {
  fn finish(mut self) -> Self {
    if self.semi_deponent || self.deponent {
      self.supine_stem = self.perfect_stem;
      self.perfect_stem = Vec::new();
    }

    if let Some(supine) = self.supine_stem.get(0) {
      self.supine_stem = supine.split('/').map(|v| v.to_owned()).collect();
    }
    if let Some(perfect) = self.perfect_stem.get(0) {
      self.perfect_stem = perfect.split('/').map(|v| v.to_owned()).collect();
    }
    
    let (auto_infect_stem, auto_perfect_stem, auto_supine_stem) = match self.conj {
      ConjugationType::C1 => {
        let stem = self.lemma.strip_suffix("ō");
        if let Some(stem) = stem {
          let perf = vec![format!("{}āv", stem)];
          if self.iv { panic!("iv?"); }
          let sup = vec![format!("{}āt", stem)];
          (vec![stem.to_owned()], perf, sup)
        } else {
          (Vec::new(), Vec::new(), Vec::new())
        }
      },
      ConjugationType::C2 => {
        let stem = self.lemma.strip_suffix("eō");
        if let Some(stem) = stem {
          let perf = vec![format!("{}uī", stem)];
          if self.iv { panic!("iv?"); }
          let sup = vec![format!("{}it", stem)];
          (vec![stem.to_owned()], perf, sup)
        } else {
          (Vec::new(), Vec::new(), Vec::new())
        }
      },
      ConjugationType::C4 => {
        let stem = self.lemma.strip_suffix("iō");
        if let Some(stem) = stem {
          let mut perf = vec![format!("{}īv", stem)];
          if self.iv { perf.push(format!("{}ī", stem)); }
          let sup = vec![format!("{}īt", stem)];
          (vec![stem.to_owned()], perf, sup)
        } else {
          (Vec::new(), Vec::new(), Vec::new())
        }
      },
      _ => (Vec::new(), Vec::new(), Vec::new()),
    };
    if self.infect_stem.is_empty() {
      self.infect_stem = auto_infect_stem;
    }
    if self.perfect_stem.is_empty() {
      self.perfect_stem = auto_perfect_stem;
    }
    if self.supine_stem.is_empty() {
      self.supine_stem = auto_supine_stem;
    }

    self
  }

  fn nopass(&mut self) {
    for (i, val) in self.table_generator.iter_mut().enumerate() {
      if i.passive() {
        *val = false;
      }
    }
    self.tags.insert("nopass".to_string());
  }

  fn gen_x(stem: &[String], suffix: &str, end: &[&str]) -> String {
    let mut out = String::new();
    for s in stem {
      for e in end {
        out += &format!("{}{}{}/", s, suffix, e);
      }
    }
    if !out.is_empty() { out.pop(); out } else { "—".to_owned() }
  }

  fn gen_indicative(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
    let gen_infect_end = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (voice, tantum, person, time) {
        (Voice::Active, Tantum::Singular, Person::First, Time::Present)     => vec!["ō"],
        (Voice::Active, Tantum::Singular, Person::First, Time::Imperfect)   => vec!["am"],
        (Voice::Active, Tantum::Singular, Person::First, Time::Future)      => vec!["ō"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Present)    => vec!["ās"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Imperfect)  => vec!["ās"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Future)     => vec!["is"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Present)     => vec!["at"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Imperfect)   => vec!["at"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Future)      => vec!["it"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Present)       => vec!["āmus"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Imperfect)     => vec!["āmus"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Future)        => vec!["imus"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Present)      => vec!["ātis"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Imperfect)    => vec!["ātis"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Future)       => vec!["itis"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Present)       => vec!["ant"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Imperfect)     => vec!["ant"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Future)        => vec!["unt"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Present)    => vec!["or"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Imperfect)  => vec!["ar"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Future)     => vec!["or"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Present)   => vec!["āris","āre"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Imperfect) => vec!["āris","āre"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Future)    => vec!["eris","ere"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Present)    => vec!["ātur"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Imperfect)  => vec!["ātur"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Future)     => vec!["itur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Present)      => vec!["āmur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Imperfect)    => vec!["āmur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Future)       => vec!["imur"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Present)     => vec!["āmini"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Imperfect)   => vec!["āmini"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Future)      => vec!["imini"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Present)      => vec!["antur"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Imperfect)    => vec!["antur"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Future)       => vec!["untur"],
        _ => unreachable!(),
      }
    };
    let gen_perfect_active_end = |tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (tantum, person, time) {
        (Tantum::Singular, Person::First, Time::Perfect)        => vec!["ī"],
        (Tantum::Singular, Person::First, Time::Pluperfect)     => vec!["am"],
        (Tantum::Singular, Person::First, Time::FuturePerfect)  => vec!["ō"],
        (Tantum::Singular, Person::Second, Time::Perfect)       => vec!["istī"],
        (Tantum::Singular, Person::Second, Time::Pluperfect)    => vec!["ās"],
        (Tantum::Singular, Person::Second, Time::FuturePerfect) => vec!["īs"],
        (Tantum::Singular, Person::Third, Time::Perfect)        => vec!["it"],
        (Tantum::Singular, Person::Third, Time::Pluperfect)     => vec!["at"],
        (Tantum::Singular, Person::Third, Time::FuturePerfect)  => vec!["it"],
        (Tantum::Plural, Person::First, Time::Perfect)          => vec!["imus"],
        (Tantum::Plural, Person::First, Time::Pluperfect)       => vec!["āmus"],
        (Tantum::Plural, Person::First, Time::FuturePerfect)    => vec!["īmus"],
        (Tantum::Plural, Person::Second, Time::Perfect)         => vec!["istis"],
        (Tantum::Plural, Person::Second, Time::Pluperfect)      => vec!["ātis"],
        (Tantum::Plural, Person::Second, Time::FuturePerfect)   => vec!["ītis"],
        (Tantum::Plural, Person::Third, Time::Perfect)          => vec!["ērunt","ēre"],
        (Tantum::Plural, Person::Third, Time::Pluperfect)       => vec!["ant"],
        (Tantum::Plural, Person::Third, Time::FuturePerfect)    => vec!["int"],
        _ => unreachable!(),
      }
    };
    let gen_perfect_passive_end = |tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (tantum, person, time) {
        (Tantum::Singular, Person::First, Time::Perfect)        => vec!["us sum"],
        (Tantum::Singular, Person::First, Time::Pluperfect)     => vec!["us eram"],
        (Tantum::Singular, Person::First, Time::FuturePerfect)  => vec!["us erō"],
        (Tantum::Singular, Person::Second, Time::Perfect)       => vec!["us es"],
        (Tantum::Singular, Person::Second, Time::Pluperfect)    => vec!["us erās"],
        (Tantum::Singular, Person::Second, Time::FuturePerfect) => vec!["us eris","us ere"],
        (Tantum::Singular, Person::Third, Time::Perfect)        => vec!["us est"],
        (Tantum::Singular, Person::Third, Time::Pluperfect)     => vec!["us erat"],
        (Tantum::Singular, Person::Third, Time::FuturePerfect)  => vec!["us erit"],
        (Tantum::Plural, Person::First, Time::Perfect)          => vec!["ī sumus"],
        (Tantum::Plural, Person::First, Time::Pluperfect)       => vec!["ī erāmus"],
        (Tantum::Plural, Person::First, Time::FuturePerfect)    => vec!["ī erimus"],
        (Tantum::Plural, Person::Second, Time::Perfect)         => vec!["ī estis"],
        (Tantum::Plural, Person::Second, Time::Pluperfect)      => vec!["ī erātis"],
        (Tantum::Plural, Person::Second, Time::FuturePerfect)   => vec!["ī eritis"],
        (Tantum::Plural, Person::Third, Time::Perfect)          => vec!["ī sunt"],
        (Tantum::Plural, Person::Third, Time::Pluperfect)       => vec!["ī erant"],
        (Tantum::Plural, Person::Third, Time::FuturePerfect)    => vec!["ī erunt"],
        _ => unreachable!(),
      }
    };
    let gen_suffix = |time: Time| -> &'static str {
      match time {
        Time::Present => "",
        Time::Imperfect => "āb",
        Time::Future => "āb",
        Time::Perfect => "",
        Time::Pluperfect => "er",
        Time::FuturePerfect => "er",
      }
    };

    let gen_infect = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> String {
      let stem = self.infect_stem.as_slice();
      let suffix = gen_suffix(time);
      let end = gen_infect_end(voice, tantum, person, time);
      Self::gen_x(stem, suffix, &end)
    };
    let gen_perfect = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> String {
      match voice {
        Voice::Active => Self::gen_x(self.perfect_stem.as_slice(), gen_suffix(time), &gen_perfect_active_end(tantum, person, time)),
        Voice::Passive => Self::gen_x(self.supine_stem.as_slice(), "", &gen_perfect_passive_end(tantum, person, time)),
      }
    };

    match time {
      Time::Present | Time::Imperfect | Time::Future => gen_infect(voice, tantum, person, time),
      Time::Perfect | Time::Pluperfect | Time::FuturePerfect => gen_perfect(voice, tantum, person, time),
    }
  }

  fn gen_subjunctive(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
    let gen_infect_end = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (voice, tantum, person, time) {
        (Voice::Active, Tantum::Singular, Person::First, Time::Present)     => vec!["em"],
        (Voice::Active, Tantum::Singular, Person::First, Time::Imperfect)   => vec!["em"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Present)    => vec!["ēs"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Imperfect)  => vec!["ēs"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Present)     => vec!["et"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Imperfect)   => vec!["et"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Present)       => vec!["ēmus"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Imperfect)     => vec!["ēmus"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Present)      => vec!["ētis"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Imperfect)    => vec!["ētis"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Present)       => vec!["ent"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Imperfect)     => vec!["ent"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Present)    => vec!["er"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Imperfect)  => vec!["er"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Present)   => vec!["ēris","ēre"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Imperfect) => vec!["ēris","ēre"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Present)    => vec!["ētur"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Imperfect)  => vec!["ētur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Present)      => vec!["ēmur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Imperfect)    => vec!["ēmur"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Present)     => vec!["ēmini"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Imperfect)   => vec!["ēmini"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Present)      => vec!["entur"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Imperfect)    => vec!["entur"],
        _ => unreachable!(),
      }
    };
    let gen_perfect_active_end = |tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (tantum, person, time) {
        (Tantum::Singular, Person::First, Time::Perfect)        => vec!["im"],
        (Tantum::Singular, Person::First, Time::Pluperfect)     => vec!["issem"],
        (Tantum::Singular, Person::Second, Time::Perfect)       => vec!["īs"],
        (Tantum::Singular, Person::Second, Time::Pluperfect)    => vec!["issēs"],
        (Tantum::Singular, Person::Third, Time::Perfect)        => vec!["it"],
        (Tantum::Singular, Person::Third, Time::Pluperfect)     => vec!["isset"],
        (Tantum::Plural, Person::First, Time::Perfect)          => vec!["īmus"],
        (Tantum::Plural, Person::First, Time::Pluperfect)       => vec!["issēmus"],
        (Tantum::Plural, Person::Second, Time::Perfect)         => vec!["ītis"],
        (Tantum::Plural, Person::Second, Time::Pluperfect)      => vec!["issētis"],
        (Tantum::Plural, Person::Third, Time::Perfect)          => vec!["int"],
        (Tantum::Plural, Person::Third, Time::Pluperfect)       => vec!["issent"],
        _ => unreachable!(),
      }
    };
    let gen_perfect_passive_end = |tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (tantum, person, time) {
        (Tantum::Singular, Person::First, Time::Perfect)        => vec!["us sim"],
        (Tantum::Singular, Person::First, Time::Pluperfect)     => vec!["us essem"],
        (Tantum::Singular, Person::Second, Time::Perfect)       => vec!["us sīs"],
        (Tantum::Singular, Person::Second, Time::Pluperfect)    => vec!["us essēs"],
        (Tantum::Singular, Person::Third, Time::Perfect)        => vec!["us sit"],
        (Tantum::Singular, Person::Third, Time::Pluperfect)     => vec!["us esset"],
        (Tantum::Plural, Person::First, Time::Perfect)          => vec!["ī sīmus"],
        (Tantum::Plural, Person::First, Time::Pluperfect)       => vec!["ī essēmus"],
        (Tantum::Plural, Person::Second, Time::Perfect)         => vec!["ī sītis"],
        (Tantum::Plural, Person::Second, Time::Pluperfect)      => vec!["ī essētis"],
        (Tantum::Plural, Person::Third, Time::Perfect)          => vec!["ī sint"],
        (Tantum::Plural, Person::Third, Time::Pluperfect)       => vec!["ī essent"],
        _ => unreachable!(),
      }
    };
    let gen_suffix = |time: Time| -> &'static str {
      match time {
        Time::Present => "",
        Time::Imperfect => "ār",
        Time::Perfect => "er",
        Time::Pluperfect => "iss",
        _ => unreachable!(),
      }
    };

    let gen_infect = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> String {
      let stem = self.infect_stem.as_slice();
      let suffix = gen_suffix(time);
      let end = gen_infect_end(voice, tantum, person, time);
      Self::gen_x(stem, suffix, &end)
    };
    let gen_perfect = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> String {
      let (stem, end) = match voice {
        Voice::Active => (self.perfect_stem.as_slice(), gen_perfect_active_end(tantum, person, time)),
        Voice::Passive => (self.supine_stem.as_slice(), gen_perfect_passive_end(tantum, person, time)),
      };
      let suffix = gen_suffix(time);
      Self::gen_x(stem, suffix, &end)
    };

    match time {
      Time::Present | Time::Imperfect | Time::Future => gen_infect(voice, tantum, person, time),
      Time::Perfect | Time::Pluperfect | Time::FuturePerfect => gen_perfect(voice, tantum, person, time),
    }
  }

  fn gen_imperative(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
    let gen_end = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (voice, tantum, person, time) {
        (Voice::Active, Tantum::Singular, Person::Second, Time::Present)  => vec!["ā"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Future)   => vec!["āto"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Present) => vec!["āre"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Future)  => vec!["ātor"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Future)    => vec!["ātō"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Future)   => vec!["ātor"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Present)    => vec!["āte"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Future)     => vec!["ātōte"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Present)   => vec!["āminī"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Future)      => vec!["antō"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Future)     => vec!["antor"],
        _ => unreachable!(),
      }
    };
    let stem = self.infect_stem.as_slice();
    let end = gen_end(voice, tantum, person, time);
    Self::gen_x(stem, "", &end)
  }

  fn gen_infinitive(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
    let stem = match (voice, time) {
      (Voice::Active, Time::Present)  => &self.infect_stem,
      (Voice::Active, Time::Perfect)  => &self.perfect_stem,
      (Voice::Active, Time::Future)   => &self.supine_stem,
      (Voice::Passive, Time::Present) => &self.infect_stem,
      (Voice::Passive, Time::Perfect) => &self.supine_stem,
      (Voice::Passive, Time::Future)  => &self.supine_stem,
      _ => unreachable!(),
    };
    let suffix = match (voice, time) {
      (Voice::Active, Time::Present)  => "",
      (Voice::Active, Time::Perfect)  => "isse",
      (Voice::Active, Time::Future)   => "ūr",
      (Voice::Passive, Time::Present) => "",
      (Voice::Passive, Time::Perfect) => "",
      (Voice::Passive, Time::Future)  => "",
      _ => unreachable!(),
    };
    let end = match (voice, time) {
      (Voice::Active, Time::Present)  => vec!["āre"],
      (Voice::Active, Time::Perfect)  => vec![""],
      (Voice::Active, Time::Future)   => vec!["um esse"],
      (Voice::Passive, Time::Present) => vec!["ārī"],
      (Voice::Passive, Time::Perfect) => vec!["um esse"],
      (Voice::Passive, Time::Future)  => vec!["um īrī"],
      _ => unreachable!(),
    };

    Self::gen_x(&stem, suffix, &end)
  }

  fn gen_participle(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
    let stem = match (voice, time) {
      (Voice::Active, Time::Present)  => &self.infect_stem,
      (Voice::Active, Time::Future)   => &self.supine_stem,
      (Voice::Passive, Time::Perfect) => &self.supine_stem,
      (Voice::Passive, Time::Future)  => &self.infect_stem,
      _ => unreachable!(),
    };
    let suffix = match (voice, time) {
      (Voice::Active, Time::Present)  => "āns",
      (Voice::Active, Time::Future)   => "ūr",
      (Voice::Passive, Time::Perfect) => "",
      (Voice::Passive, Time::Future)  => "and",
      _ => unreachable!(),
    };
    let end = match (voice, time) {
      (Voice::Active, Time::Present)  => vec![""],
      (Voice::Active, Time::Future)   => vec!["us"],
      (Voice::Passive, Time::Perfect) => vec!["us"],
      (Voice::Passive, Time::Future)  => vec!["us"],
      _ => unreachable!(),
    };

    Self::gen_x(stem, suffix, &end)
  }

  fn gen_noun(&self, noun: VerbalNoun) -> String {
    let stem = match noun {
      VerbalNoun::GerGen | VerbalNoun::GerDat | VerbalNoun::GerAcc | VerbalNoun::GerAbl => &self.infect_stem,
      VerbalNoun::SupAcc | VerbalNoun::SupAbl => &self.supine_stem,
    };
    let suffix = match noun {
      VerbalNoun::GerGen | VerbalNoun::GerDat | VerbalNoun::GerAcc | VerbalNoun::GerAbl => "and",
      VerbalNoun::SupAcc | VerbalNoun::SupAbl => "",
    };
    let end = match noun {
      VerbalNoun::GerGen => vec!["ī"],
      VerbalNoun::GerDat => vec!["ō"],
      VerbalNoun::GerAcc => vec!["um"],
      VerbalNoun::GerAbl => vec!["ō"],
      VerbalNoun::SupAcc => vec!["um"],
      VerbalNoun::SupAbl => vec!["ū"],
    };
    Self::gen_x(stem, suffix, &end)
  }

  fn gen_table(&self) -> HashMap<String, String> {

    let mut out = HashMap::new();

    for (id, generator) in self.table_generator.iter().enumerate() {
      if !generator { continue; }
      match id.category() {
        Category::Indicative => {
          out.insert(id.code(), self.gen_indicative(id.voice().unwrap(), id.tantum().unwrap(), id.person().unwrap(), id.time().unwrap()));
        },
        Category::Subjunctive => {
          out.insert(id.code(), self.gen_subjunctive(id.voice().unwrap(), id.tantum().unwrap(), id.person().unwrap(), id.time().unwrap()));
        },
        Category::Imperative => {
          out.insert(id.code(), self.gen_imperative(id.voice().unwrap(), id.tantum().unwrap(), id.person().unwrap(), id.time().unwrap()));
        },
        Category::NonFinite => {
          if id.infinitive() { 
            out.insert(id.code(), self.gen_infinitive(id.voice().unwrap(), id.tantum().unwrap(), id.person().unwrap(), id.time().unwrap()));
          } else {
            out.insert(id.code(), self.gen_participle(id.voice().unwrap(), id.tantum().unwrap(), id.person().unwrap(), id.time().unwrap()));
          }
        },
        Category::VerbalNoun => {
          out.insert(id.code(), self.gen_noun(id.noun().unwrap()));
        },
      }
    }
    unimplemented!()
  }
}

/*
fn latina(word: &str) -> TemplateText {
  let noun = if let Ok(one_lemma) = env_var("ENV_1") {
    Noun::new(word, one_lemma)
  } else {
    let mut nouns = Vec::new();
    let mut i = 0;
    while let Ok(lemma) = env_var(&format!("ENV_1_{}", i)) {
      i += 1;
      nouns.push(Noun::new(word, lemma));
    }
    let mut nouns = nouns.into_iter();
    let mut noun = nouns.next().unwrap();
    for variant in nouns {
      noun.merge(variant);
    }
    noun
  };

  let mut subwords = Vec::new();
  if let Ok(word) = env_var("ENV_f") { subwords.push(word); }
  if let Ok(word) = env_var("ENV_m") { subwords.push(word); }

  TemplateText {
    mutation: Some(noun.table),
    lemma: Some(noun.lemmas[0].clone()),
    tags: noun.tags.into_iter().map(|v| v.to_string()).collect(),
    notes: noun.footnote,
    conjugation: None,
    pronunciation: None,
    meanings: None,
    examples: None,
    subwords,
  }
}
f*/

// lemma stem declension
fn parse_lemma(src: &str) -> (String, Option<String>, String) {
  let mut lemmas = vec![String::new()];
  for c in src.chars() {
    match c {
      '<' | '/' => {
        lemmas.push(String::new());
      },
      '>' => { break; },
      x => lemmas.last_mut().unwrap().push(x),
    }
  }
  let decl = lemmas.pop().unwrap();
  let stem = lemmas.pop().unwrap();
  if let Some(lemma) = lemmas.pop() {
    (lemma, Some(stem), decl)
  } else {
    (stem, None, decl)
  }
}

// declension subtypes
fn parse_declension(src: &str) -> (String, Vec<String>) {
  let mut iter = src.split('.');
  (iter.next().unwrap().to_string(), iter.map(|v| v.to_owned()).collect())
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Time {
  Present,
  Imperfect,
  Future,
  Perfect,
  Pluperfect,
  FuturePerfect,
}
impl Time {
  fn code(self) -> &'static str {
    match self {
      Self::Future => "Fut",
      Self::Imperfect => "Imperf",
      Self::FuturePerfect => "FutPerf",
      Self::Perfect => "Perf",
      Self::Pluperfect => "Pluperf",
      Self::Present => "Pres",
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Tantum {
  Singular,
  Plural,
}
impl Tantum {
  fn code(self) -> &'static str {
    match self {
      Self::Plural => "Pl",
      Self::Singular => "Sg",
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Person {
  First,
  Second,
  Third,
}
impl Person {
  fn code(self) -> &'static str {
    match self {
      Self::First => "1",
      Self::Second => "2",
      Self::Third => "3",
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Category {
  Indicative,
  Subjunctive,
  Imperative,
  NonFinite,
  VerbalNoun,
}
impl Category {
  fn code(self) -> &'static str {
    match self {
      Self::Indicative => "Ind",
      Self::Subjunctive => "Sub",
      Self::Imperative => "Imp",
      Self::NonFinite => "",
      Self::VerbalNoun => "",
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum VerbalNoun {
  GerGen,
  GerDat,
  GerAcc,
  GerAbl,
  SupAcc,
  SupAbl,
}
impl VerbalNoun {
  fn code(self) -> &'static str {
    match self {
      Self::GerGen => "GerGen",
      Self::GerDat => "GerDat",
      Self::GerAcc => "GerAcc",
      Self::GerAbl => "GerAbl",
      Self::SupAcc => "SupAcc",
      Self::SupAbl => "SupAbl",
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Voice {
  Active,
  Passive,
}
impl Voice {
  fn code(self) -> &'static str {
    match self {
      Self::Active => "Act",
      Self::Passive => "Pas",
    }
  }
}

trait TableContext: Sized + PartialEq + Copy {
  fn active(self) -> bool;
  fn passive(self) -> bool;
  fn singular(self) -> bool;
  fn plural(self) -> bool;
  fn indicative(self) -> bool;
  fn subjunctive(self) -> bool;
  fn imperative(self) -> bool;
  fn non_finite(self) -> bool;
  fn present(self) -> bool;
  fn imperfect(self) -> bool;
  fn future(self) -> bool;
  fn perfect(self) -> bool;
  fn pluperfect(self) -> bool;
  fn future2(self) -> bool;
  fn infinitive(self) -> bool;
  fn participle(self) -> bool;
  fn gerund(self) -> bool;
  fn supine(self) -> bool;
  fn genetive(self) -> bool;
  fn dative(self) -> bool;
  fn accusative(self) -> bool;
  fn ablative(self) -> bool;
  fn first(self) -> bool;
  fn second(self) -> bool;
  fn third(self) -> bool;
  fn max() -> Self;
  fn time(self) -> Option<Time> {
    if self.present() {
      Some(Time::Present)
    } else if self.imperfect() {
      Some(Time::Imperfect)
    } else if self.future() {Some(Time::Future)
    } else if self.perfect() {
      Some(Time::Perfect)
    } else if self.pluperfect() {
      Some(Time::Pluperfect)
    } else if self.future2() {
      Some(Time::FuturePerfect)
    } else {
      None
    }
  }
  fn tantum(self) -> Option<Tantum> {
    if self.singular() {
      Some(Tantum::Singular)
    } else if self.plural() {
      Some(Tantum::Plural)
    } else {
      None
    }
  }
  fn person(self) -> Option<Person> {
    if self.first() {
      Some(Person::First)
    } else if self.second() {
      Some(Person::Second)
    } else if self.third() {
      Some(Person::Third)
    } else {
      None
    }
  }
  fn category(self) -> Category {
    if self.indicative() {
      Category::Indicative
    } else if self.subjunctive() {
      Category::Subjunctive
    } else if self.imperative() {
      Category::Imperative
    } else if self.non_finite() {
      Category::NonFinite
    } else if self.verbal_noun() {
      Category::VerbalNoun
    } else {
      panic!("unknown category")
    }
  }
  fn noun(self) -> Option<VerbalNoun> {
    if self.gerund() {
      Some(if self.genetive() {
        VerbalNoun::GerGen
      } else if self.dative() {
        VerbalNoun::GerDat
      } else if self.accusative() {
        VerbalNoun::GerAcc
      } else if self.ablative() {
        VerbalNoun::GerAbl
      } else {
        panic!("uknown gerund")
      })
    } else if self.supine() {
      Some(if self.accusative() {
        VerbalNoun::SupAcc
      } else if self.ablative() {
        VerbalNoun::SupAbl
      } else {
        panic!("uknown supine")
      })
    } else {
      None
    }
  }
  fn voice(self) -> Option<Voice> {
    if self.active() {
      Some(Voice::Active)
    } else if self.passive() {
      Some(Voice::Passive)
    } else {
      None
    }
  }
  fn verbal_noun(self) -> bool {
    self.gerund() || self.supine()
  }
  fn code(self) -> String {
    format!(
      "{}{}{}{}{}{}",
      self.category().code(),
      self.time().map(|v| v.code()).unwrap_or_default(),
      self.voice().map(|v| v.code()).unwrap_or_default(),
      self.person().map(|v| v.code()).unwrap_or_default(),
      self.tantum().map(|v| v.code()).unwrap_or_default(),
      self.noun().map(|v| v.code()).unwrap_or_default(),
    )
  }
}

impl TableContext for usize {
  fn active(self) -> bool {
    self >= 138 && self <= 140 ||
      self >= 131 && self <= 133 ||
      self >= 122 && self <= 126 ||
      self >= 72 && self <= 95 ||
      self <= 35
  }
  fn passive(self) -> bool {
    self >= 141 && self <= 143 ||
      self >= 135 && self <= 137 ||
      self >= 127 && self <= 131 ||
      self >= 96 && self <= 120 ||
      self >= 36 && self <= 71
  }
  fn singular(self) -> bool {
    self <= 120 && self % 6 >= 0 && self % 6 <= 2 ||
      self == 121 || self == 123 || self == 124 || self == 127 || self == 129 || self == 130
  }
  fn plural(self) -> bool {
    self <= 120 && self % 6 >= 3 && self % 6 <= 5 ||
      self == 122 || self == 125 || self == 126 || self == 128 || self == 131
  }
  fn indicative(self) -> bool {
    self <= 71
  }
  fn subjunctive(self) -> bool {
    self >= 72 && self <= 120
  }
  fn imperative(self) -> bool {
    self >= 121 && self <= 128
  }
  fn non_finite(self) -> bool {
    self >= 132 && self <= 143
  }
  fn present(self) -> bool {
    self <= 5 ||
      self <= 41 && self >= 36 ||
      self <= 77 && self >= 72 ||
      self <= 101 && self >= 96 ||
      self == 121 || self == 122 || self == 127 || self == 128 ||
      self == 132 || self == 138 || self == 135 || self == 141
  }
  fn imperfect(self) -> bool {
    self <= 11 && self >= 6 ||
      self <= 47 && self >= 42 ||
      self <= 83 && self >= 78 ||
      self <= 107 && self >= 102
  }
  fn future(self) -> bool {
    self <= 17 && self >= 12 ||
      self <= 53 && self >= 48 ||
      self == 123 || self == 124 || self == 125 || self == 126 ||
      self == 129 || self == 130 || self == 131 ||
      self == 134 || self == 140 || self == 137 || self == 143
  }
  fn perfect(self) -> bool {
    self <= 23 && self >= 18 ||
      self <= 59 && self >= 54 ||
      self <= 89 && self >= 84 ||
      self <= 114 && self >= 109 ||
      self == 133 || self == 139 || self == 136 || self == 142
  }
  fn pluperfect(self) -> bool {
    self <= 29 && self >= 24 ||
      self <= 65 && self >= 60 ||
      self <= 95 && self >= 90 ||
      self <= 120 && self >= 115
  }
  fn future2(self) -> bool {
    self <= 35 && self >= 30 ||
      self <= 71 && self >= 66
  }
  fn infinitive(self) -> bool {
    self >= 121 && self <= 131
  }
  fn participle(self) -> bool {
    self >= 132 && self <= 143
  }
  fn gerund(self) -> bool {
    self >= 144 && self <= 147
  }
  fn supine(self) -> bool {
    self == 148 || self == 149
  }
  fn genetive(self) -> bool {
    self == 144
  }
  fn dative(self) -> bool {
    self == 145
  }
  fn accusative(self) -> bool {
    self == 146 || self == 148
  }
  fn ablative(self) -> bool {
    self == 147 || self == 149
  }
  fn first(self) -> bool {
    self <= 120 && self % 3 == 0
  }
  fn second(self) -> bool {
    self <= 120 && self % 3 == 1 ||
      self == 121 || self == 123 || self == 127 || self == 129 ||
      self == 122 || self == 125 || self == 128
  }
  fn third(self) -> bool {
    self <= 120 && self % 3 == 2 ||
      self == 124 || self == 130 || self == 126
  }
  fn max() -> Self {
    149
  }
}