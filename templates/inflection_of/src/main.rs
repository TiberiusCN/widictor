use template::*;

fn main() {
  let mut params: Params = serde_json::from_reader(std::io::stdin()).unwrap();
  serde_json::to_writer(std::io::stdout(), &Word {
    mutation: Default::default(),
    subwords: vec![params.args.remove("2").unwrap()[0].to_owned()],
    tags: Default::default(),
    value: Default::default(),
  }).unwrap()
}
