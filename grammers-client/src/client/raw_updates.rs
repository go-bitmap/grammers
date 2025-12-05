// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simplified update stream that directly forwards raw `UpdatesLike` without caching or processing.

use super::Client;
use grammers_mtsender::InvocationError;
pub use grammers_session::updates::UpdatesLike;
use tokio::sync::mpsc;

/// A simplified update stream that directly forwards raw `UpdatesLike` updates.
///
/// Unlike [`UpdateStream`](super::updates::UpdateStream), this stream:
/// - Does not cache messages or peer information
/// - Does not handle update ordering or gaps
/// - Does not automatically fetch differences
/// - Directly returns raw `UpdatesLike` from the socket
///
/// This is useful when you want to handle updates yourself without the overhead
/// of message caching and state management.
pub struct RawUpdateStream {
    updates: mpsc::UnboundedReceiver<UpdatesLike>,
}

impl RawUpdateStream {
    /// Get the next update from the stream.
    ///
    /// Returns `Ok(UpdatesLike)` when an update is available, or `Err(InvocationError::Dropped)`
    /// when the stream is closed.
    pub async fn next(&mut self) -> Result<UpdatesLike, InvocationError> {
        self.updates
            .recv()
            .await
            .ok_or(InvocationError::Dropped)
    }
}

impl Client {
    /// Returns a simplified asynchronous stream of raw updates.
    ///
    /// Unlike [`stream_updates`](Self::stream_updates), this method returns a stream that:
    /// - Directly forwards `UpdatesLike` without any processing
    /// - Does not cache messages or peer information
    /// - Does not handle update ordering or gaps
    /// - Does not automatically fetch differences
    ///
    /// This is useful when you want full control over update handling without the overhead
    /// of message caching and state management.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use grammers_client::Client;
    /// use grammers_session::updates::UpdatesLike;
    ///
    /// # async fn example(client: Client, updates: tokio::sync::mpsc::UnboundedReceiver<UpdatesLike>) {
    /// let mut stream = client.stream_raw_updates(updates);
    /// while let Ok(update) = stream.next().await {
    ///     match update {
    ///         UpdatesLike::Updates(updates) => {
    ///             // Handle updates directly
    ///         }
    ///         UpdatesLike::ShortSentMessage { .. } => {
    ///             // Handle short sent message
    ///         }
    ///         UpdatesLike::AffectedMessages(_) => {
    ///             // Handle affected messages
    ///         }
    ///         UpdatesLike::InvitedUsers(_) => {
    ///             // Handle invited users
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn stream_raw_updates(
        &self,
        updates: mpsc::UnboundedReceiver<UpdatesLike>,
    ) -> RawUpdateStream {
        RawUpdateStream { updates }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::future::Future;

    fn get_raw_update_stream() -> RawUpdateStream {
        panic!()
    }

    #[test]
    fn ensure_next_update_future_impls_send() {
        if false {
            // We just want it to type-check, not actually run.
            fn typeck(_: impl Future + Send) {}
            typeck(get_raw_update_stream().next());
        }
    }
}
