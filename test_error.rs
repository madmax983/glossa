fn main() {
    let result = glossa::tools::labyrinth::run_labyrinth(std::path::Path::new("test.gl"));
    println!("{:?}", result);
}
