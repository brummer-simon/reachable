use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use reachable::*;

#[cfg(feature = "async")]
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

    let targets = vec![
        AsyncTarget::from((icmp_target, handler, Duration::from_secs(1))),
        AsyncTarget::from((tcp_target, handler, Duration::from_secs(1))),
    ];

    // Spawn Async executor
    let mut exec = AsyncTargetExecutor::new();
    exec.start(targets);
    sleep(Duration::from_secs(3));
    exec.stop();
}
