#[derive(Debug, Clone)]
struct PieceParams {
  section: SectionSpecies,
  com: String,
  args: HashMap<String, Vec<Piece>>,
}
