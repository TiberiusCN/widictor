use super::*;

impl Conjugation {
  pub fn gen_indicative4(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
    let gen_infect_end = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (voice, tantum, person, time) {
        (Voice::Active, Tantum::Singular, Person::First, Time::Present)     => vec!["iō"],
        (Voice::Active, Tantum::Singular, Person::First, Time::Imperfect)   => vec!["am"],
        (Voice::Active, Tantum::Singular, Person::First, Time::Future)      => vec!["am"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Present)    => vec!["īs"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Imperfect)  => vec!["ās"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Future)     => vec!["ēs"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Present)     => vec!["it"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Imperfect)   => vec!["at"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Future)      => vec!["et"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Present)       => vec!["īmus"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Imperfect)     => vec!["āmus"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Future)        => vec!["ēmus"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Present)      => vec!["ītis"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Imperfect)    => vec!["ātis"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Future)       => vec!["ētis"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Present)       => vec!["unt"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Imperfect)     => vec!["ant"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Future)        => vec!["ent"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Present)    => vec!["or"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Imperfect)  => vec!["ar"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Future)     => vec!["ar"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Present)   => vec!["īris","īre"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Imperfect) => vec!["āris","āre"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Future)    => vec!["ēris","ēre"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Present)    => vec!["ītur"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Imperfect)  => vec!["ātur"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Future)     => vec!["ētur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Present)      => vec!["īmur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Imperfect)    => vec!["āmur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Future)       => vec!["ēmur"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Present)     => vec!["īmini"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Imperfect)   => vec!["āmini"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Future)      => vec!["ēmini"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Present)      => vec!["untur"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Imperfect)    => vec!["antur"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Future)       => vec!["entur"],
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
        (Tantum::Singular, Person::Second, Time::FuturePerfect) => vec!["is"],
        (Tantum::Singular, Person::Third, Time::Perfect)        => vec!["it"],
        (Tantum::Singular, Person::Third, Time::Pluperfect)     => vec!["at"],
        (Tantum::Singular, Person::Third, Time::FuturePerfect)  => vec!["it"],
        (Tantum::Plural, Person::First, Time::Perfect)          => vec!["imus"],
        (Tantum::Plural, Person::First, Time::Pluperfect)       => vec!["āmus"],
        (Tantum::Plural, Person::First, Time::FuturePerfect)    => vec!["imus"],
        (Tantum::Plural, Person::Second, Time::Perfect)         => vec!["istis"],
        (Tantum::Plural, Person::Second, Time::Pluperfect)      => vec!["ātis"],
        (Tantum::Plural, Person::Second, Time::FuturePerfect)   => vec!["itis"],
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
        Time::Imperfect => "ēb",
        Time::Future => "",
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

  pub fn gen_subjunctive4(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
    let gen_infect_end = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (voice, tantum, person, time) {
        (Voice::Active, Tantum::Singular, Person::First, Time::Present)     => vec!["am"],
        (Voice::Active, Tantum::Singular, Person::First, Time::Imperfect)   => vec!["em"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Present)    => vec!["ās"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Imperfect)  => vec!["ēs"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Present)     => vec!["at"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Imperfect)   => vec!["et"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Present)       => vec!["āmus"],
        (Voice::Active, Tantum::Plural, Person::First, Time::Imperfect)     => vec!["ēmus"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Present)      => vec!["ātis"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Imperfect)    => vec!["ētis"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Present)       => vec!["ant"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Imperfect)     => vec!["ent"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Present)    => vec!["ar"],
        (Voice::Passive, Tantum::Singular, Person::First, Time::Imperfect)  => vec!["er"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Present)   => vec!["āris","āre"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Imperfect) => vec!["ēris","ēre"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Present)    => vec!["ātur"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Imperfect)  => vec!["ētur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Present)      => vec!["āmur"],
        (Voice::Passive, Tantum::Plural, Person::First, Time::Imperfect)    => vec!["ēmur"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Present)     => vec!["āmini"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Imperfect)   => vec!["ēmini"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Present)      => vec!["antur"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Imperfect)    => vec!["entur"],
        _ => unreachable!(),
      }
    };
    let gen_perfect_active_end = |tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (tantum, person, time) {
        (Tantum::Singular, Person::First, Time::Perfect)        => vec!["im"],
        (Tantum::Singular, Person::First, Time::Pluperfect)     => vec!["em"],
        (Tantum::Singular, Person::Second, Time::Perfect)       => vec!["īs"],
        (Tantum::Singular, Person::Second, Time::Pluperfect)    => vec!["ēs"],
        (Tantum::Singular, Person::Third, Time::Perfect)        => vec!["it"],
        (Tantum::Singular, Person::Third, Time::Pluperfect)     => vec!["et"],
        (Tantum::Plural, Person::First, Time::Perfect)          => vec!["īmus"],
        (Tantum::Plural, Person::First, Time::Pluperfect)       => vec!["ēmus"],
        (Tantum::Plural, Person::Second, Time::Perfect)         => vec!["ītis"],
        (Tantum::Plural, Person::Second, Time::Pluperfect)      => vec!["ētis"],
        (Tantum::Plural, Person::Third, Time::Perfect)          => vec!["int"],
        (Tantum::Plural, Person::Third, Time::Pluperfect)       => vec!["ent"],
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
        Time::Imperfect => "īr",
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

  pub fn gen_imperative4(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
    let gen_end = |voice: Voice, tantum: Tantum, person: Person, time: Time| -> Vec<&'static str> {
      match (voice, tantum, person, time) {
        (Voice::Active, Tantum::Singular, Person::Second, Time::Present)  => vec!["ī"],
        (Voice::Active, Tantum::Singular, Person::Second, Time::Future)   => vec!["īto"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Present) => vec!["īre"],
        (Voice::Passive, Tantum::Singular, Person::Second, Time::Future)  => vec!["ītor"],
        (Voice::Active, Tantum::Singular, Person::Third, Time::Future)    => vec!["ītō"],
        (Voice::Passive, Tantum::Singular, Person::Third, Time::Future)   => vec!["ītor"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Present)    => vec!["īte"],
        (Voice::Active, Tantum::Plural, Person::Second, Time::Future)     => vec!["ītōte"],
        (Voice::Passive, Tantum::Plural, Person::Second, Time::Present)   => vec!["īminī"],
        (Voice::Active, Tantum::Plural, Person::Third, Time::Future)      => vec!["iuntō"],
        (Voice::Passive, Tantum::Plural, Person::Third, Time::Future)     => vec!["iuntor"],
        _ => unreachable!(),
      }
    };
    let stem = self.infect_stem.as_slice();
    let end = gen_end(voice, tantum, person, time);
    Self::gen_x(stem, "", &end)
  }

  pub fn gen_infinitive4(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
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
      (Voice::Active, Time::Present)  => vec!["īre"],
      (Voice::Active, Time::Perfect)  => vec![""],
      (Voice::Active, Time::Future)   => vec!["um esse"],
      (Voice::Passive, Time::Present) => vec!["īrī"],
      (Voice::Passive, Time::Perfect) => vec!["um esse"],
      (Voice::Passive, Time::Future)  => vec!["um īrī"],
      _ => unreachable!(),
    };

    Self::gen_x(&stem, suffix, &end)
  }

  pub fn gen_participle4(&self, voice: Voice, tantum: Tantum, person: Person, time: Time) -> String {
    let stem = match (voice, time) {
      (Voice::Active, Time::Present)  => &self.infect_stem,
      (Voice::Active, Time::Future)   => &self.supine_stem,
      (Voice::Passive, Time::Perfect) => &self.supine_stem,
      (Voice::Passive, Time::Future)  => &self.infect_stem,
      _ => unreachable!(),
    };
    let suffix = match (voice, time) {
      (Voice::Active, Time::Present)  => "ēns",
      (Voice::Active, Time::Future)   => "ūr",
      (Voice::Passive, Time::Perfect) => "",
      (Voice::Passive, Time::Future)  => "end",
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

  pub fn gen_noun4(&self, noun: VerbalNoun) -> String {
    let stem = match noun {
      VerbalNoun::GerGen | VerbalNoun::GerDat | VerbalNoun::GerAcc | VerbalNoun::GerAbl => &self.infect_stem,
      VerbalNoun::SupAcc | VerbalNoun::SupAbl => &self.supine_stem,
    };
    let suffix = match noun {
      VerbalNoun::GerGen | VerbalNoun::GerDat | VerbalNoun::GerAcc | VerbalNoun::GerAbl => "end",
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
}
