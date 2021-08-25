// TODO: Claim authorship
// TODO: Document all of this
// TODO: Publish on github

mod endpoint;
pub use endpoint::Endpoint;

mod status;
pub use status::Status;

mod target;
pub use target::Target;

#[cfg(feature = "async")]
mod endpoint_async;

#[cfg(feature = "async")]
pub use endpoint_async::{EndpointAsync, Exec, Executor, OldStatus};
