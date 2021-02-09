#[derive(Debug, Clone)]
enum Piece {
  Raw(String),
  Template(PieceParams),
}

impl Piece {
  fn text(&self, prefix: &str, lemma: &mut Lemma, suffix: &str, section: &SectionSpecies, wiki: &Wiki) {
    match self {
      Self::Raw(raw) => if !raw.is_empty() { lemma.append_value(prefix, raw, suffix); },
      Self::Template(map) => {
        if let Some(template_wrapper) = TEMPLATES.get(&map.com) {
          let mut template = Params {
            section: *section,
            com: map.com.clone(),
            args: HashMap::new(),
          };

          for (arg, line) in &map.args {
            let mut lemma = Lemma::default();
            for val in line {
              val.text("", &mut lemma, "", section, wiki);
            }
            template.args.insert(arg.clone(), lemma.value.unwrap_or_default());
          };

          let mut map = template;
          map.section = *section;
          let mut com = match std::process::Command::new(template_wrapper)
            .env("ENV_MAINWORD", &wiki.word)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
          {
            Err(e) => {
              eprint!("template {} failed: {}", map.com, e);
              return;
            },
            Ok(v) => v,
          };
          let stdin = com.stdin.take().unwrap();
          let stdout = com.stdout.take().unwrap();
          let mut stderr = com.stderr.take().unwrap();
          serde_json::to_writer(stdin, &map).unwrap();
          let com = com.wait().unwrap();
          let mut err = String::new();
          let _ = stderr.read_to_string(&mut err).map_err(|e| eprintln!("bad stderr: {}", e));
          if !err.is_empty() { eprint!("template {}: {}", map.com, err); }
          if !com.success() { eprintln!("template {} failed with {:?}", map.com, com.code()); return; }

          if let Ok(new_lemma) = serde_json::from_reader(stdout).map_err(|e| eprintln!("bad json from {}: {}", map.com, e)) {
            *lemma += new_lemma;
          }
        } else {
          eprintln!("unknown template: {}", map.com);
        }
      },
    }
  }
}
