// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Author: Simon Brummer (simon.brummer@posteo.de)

mod error;
pub use error::{CheckTargetError, ParseTargetError, ResolveTargetError};

mod resolve_policy;
pub use resolve_policy::ResolvePolicy;

mod target;
pub use target::{IcmpTarget, Status, Target, TcpTarget};

#[cfg(feature = "async")]
mod async_target;

#[cfg(feature = "async")]
pub use async_target::{AsyncTarget, AsyncTargetExecutor, OldStatus};
