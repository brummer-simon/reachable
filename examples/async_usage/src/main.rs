// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Author: Simon Brummer (simon.brummer@posteo.de)

use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use reachable::*;

fn main() {
    // Setup AsyncTargets
    let icmp_target = IcmpTarget::from_str("www.google.de").unwrap();
    let tcp_target = TcpTarget::from_str("www.google.de:80").unwrap();

    let handler = |target: &dyn Target, status, old_status, error| {
        print!("Target \"{}\"", target.get_id());
        print!(", old status \"{}\"", old_status);
        print!(", new status \"{}\"", status);
        match error {
            None => println!(""),
            Some(err) => println!(", Error: \"{}\"", err),
        }
    };

    // Spawn Async executor
    let mut exec = AsyncTargetExecutor::new();
    exec.start(vec![
        AsyncTarget::from((icmp_target, handler, Duration::from_secs(1))),
        AsyncTarget::from((tcp_target, handler, Duration::from_secs(1))),
    ]);
    sleep(Duration::from_secs(3));
    exec.stop();
}
