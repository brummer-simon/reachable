use std::thread::sleep;
use std::time::Duration;

use reachable::*;

struct MyTarget;

// Implement trait Target for custom type
impl Target for MyTarget {
    fn get_id(&self) -> String {
        String::from("MyTarget")
    }

    fn check_availability(&self) -> Result<Status, CheckTargetError> {
        Ok(Status::Available)
    }
}

#[cfg(feature = "async")]
fn main() {
    // Setup custom Target for async availability check execution
    let my_target = MyTarget {};

    let handler = |target: &dyn Target, status, old_status, _error| {
        print!("Target \"{}\"", target.get_id());
        print!(", old status \"{}\"", old_status);
        println!(", new status \"{}\"", status);
    };

    let targets = vec![AsyncTarget::from((my_target, handler, Duration::from_secs(1)))];

    // Spawn async executor
    let mut exec = AsyncTargetExecutor::new();
    exec.start(targets);
    sleep(Duration::from_secs(3));
    exec.stop();
}
