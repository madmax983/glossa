use glossa::parser::parse;
use miette::GraphicalReportHandler;

fn main() {
    let source = "«χαῖρε»"; // Missing verb
    let result = parse(source);

    if let Err(e) = result {
        println!("--- MIETTE ERROR ---");
        let mut out = String::new();
        GraphicalReportHandler::new()
            .render_report(&mut out, &e)
            .unwrap();
        println!("{}", out);
        println!("--------------------");
    } else {
        println!("Unexpected success");
    }
}
