use crate::status::Status;
use crate::target::Target;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::process::{Command, Stdio};

pub struct Endpoint {
    target: Target,
    status: Status,
}

impl Endpoint {
    pub fn new(target: Target) -> Endpoint {
        Self {
            target: target,
            status: Status::Unknown,
        }
    }

    pub fn update_status(&mut self) -> Status {
        let any_pred = |ip: IpAddr| {
            match self.target {
                // ICMP: Use ping instead of selfmade ICMP packages. That would require root rights.
                Target::Icmp(_) | Target::IcmpOnIpv4(_) | Target::IcmpOnIpv6(_) => {
                    let status = if ip.is_ipv6() {
                        Command::new("ping")
                            .stdout(Stdio::null())
                            .arg("-c 1")
                            .arg("-6")
                            .arg(ip.to_string())
                            .status()
                            .unwrap()
                    } else {
                        Command::new("ping")
                            .stdout(Stdio::null())
                            .arg("-c 1")
                            .arg(ip.to_string())
                            .status()
                            .unwrap()
                    };

                    if status.success() {
                        true
                    } else {
                        false
                    }
                }

                // TCP: Try to open a socket to remote target
                Target::Tcp(_, port) | Target::TcpOnIpv4(_, port) | Target::TcpOnIpv6(_, port) => {
                    match TcpStream::connect(SocketAddr::from((ip, port))) {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                }
            }
        };

        // Resolve target check any of the results can be used to establish a connection.
        if let Some(ips) = self.target.resolve() {
            if ips.into_iter().any(any_pred) {
                self.status = Status::Available
            } else {
                self.status = Status::NotAvailable
            }
        } else {
            self.status = Status::Unknown
        }

        // Return a clone of the updated status
        self.status.clone()
    }

    pub fn get_status(&self) -> Status {
        self.status.clone()
    }

    pub fn get_target(&self) -> Target {
        self.target.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread::{sleep, spawn};
    use std::time::Duration;

    #[test]
    fn icmp_endpoint_is_available() {
        let target = Target::Icmp("127.0.0.1".to_string());
        let mut endpoint = Endpoint::new(target);

        assert_eq!(endpoint.get_status(), Status::Unknown);
        assert_eq!(endpoint.update_status(), Status::Available);
        assert_eq!(endpoint.get_status(), Status::Available);
    }

    #[test]
    fn icmp_on_ipv4_endpoint_is_available() {
        let target = Target::IcmpOnIpv4("127.0.0.1".to_string());
        let mut endpoint = Endpoint::new(target);

        assert_eq!(endpoint.get_status(), Status::Unknown);
        assert_eq!(endpoint.update_status(), Status::Available);
        assert_eq!(endpoint.get_status(), Status::Available);
    }

    #[test]
    fn icmp_on_ipv6_endpoint_is_available() {
        let target = Target::IcmpOnIpv6("::1".to_string());
        let mut endpoint = Endpoint::new(target);

        assert_eq!(endpoint.get_status(), Status::Unknown);
        assert_eq!(endpoint.update_status(), Status::Available);
        assert_eq!(endpoint.get_status(), Status::Available);
    }

    #[test]
    fn tcp_endpoint_is_available() {
        // Spawn TCP server and wait for incoming connection
        let srv = || {
            let listener = TcpListener::bind("127.0.0.1:24211").unwrap();
            let _stream = listener.accept();
        };

        let srv_handle = spawn(srv);
        sleep(Duration::from_millis(500));

        // Connect to local TCP connection
        let target = Target::Tcp("127.0.0.1".to_string(), 24211);
        let mut endpoint = Endpoint::new(target);
        assert_eq!(endpoint.get_status(), Status::Unknown);
        assert_eq!(endpoint.update_status(), Status::Available);
        assert_eq!(endpoint.get_status(), Status::Available);

        // Join spawned thread
        srv_handle.join().unwrap();
    }

    #[test]
    fn tcp_on_ipv6_endpoint_is_available() {
        // Spawn TCP server and wait for incoming connection
        let srv = || {
            let listener = TcpListener::bind("[::1]:24212").unwrap();
            let _stream = listener.accept();
        };

        let srv_handle = spawn(srv);
        sleep(Duration::from_millis(500));

        // Connect to local TCP connection
        let target = Target::TcpOnIpv6("::1".to_string(), 24212);
        let mut endpoint = Endpoint::new(target);
        assert_eq!(endpoint.get_status(), Status::Unknown);
        assert_eq!(endpoint.update_status(), Status::Available);
        assert_eq!(endpoint.get_status(), Status::Available);

        // Join spawned thread
        srv_handle.join().unwrap();
    }

    #[test]
    fn tcp_on_ipv4_endpoint_is_available() {
        // Spawn TCP server and wait for incoming connection
        let srv = || {
            let listener = TcpListener::bind("127.0.0.1:24213").unwrap();
            let _stream = listener.accept();
        };

        let srv_handle = spawn(srv);
        sleep(Duration::from_millis(500));

        // Connect to local TCP connection
        let target = Target::TcpOnIpv4("127.0.0.1".to_string(), 24213);
        let mut endpoint = Endpoint::new(target);
        assert_eq!(endpoint.get_status(), Status::Unknown);
        assert_eq!(endpoint.update_status(), Status::Available);
        assert_eq!(endpoint.get_status(), Status::Available);

        // Join spawned thread
        srv_handle.join().unwrap();
    }
}
