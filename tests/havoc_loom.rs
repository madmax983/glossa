use loom::thread;
use glossa::morphology::lexicon::{lookup, is_verb};
use glossa::morphology::participle::analyze_participle;

#[test]
fn test_concurrent_lexicon() {
    loom::model(|| {
        let t1 = thread::spawn(|| {
            lookup("λεγε");
        });
        let t2 = thread::spawn(|| {
            is_verb("λεγε");
        });
        let t3 = thread::spawn(|| {
            analyze_participle("γραφων");
        });

        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
    });
}
