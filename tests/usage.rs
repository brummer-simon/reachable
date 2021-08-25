mod test {
    extern crate reachable;

    #[test]
    fn basic_usage() {
        // This test acts as a basic example. Check the availablity of the Endpoint "localhost"
        let target = reachable::Target::IcmpOnIpv6(String::from("::1"));
        let mut endpoint = reachable::Endpoint::new(target);

        // After the construction the Endpoint is in unknown state
        assert_eq!(endpoint.get_status(), reachable::Status::Unknown);

        // Refresh connection status: Since localhost should be always available, 
        // the given status must change to "Available"
        assert_eq!(endpoint.update_status(), reachable::Status::Available);
        assert_eq!(endpoint.get_status(), reachable::Status::Available);
    }
}
