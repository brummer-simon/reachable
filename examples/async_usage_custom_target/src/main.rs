// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Author: Simon Brummer (simon.brummer@posteo.de)

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
