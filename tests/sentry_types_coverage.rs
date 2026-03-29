use glossa::morphology::models::Gender;
use glossa::semantic::GlossaType;

#[test]
fn test_glossa_type_to_greek() {
    assert_eq!(GlossaType::Number.to_greek(), "ἀριθμός");
    assert_eq!(GlossaType::String.to_greek(), "ὄνομα");
    assert_eq!(GlossaType::Boolean.to_greek(), "ἀληθές");
    assert_eq!(
        GlossaType::List(Box::new(GlossaType::Number)).to_greek(),
        "λίστη"
    );
    assert_eq!(
        GlossaType::Set(Box::new(GlossaType::Number)).to_greek(),
        "σύνολον"
    );
    assert_eq!(
        GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)).to_greek(),
        "χάρτης"
    );
    assert_eq!(
        GlossaType::Option(Box::new(GlossaType::Number)).to_greek(),
        "εὑρεθείη"
    );
    assert_eq!(
        GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String)).to_greek(),
        "ἀποτέλεσμα"
    );
    assert_eq!(GlossaType::Unit.to_greek(), "οὐδέν");
    assert_eq!(
        GlossaType::Struct {
            name: "Test".into(),
            fields: vec![],
            gender: Gender::Neuter
        }
        .to_greek(),
        "εἶδος"
    );
    assert_eq!(
        GlossaType::Function {
            params: vec![],
            returns: Box::new(GlossaType::Unit)
        }
        .to_greek(),
        "ἔργον"
    );
    assert_eq!(GlossaType::Unknown.to_greek(), "ἄγνωστον");
}

#[test]
fn test_glossa_type_is_compatible() {
    let list_num = GlossaType::List(Box::new(GlossaType::Number));
    let list_str = GlossaType::List(Box::new(GlossaType::String));
    let set_num = GlossaType::Set(Box::new(GlossaType::Number));
    let set_str = GlossaType::Set(Box::new(GlossaType::String));
    let map_num_str = GlossaType::Map(Box::new(GlossaType::Number), Box::new(GlossaType::String));
    let map_num_num = GlossaType::Map(Box::new(GlossaType::Number), Box::new(GlossaType::Number));
    let map_str_str = GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::String));

    // Lists
    assert!(list_num.is_compatible(&list_num));
    assert!(!list_num.is_compatible(&list_str));

    // Sets
    assert!(set_num.is_compatible(&set_num));
    assert!(!set_num.is_compatible(&set_str));

    // Maps
    assert!(map_num_str.is_compatible(&map_num_str));
    assert!(!map_num_str.is_compatible(&map_num_num)); // Different value type
    assert!(!map_num_str.is_compatible(&map_str_str)); // Different key type

    // Options
    let opt_num = GlossaType::Option(Box::new(GlossaType::Number));
    let opt_str = GlossaType::Option(Box::new(GlossaType::String));
    assert!(opt_num.is_compatible(&opt_num));
    assert!(!opt_num.is_compatible(&opt_str));

    // Results
    let res_num_str =
        GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String));
    let res_str_num =
        GlossaType::Result(Box::new(GlossaType::String), Box::new(GlossaType::Number));
    assert!(res_num_str.is_compatible(&res_num_str));
    assert!(!res_num_str.is_compatible(&res_str_num));
}
