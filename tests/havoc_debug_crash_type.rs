#![allow(missing_docs)]
use glossa::semantic::GlossaType;
use glossa::morphology::Gender;

#[test]
fn test_glossa_type_debug_coverage() {
    let t_num = GlossaType::Number;
    let t_str = GlossaType::String;
    let t_bool = GlossaType::Boolean;
    let t_list = GlossaType::List(Box::new(GlossaType::Number));
    let t_set = GlossaType::Set(Box::new(GlossaType::Number));
    let t_map = GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number));
    let t_opt = GlossaType::Option(Box::new(GlossaType::Number));
    let t_res = GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String));
    let t_struct = GlossaType::Struct {
        name: "TestStruct".into(),
        gender: Gender::Neuter,
        fields: vec![("field1".into(), GlossaType::Number)],
    };
    let t_func = GlossaType::Function {
        params: vec![GlossaType::Number],
        returns: Box::new(GlossaType::Boolean),
    };
    let t_unit = GlossaType::Unit;
    let t_unknown = GlossaType::Unknown;

    assert_eq!(format!("{:?}", t_num), "Number");
    assert_eq!(format!("{:?}", t_str), "String");
    assert_eq!(format!("{:?}", t_bool), "Boolean");
    assert_eq!(format!("{:?}", t_list), "List(Number)");
    assert_eq!(format!("{:?}", t_set), "Set(Number)");
    assert_eq!(format!("{:?}", t_map), "Map(String, Number)");
    assert_eq!(format!("{:?}", t_opt), "Option(Number)");
    assert_eq!(format!("{:?}", t_res), "Result(Number, String)");

    let struct_str = format!("{:?}", t_struct);
    assert!(struct_str.contains("TestStruct"));
    assert!(struct_str.contains("Neuter"));

    let func_str = format!("{:?}", t_func);
    assert!(func_str.contains("Number"));
    assert!(func_str.contains("Boolean"));

    assert_eq!(format!("{:?}", t_unit), "Unit");
    assert_eq!(format!("{:?}", t_unknown), "Unknown");
}

use std::env;
use std::process::Command;

#[test]
fn havoc_crash_debug_stack_overflow_glossa_type() {
    if env::var("HAVOC_DETONATE_DEBUG_GLOSSA_TYPE").is_ok() {
        let depth = 50_000;

        let mut deep_type = GlossaType::Number;
        for _ in 0..depth {
            deep_type = GlossaType::Function {
                params: vec![deep_type],
                returns: Box::new(GlossaType::Number),
            };
        }

        println!("Formatting deep expression (depth {})...", depth);
        let _s = format!("{:?}", deep_type);

        println!("Survived and mitigated!");
        std::process::exit(0);
    }

    let exe = env::current_exe().expect("Failed to get current executable");

    let status = Command::new(exe)
        .env("HAVOC_DETONATE_DEBUG_GLOSSA_TYPE", "1")
        .arg("--nocapture")
        .arg("havoc_crash_debug_stack_overflow_glossa_type")
        .status()
        .expect("Failed to spawn subprocess");

    assert!(status.success(), "Subprocess should not have crashed!");
}
