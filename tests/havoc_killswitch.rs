#![allow(missing_docs)]
use std::sync::{Arc, Mutex};
use std::thread;

// A concurrency test verifying heavy multi-threaded usage of the parser/analyzer
// Does not deadlock or panic
#[test]
fn test_multithreaded_analyze_source() {
    let source = "«test» λέγε.";
    let mut handles = vec![];
    // Create a shared mutex just to verify standard library locks compile and run around it
    let lock = Arc::new(Mutex::new(0));

    for _ in 0..100 {
        let lock_clone = Arc::clone(&lock);
        let handle = thread::spawn(move || {
            let mut guard = lock_clone.lock().unwrap();
            *guard += 1;
            drop(guard);

            let _ = glossa::tools::runner::analyze_source(source);
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    assert_eq!(*lock.lock().unwrap(), 100);
}
