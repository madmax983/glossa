use glossa::tools::{Cache, Status};

#[test]
fn test_tools_facade_coverage() {
    let _cache = Cache::new();
    // Use Status to ignore warning
    let status = Status::start("Test");
    status.success();
}
