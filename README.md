# reachable

Rust crate to check if a "Target" is available. The crate comes with the trait
"Target" and ICMP/TCP based implementations of it. Additionally, the crate offers
an async task executor to perform availability checks of "Targets" on a regular basis.

## Usage

With this crate you can easily check if a computer is currently reachable over the network.
Since all targets are implementations of trait "Target" the availability check behavior is customizable.
For example, it is easy to implement a custom Target to check if a Process is
running or not.

## Example (from examples/usage/src/main.rs)

```rust
use std::str::FromStr;

use reachable::*;

fn main() {
    // Construct ICMP Target check if the target is availabile
    let icmp_target = IcmpTarget::from_str("www.google.de").unwrap();
    match icmp_target.check_availability() {
        Ok(status) => println!("{} is {}", icmp_target.get_id(), status),
        Err(error) => println!("Check failed for {} reason {}", icmp_target.get_id(), error),
    }
}
```

## Async Example (from examples/async_usage/src/main.rs)

```rust
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

    // Spawn Async execution
    let mut exec = AsyncTargetExecutor::new();
    exec.start(vec![
        AsyncTarget::from((icmp_target, handler, Duration::from_secs(1))),
        AsyncTarget::from((tcp_target, handler, Duration::from_secs(1))),
    ]);
    sleep(Duration::from_secs(3));
    exec.stop();
}
```
