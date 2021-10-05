use std::str::FromStr;

use reachable::*;

fn main() {
    // Construct ICMP Target and if its availability
    let icmp_target = IcmpTarget::from_str("www.google.de").unwrap();
    match icmp_target.check_availability() {
        Ok(status) => println!("{} is {}", icmp_target.get_id(), status),
        Err(error) => println!("Check failed for {} reason {}", icmp_target.get_id(), error),
    }
}
