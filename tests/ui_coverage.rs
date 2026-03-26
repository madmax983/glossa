use glossa::tools::ui::Status;
use std::thread;
use std::time::Duration;

#[test]
fn test_ui_spinner_drains_channel_and_cleans_up() {
    let mut status = Status::start("Connecting");
    status.update("Downloading");
    status.update("Analyzing");
    status.update("Extracting");

    // Give thread time to wake up and process updates
    thread::sleep(Duration::from_millis(250));

    status.success();
}
