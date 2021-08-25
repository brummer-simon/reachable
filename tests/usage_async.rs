#[cfg(feature = "async")]
mod test {
    extern crate reachable;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn basic_usage_async() {
        // This test acts as a basic example. Check the availablity of the Endpoint "localhost" iteratively.
        // Details: This test constructs two async Endpoints each registering a function to execute
        // after the a second. One async Endpoint should be executed with each update, one only on
        // change of the connection status.
        let target = reachable::Target::Icmp(String::from("::1"));
        let check_interval = Duration::from_secs(1);
        let counter_on_update = Arc::new(AtomicU32::new(0));
        let counter_on_status_change = Arc::new(AtomicU32::new(0));

        // Function to be executed on status update
        let counter_on_update_clone = counter_on_update.clone();
        let count_on_update = move |_target, _status, _old_status| {
            counter_on_update_clone.fetch_add(1, Ordering::Relaxed);
        };

        // Function to be executed on status change
        let counter_on_status_change_clone = counter_on_status_change.clone();
        let count_on_status_change = move |_target, _status, _old_status| {
            counter_on_status_change_clone.fetch_add(1, Ordering::Relaxed);
        };

        // Setup Async Endpoints
        let endpoint_count_on_update = reachable::EndpointAsync::new(
            target.clone(),
            reachable::Exec::OnUpdate,
            count_on_update,
            check_interval.clone(),
        );

        let endpoint_count_on_status_change = reachable::EndpointAsync::new(
            target,
            reachable::Exec::OnStatusChange,
            count_on_status_change,
            check_interval,
        );

        // Setup executor let it process for 3 seconds
        let mut executor = reachable::Executor::new();
        executor.start_processing(vec![
            endpoint_count_on_update,
            endpoint_count_on_status_change,
        ]);
        sleep(Duration::from_secs(3));
        executor.stop_processing();

        // Check counter values to verify if the given Function were executed.
        // Assumption:
        // 1) counter_on_update has a value of 3 (incremented each second)
        // 2) counter_on_status_change has a value of 1 (only one status change occured)
        assert_eq!(counter_on_update.load(Ordering::Relaxed), 3);
        assert_eq!(counter_on_status_change.load(Ordering::Relaxed), 1);
    }
}
