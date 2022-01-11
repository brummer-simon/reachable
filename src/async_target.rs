// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Author: Simon Brummer (simon.brummer@posteo.de)

//! Module contains utilities for asynchronous, iterative "Target" reachability checking.
//!
//! # Notes
//! Requires crate to be configured with feature "async".

use super::{CheckTargetError, Status, Target};
use futures::future::{join, join_all, BoxFuture, FutureExt};
use std::thread::{spawn, JoinHandle};
use std::time::Duration;
use tokio::runtime::{self};
use tokio::select;
use tokio::sync::watch::{self, Receiver, Sender};
use tokio::task::{self};
use tokio::time::{self};

/// Alias on [Status] to distinct between status of previous availability
/// check and the current availability check
pub type OldStatus = Status;

/// Type for a boxed trait object implementing [Target]
pub type BoxedTarget<'a> = Box<dyn Target + Send + 'a>;

/// Type containing a boxed trait object implementing [FnMut] that is called with each async check.
pub type BoxedHandler<'a> = Box<dyn FnMut(&dyn Target, Status, OldStatus, Option<CheckTargetError>) + Send + 'a>;

/// Struct storing all data used during asynchronous execution.
///
/// For async check execution, wrap the instances of [Target] in [AsyncTarget] and hand them to
/// [AsyncTargetExecutor::start].
pub struct AsyncTarget<'a> {
    target: BoxedTarget<'a>,
    check_handler: BoxedHandler<'a>,
    check_interval: Duration,
    status: Status,
}

impl<'a> AsyncTarget<'a> {
    /// Construct an [AsyncTarget]. For more convenience use [AsyncTarget::from] instead.
    ///
    /// # Arguments
    /// * target: trait object implementing [Target] to use in periodic checks.
    /// * check_handler: Function to call with the results of [Target::check_availability].
    /// * check_interval: time [Duration] between periodic availability checks.
    ///
    /// # Returns
    /// Instance of [AsyncTarget].
    pub fn new(target: BoxedTarget<'a>, check_handler: BoxedHandler<'a>, check_interval: Duration) -> Self {
        AsyncTarget {
            target,
            check_handler,
            check_interval,
            status: Status::Unknown,
        }
    }
}

impl<'a, T, U> From<(T, U, Duration)> for AsyncTarget<'a>
where
    T: Target + Send + 'a,
    U: FnMut(&dyn Target, Status, OldStatus, Option<CheckTargetError>) + Send + 'a,
{
    /// Build a [AsyncTarget] from a Target, a function to be executed with the results of
    /// an availability check and a time interval an availability check occurs.
    ///
    /// # Example
    /// See Example in [AsyncTargetExecutor::start]
    fn from(pieces: (T, U, Duration)) -> AsyncTarget<'a> {
        let (target, check_handler, check_interval) = pieces;
        AsyncTarget::new(Box::from(target), Box::from(check_handler), check_interval)
    }
}

/// Async target check executor used to check the availability of a given number of [AsyncTarget]s.
pub struct AsyncTargetExecutor {
    /// Optional threadhandle and synchronization channel to executing runtime.
    worker: Option<(JoinHandle<()>, Sender<()>)>,
}

impl AsyncTargetExecutor {
    /// Construct a new [AsyncTargetExecutor]
    pub fn new() -> Self {
        AsyncTargetExecutor {
            worker: None,
        }
    }

    /// Start periodic availability checks for all given targets
    ///
    /// Each targets execution behavior is configured during [AsyncTarget] construction.
    ///
    /// # Arguments
    /// * targets: a vector of [AsyncTarget]s, those availability should be check periodically.
    ///
    /// # Example
    /// ```
    /// # use std::{str::FromStr, thread::sleep, time::Duration};
    /// # use reachable::*;
    ///
    /// // Setup AsyncTarget
    /// let target = IcmpTarget::from_str("127.0.0.1").unwrap();
    /// let check_handler = |_: &dyn Target, _: Status, _: OldStatus, _: Option<CheckTargetError>| {
    ///    // Handle check results
    /// };
    /// let check_interval = Duration::from_secs(1);
    /// let async_target = AsyncTarget::from((target, check_handler, check_interval));
    ///
    /// // Setup AsyncTargetExecutor and let it run for 1s
    /// let mut exec = AsyncTargetExecutor::new();
    /// exec.start(vec![async_target]);
    /// sleep(Duration::from_secs(1));
    /// exec.stop();
    /// ```
    pub fn start(&mut self, targets: Vec<AsyncTarget<'static>>) {
        if self.worker.is_none() {
            // Setup teardown mechanism and construct runtime
            let (teardown_send, teardown_recv) = watch::channel(());
            let runtime = runtime::Builder::new_multi_thread().enable_time().build().unwrap();

            // Convert all targets into BoxFutures and execute them afterwards
            let tasks: Vec<BoxFuture<()>> = targets
                .into_iter()
                .map(|target| check_target_periodically(target, teardown_recv.clone()).boxed())
                .collect();

            // Spawn eventloop in a dedicated thread.
            // Note: After sending a shutdown message, all spawend tasks terminate.
            // The Problem here is that some async calles were offloaded to dedicated processing
            // threads. For a runtime to shutdown, these threads must have been processed, this
            // causes potentially a huge delay.
            // To prevent this, all unfinished tasks are moved to a detached thread
            // allowing this thread to terminate in a timely manner.
            let handle = spawn(move || {
                runtime.block_on(join_all(tasks));
                runtime.shutdown_background();
            });

            self.worker = Some((handle, teardown_send));
        }
    }

    /// Stop asynchronous processing started with [AsyncTargetExecutor::start] gracefully.
    pub fn stop(&mut self) {
        if let Some((handle, teardown_send)) = self.worker.take() {
            // Signal all async tasks to terminate and wait until runtime thread stopped.
            teardown_send.send(()).unwrap();
            handle.join().unwrap();
        }
    }
}

impl Default for AsyncTargetExecutor {
    fn default() -> Self {
        AsyncTargetExecutor::new()
    }
}

impl Drop for AsyncTargetExecutor {
    fn drop(&mut self) {
        self.stop()
    }
}

async fn check_target_periodically(mut target: AsyncTarget<'static>, mut teardown_recv: Receiver<()>) {
    loop {
        target = select! {
            // Teardown message was not received. Perform next check.
            target = check_target(target) => target,

            // Teardown message was received: Stop processing
            _ = teardown_recv.changed() => return,
        };
    }
}

async fn check_target(mut target: AsyncTarget<'static>) -> AsyncTarget<'static> {
    // Setup sleep timer to wait, to prevent further execution before the check_interval elapsed.
    let sleep = time::sleep(target.check_interval);

    // Offload potentially blocking check_availability call onto a separate thread
    let task = task::spawn_blocking(|| {
        // Check current target availability
        let (status, error) = match target.target.check_availability() {
            Ok(status) => (status, None),
            Err(error) => (Status::Unknown, Some(error)),
        };

        // Update stored status
        let old_status = target.status;
        target.status = status.clone();

        // Call stored Handler
        target.check_handler.as_mut()(target.target.as_ref(), status, old_status, error);
        target
    });

    // Wait until the task was processed and the sleep interval expired. Return given async_target
    let (tmp, _) = join(task, sleep).await;
    tmp.unwrap()
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
