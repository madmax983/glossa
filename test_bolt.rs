use glossa::semantic::GlossaType;
use glossa::codegen::to_rust_type;

fn main() {
    let mut ty = GlossaType::Number;
    for _ in 0..1000 {
        ty = GlossaType::List(Box::new(ty));
    }
    let start = std::time::Instant::now();
    let res = to_rust_type(&ty);
    println!("Elapsed: {:?}", start.elapsed());
    assert!(res.starts_with("Vec<"));
}
