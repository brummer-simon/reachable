use dns_lookup::lookup_host;
use std::net::IpAddr;

#[derive(Clone)]
pub enum Target {
    Icmp(String),
    IcmpOnIpv4(String),
    IcmpOnIpv6(String),
    Tcp(String, u16),
    TcpOnIpv4(String, u16),
    TcpOnIpv6(String, u16),
}

impl Target {
    pub fn resolve(&self) -> Option<Vec<IpAddr>> {
        let mut ips = match lookup_host(self.get_fqhn()) {
            Ok(ips) => ips,
            Err(_) => return None,
        };

        ips = match &self {
            Target::IcmpOnIpv4(_) | Target::TcpOnIpv4(_, _) => {
                ips.into_iter().filter(|ip| ip.is_ipv4()).collect()
            }

            Target::IcmpOnIpv6(_) | Target::TcpOnIpv6(_, _) => {
                ips.into_iter().filter(|ip| ip.is_ipv6()).collect()
            }

            _ => ips,
        };

        if !ips.is_empty() {
            Some(ips)
        } else {
            None
        }
    }

    pub fn get_fqhn(&self) -> &String {
        match self {
            Target::Icmp(fqhn)
            | Target::IcmpOnIpv4(fqhn)
            | Target::IcmpOnIpv6(fqhn)
            | Target::Tcp(fqhn, _)
            | Target::TcpOnIpv4(fqhn, _)
            | Target::TcpOnIpv6(fqhn, _) => fqhn,
        }
    }

    pub fn get_port(&self) -> Option<u16> {
        match self {
            Target::Tcp(_, port) | Target::TcpOnIpv4(_, port) | Target::TcpOnIpv6(_, port) => {
                Some(*port)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn icmp_resolve_localhost() {
        assert_eq!(
            Target::Icmp("127.0.0.1".to_string()).resolve(),
            Some(vec![IpAddr::V4(Ipv4Addr::LOCALHOST)])
        );
    }

    #[test]
    fn icmp_resolve_existing_url() {
        let opt = Target::Icmp("www.ietf.com".to_string()).resolve();
        assert_eq!(opt.is_some(), true);
    }

    #[test]
    fn icmp_resolve_non_existing_url() {
        assert_eq!(
            Target::Icmp("saldkalskdj.foobar".to_string()).resolve(),
            None
        );
    }

    #[test]
    fn icmp_on_ipv4_resolve_v4_localhost() {
        assert_eq!(
            Target::IcmpOnIpv4("127.0.0.1".to_string()).resolve(),
            Some(vec![IpAddr::V4(Ipv4Addr::LOCALHOST)])
        );
    }

    #[test]
    fn icmp_on_ipv4_resolve_v6_localhost() {
        assert_eq!(Target::IcmpOnIpv4("::1".to_string()).resolve(), None);
    }

    #[test]
    fn icmp_on_ipv6_resolve_v6_localhost() {
        assert_eq!(
            Target::IcmpOnIpv6("::1".to_string()).resolve(),
            Some(vec![IpAddr::V6(Ipv6Addr::LOCALHOST)])
        );
    }

    #[test]
    fn icmp_on_ipv6_resolve_v4_localhost() {
        assert_eq!(Target::IcmpOnIpv6("127.0.0.1".to_string()).resolve(), None);
    }

    #[test]
    fn tcp_resolve_localhost() {
        assert_eq!(
            Target::Tcp("127.0.0.1".to_string(), 80).resolve(),
            Some(vec![IpAddr::V4(Ipv4Addr::LOCALHOST)])
        );
    }

    #[test]
    fn tcp_resolve_existing_url() {
        let opt = Target::Tcp("www.ietf.com".to_string(), 80).resolve();
        assert_eq!(opt.is_some(), true);
    }

    #[test]
    fn tcp_resolve_non_existing_url() {
        assert_eq!(
            Target::Tcp("saldkalskdj.foobar".to_string(), 80).resolve(),
            None
        );
    }

    #[test]
    fn tcp_on_ipv4_resolve_v4_localhost() {
        assert_eq!(
            Target::TcpOnIpv4("127.0.0.1".to_string(), 80).resolve(),
            Some(vec![IpAddr::V4(Ipv4Addr::LOCALHOST)])
        );
    }

    #[test]
    fn tcp_on_ipv4_resolve_v6_localhost() {
        assert_eq!(Target::TcpOnIpv4("::1".to_string(), 80).resolve(), None);
    }

    #[test]
    fn tcp_on_ipv6_resolve_v6_localhost() {
        assert_eq!(
            Target::TcpOnIpv6("::1".to_string(), 80).resolve(),
            Some(vec![IpAddr::V6(Ipv6Addr::LOCALHOST)])
        );
    }

    #[test]
    fn tcp_on_ipv6_resolve_v4_localhost() {
        assert_eq!(
            Target::TcpOnIpv6("127.0.0.1".to_string(), 80).resolve(),
            None
        );
    }
}
