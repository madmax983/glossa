use glossa::tools::runner::run_file;
use std::fs::File;
use std::io::Write;

#[test]
fn test_compiler_crash_collision_structs() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("crash_structs.gl");

    // Program:
    // εἶδος a ὁρίζειν { }. (Struct 'A')
    // εἶδος α ὁρίζειν { }. (Struct 'Alpha' -> 'A')
    //
    // Collision: `struct G_A {}` and `struct G_A {}`
    // Rustc Error: the name `G_A` is defined multiple times

    let source = r#"
    εἶδος a ὁρίζειν { }.
    εἶδος α ὁρίζειν { }.
    "#;

    {
        let mut f = File::create(&input_path).unwrap();
        f.write_all(source.as_bytes()).unwrap();
    }

    let result = run_file(&input_path);

    // We expect this to SUCCEED now that collision is fixed
    if let Err(e) = result {
        panic!("Compiler crashed: {}", e);
    }
}
