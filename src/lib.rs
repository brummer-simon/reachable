// TODO: Document all of this

mod error;
pub use error::{CheckTargetError, ParseTargetError, ResolveTargetError};

mod resolve_policy;
pub use resolve_policy::ResolvePolicy;

mod target;
pub use target::{IcmpTarget, Status, Target, TcpTarget};

#[cfg(feature = "async")]
mod async_target;

#[cfg(feature = "async")]
pub use async_target::{check_target, AsyncTarget, AsyncTargetExecutor, OldStatus};
