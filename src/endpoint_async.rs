use crate::endpoint::Endpoint;
use crate::status::Status;
use crate::target::Target;
use async_io::{block_on, Timer};
use futures::future::{join_all, BoxFuture, FutureExt};
use futures::join;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub type OldStatus = Status;

pub enum Exec {
    OnUpdate,
    OnStatusChange,
}

pub struct EndpointAsync<'a> {
    endpoint: Endpoint,
    policy: Exec,
    callback: Box<dyn FnMut(Target, Status, OldStatus) + Send + 'a>,
    interval: Duration,
}

impl<'a> EndpointAsync<'a> {
    pub fn new(
        target: Target,
        policy: Exec,
        callback: impl FnMut(Target, Status, OldStatus) + Send + 'a,
        interval: Duration,
    ) -> Self {
        Self {
            endpoint: Endpoint::new(target),
            policy: policy,
            callback: Box::new(callback),
            interval: interval,
        }
    }
}

fn update_status<'a>(mut ep: EndpointAsync<'a>, shutdown: Arc<AtomicBool>) -> BoxFuture<'a, ()> {
    async move {
        // Early return if external atomic was set -> Stop recursion
        if shutdown.load(Ordering::Relaxed) {
            return;
        }

        // Update status, determine if callback should be executed based on Exec value
        let target = ep.endpoint.get_target();
        let old_status = ep.endpoint.get_status();
        let status = ep.endpoint.update_status();
        let exec_callback = match ep.policy {
            Exec::OnUpdate => true,
            Exec::OnStatusChange => {
                if old_status != status {
                    true
                } else {
                    false
                }
            }
        };

        // Perform async operations: Wait for timer and execute callback
        let async_timer = Timer::after(ep.interval);
        if exec_callback {
            let async_callback = async {
                (ep.callback)(target, status, old_status);
            };
            join!(async_timer, async_callback);
        } else {
            async_timer.await;
        }

        // Call recursivly and transfer ownership of all data.
        update_status(ep, shutdown).await;
    }
    .boxed()
}

pub struct Executor {
    worker: Option<(thread::JoinHandle<()>, Arc<AtomicBool>)>,
}

impl Executor {
    pub fn new() -> Self {
        Self { worker: None }
    }

    pub fn start_processing(&mut self, endpoints: Vec<EndpointAsync<'static>>) {
        if self.worker.is_none() {
            let shutdown = Arc::new(AtomicBool::new(false));

            // Spawn Async processing loop until shutdown is set to true
            let shutdown_clone = shutdown.clone();
            let handle = thread::spawn(move || {
                // Consume given endpoints and transform them to async
                // operations for parallel execution
                let futures: Vec<BoxFuture<'static, ()>> = endpoints
                    .into_iter()
                    .map(move |endpoint| update_status(endpoint, shutdown_clone.clone()))
                    .collect();

                block_on(join_all(futures));
            });
            self.worker = Some((handle, shutdown));
        }
    }

    pub fn stop_processing(&mut self) {
        if let Some((handle, shutdown)) = self.worker.take() {
            // Initate Teardown, stop processing and wait for spawned thread to terminate
            shutdown.store(true, Ordering::Relaxed);
            handle.join().unwrap();
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.stop_processing()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn endpoint_async_execute_on_update_3s() {
        // Execute async endpoint for 3s with an interval of 1s
        // Due to OnUpdate, the Callback should have been executed three times.
        let cnt = Arc::new(AtomicU64::new(0));
        let cnt_clone = Arc::clone(&cnt);

        let func = move |_t, _s, _ls| {
            cnt_clone.fetch_add(1, Ordering::Relaxed);
        };

        let ep = EndpointAsync::new(
            Target::Icmp("127.0.0.1".to_string()),
            Exec::OnUpdate,
            func,
            Duration::from_secs(1),
        );

        // Spawn async block in dedicated thread, sleep 3s and verify results
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        let handle = thread::spawn(move || {
            block_on(update_status(ep, shutdown_clone));
        });

        thread::sleep(Duration::from_secs(3));
        shutdown.store(true, Ordering::Relaxed);
        handle.join().unwrap();

        assert_eq!(cnt.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn endpoint_async_execute_on_status_change() {
        // Execute async endpoint for 3s with an interval of 1s
        // Due to OnStatusChange, the Callback should have been executed three times.
        let cnt = Arc::new(AtomicU64::new(0));
        let cnt_clone = Arc::clone(&cnt);

        let func = move |_t, _s, _ls| {
            cnt_clone.fetch_add(1, Ordering::Relaxed);
        };

        let ep = EndpointAsync::new(
            Target::Icmp("127.0.0.1".to_string()),
            Exec::OnStatusChange,
            func,
            Duration::from_secs(1),
        );

        // Spawn async block in dedicated thread, sleep 3s and verify results
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        let handle = thread::spawn(move || {
            block_on(update_status(ep, shutdown_clone));
        });

        thread::sleep(Duration::from_secs(3));
        shutdown.store(true, Ordering::Relaxed);
        handle.join().unwrap();

        assert_eq!(cnt.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn executor_start_stop_no_endpoint() {
        // Start and stop Executor without any Endpoints to see if this works
        let mut exec = Executor::new();
        exec.start_processing(vec![]);
        exec.stop_processing();
    }

    #[test]
    fn executor_start_stop_concurent_endpoints_3s() {
        // Spawn a number of endpoint and let them execute for three seconds
        // Each update increases a given count.endpoint
        // The test is successfull if the has the value of spawned_endpoints * runtime_sec
        let spawned_endpoints = 50;
        let runtime_sec = 3;
        let cnt = Arc::new(AtomicU64::new(0));

        let mut eps = vec![];
        for _ in 0..spawned_endpoints {
            let c = cnt.clone();
            let f = move |_t, _s, _ls| {
                c.fetch_add(1, Ordering::Relaxed);
            };

            let ep = EndpointAsync::new(
                Target::Icmp("127.0.0.1".to_string()),
                Exec::OnUpdate,
                f,
                Duration::from_secs(1),
            );

            eps.push(ep);
        }

        // Start and stop Executor without any Endpoints to see if this works
        let mut exec = Executor::new();
        exec.start_processing(eps);
        thread::sleep(Duration::from_secs(runtime_sec));
        exec.stop_processing();

        assert_eq!(cnt.load(Ordering::Relaxed), spawned_endpoints * runtime_sec);
    }
}
