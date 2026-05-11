cat src/tools/gnomon.rs | sed '/\/\/\/ let input = Path::new("algorithm.γλ");/,/```/c\
/// let input = Path::new("algorithm.γλ");\
/// if let Err(e) = run_gnomon(\&input) {\
///     eprintln!("Failed to estimate complexity: {}", e);\
/// }\
/// ```\
pub fn run_gnomon(input: &Path) -> Result<()> {' > src/tools/gnomon.rs.new
mv src/tools/gnomon.rs.new src/tools/gnomon.rs
