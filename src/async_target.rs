use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::Duration;

use async_io::Timer;
use futures::executor::block_on;
use futures::future::{join_all, BoxFuture, FutureExt};

use crate::{CheckTargetError, Status, Target};

pub type OldStatus = Status;

pub struct AsyncTarget<'a> {
    target: Box<dyn Target + Send + 'a>,
    check_handler: Box<dyn FnMut(&dyn Target, Status, OldStatus, Option<CheckTargetError>) + Send + 'a>,
    check_interval: Duration,
    status: Status,
}

impl<'a> AsyncTarget<'a> {
    pub fn new(
        target: Box<dyn Target + Send + 'a>,
        check_handler: Box<dyn FnMut(&dyn Target, Status, OldStatus, Option<CheckTargetError>) + Send + 'a>,
        check_interval: Duration,
    ) -> Self {
        AsyncTarget {
            target: target,
            check_handler: check_handler,
            check_interval: check_interval,
            status: Status::Unknown,
        }
    }
}

impl<'a, T, U> From<(T, U, Duration)> for AsyncTarget<'a>
where
    T: Target + Send + 'a,
    U: FnMut(&dyn Target, Status, OldStatus, Option<CheckTargetError>) + Send + 'a,
{
    fn from(pieces: (T, U, Duration)) -> AsyncTarget<'a> {
        let (target, check_handler, check_interval) = pieces;
        AsyncTarget::new(Box::from(target), Box::from(check_handler), check_interval)
    }
}

pub struct AsyncTargetExecutor {
    worker: Option<(JoinHandle<()>, Arc<AtomicBool>)>,
}

impl AsyncTargetExecutor {
    pub fn new() -> Self {
        AsyncTargetExecutor {
            worker: None,
        }
    }

    pub fn start(&mut self, targets: Vec<AsyncTarget<'static>>) {
        if self.worker.is_none() {
            let shutdown_requested = Arc::new(AtomicBool::new(false));
            let shutdown_requested_clone = shutdown_requested.clone();

            // Convert all targets into BoxFutures and execute them afterwards
            let handle = spawn(move || {
                let tasks: Vec<BoxFuture<()>> = targets
                    .into_iter()
                    .map(|target| check_target(target, shutdown_requested_clone.clone()))
                    .collect();

                block_on(join_all(tasks));
            });

            self.worker = Some((handle, shutdown_requested));
        }
    }

    pub fn stop(&mut self) {
        if let Some((handle, shutdown_requested)) = self.worker.take() {
            shutdown_requested.store(true, Ordering::Relaxed);
            handle.join().unwrap();
        }
    }
}

impl Drop for AsyncTargetExecutor {
    fn drop(&mut self) {
        self.stop()
    }
}

pub fn check_target(mut async_target: AsyncTarget, shutdown_requested: Arc<AtomicBool>) -> BoxFuture<()> {
    async {
        // Stop recursive execution if shutdown_requested flag has been set.
        if shutdown_requested.load(Ordering::Relaxed) {
            return;
        }

        // Setup async timeout to wait for next check
        let timer = Timer::after(async_target.check_interval);

        // Note: async forces variables in scope, that outlive .await calls, to implement the Send Trait.
        // The Error trait does implement Send, therefore Option<CheckTargetError>
        // must not live longer then timer.await below. To limit the lifespan of error
        // we open a inner scope and close it before await
        {
            // Try to query current status. If this fails, store error and hand it to check_handler
            let (status, error) = match async_target.target.check_availability() {
                Ok(status) => (status, None),
                Err(error) => (Status::Unknown, Some(error)),
            };

            // Update stored status
            let old_status = async_target.status;
            async_target.status = status.clone();

            // Run timer and callable execution concurrently.
            async_target.check_handler.as_mut()(async_target.target.as_ref(), status, old_status, error);
        }
        // Wait until the timer expired and wait for next execution
        timer.await;
        check_target(async_target, shutdown_requested).await;
    }
    .boxed()
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use mockall::Sequence;

    use super::*;
    use crate::target::MockTarget;

    #[test]
    fn async_target_call_behavior() {
        // Expectency: This Test verifies the basic behavior as specified.
        // 1) First call: old_status is Status::Unknown, status depends on the result check_availability
        // 2) Next calls: old_status contains the status of the previous call and status contains
        //    the result of check_availability
        // 3) On Error: status contains Status::Unknown and error contains the occurred
        //    CheckTargetError

        // Prepare Mock
        let mut mock = MockTarget::new();
        let mut call_sequence = Sequence::new();

        // First call: return Status::Available
        mock.expect_check_availability()
            .times(1)
            .returning(|| Ok(Status::Available))
            .in_sequence(&mut call_sequence);

        // Second call: return Status::NotAvailable
        mock.expect_check_availability()
            .times(1)
            .returning(|| Ok(Status::NotAvailable))
            .in_sequence(&mut call_sequence);

        // Third call: return an Error
        mock.expect_check_availability()
            .times(1)
            .returning(|| Err(CheckTargetError::from("Error")))
            .in_sequence(&mut call_sequence);

        // Prepare Handler to verify given expectations
        let (send, recv) = mpsc::channel();
        let handler = move |_: &dyn Target, new: Status, old: OldStatus, error: Option<CheckTargetError>| {
            match old {
                // Verify expectency of the first call to check_availability
                Status::Unknown => {
                    assert_eq!(new, Status::Available);
                    assert_eq!(error.is_none(), true);
                }
                // Verify expectency of the second call to check_availability
                Status::Available => {
                    assert_eq!(new, Status::NotAvailable);
                    assert_eq!(error.is_none(), true);
                }
                // Verify expectency of the third call to check_availability. Stop handler.
                Status::NotAvailable => {
                    assert_eq!(new, Status::Unknown);
                    assert_eq!(error.is_some(), true);
                    let error = error.unwrap();
                    assert_eq!(format!("{}", error), "Error");
                    send.send(()).unwrap();
                }
            }
        };

        // Run test
        let mut exec = AsyncTargetExecutor::new();
        exec.start(vec![AsyncTarget::from((mock, handler, Duration::from_millis(100)))]);
        recv.recv().unwrap();
        exec.stop();
    }
}
