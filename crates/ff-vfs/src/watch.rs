//! File watching types and handle for the VFS abstraction layer.
//!
//! Defines the `WatchHandle` for receiving file change events and the `WatchEvent`
//! enum representing the types of changes that can occur.

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::uri::ResourceUri;

/// The type of change that occurred on a watched resource.
///
/// Addresses: Requirement 7 AC 2
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WatchEvent {
    /// A new resource was created at the given URI.
    Created(ResourceUri),
    /// The content of the resource at the given URI was modified.
    Modified(ResourceUri),
    /// The resource at the given URI was deleted.
    Deleted(ResourceUri),
    /// A resource was renamed/moved from `old_uri` to `new_uri`.
    Renamed {
        /// The original URI before the rename.
        old_uri: ResourceUri,
        /// The new URI after the rename.
        new_uri: ResourceUri,
    },
}

/// A handle for a file watch subscription.
///
/// Provides an async receiver for watch events and a method to cancel the subscription.
/// Returned by `VfsProvider::watch()`.
///
/// Addresses: Requirement 7 AC 1, AC 3, AC 4
#[derive(Debug)]
pub struct WatchHandle {
    /// Receiver for incoming watch events.
    receiver: mpsc::Receiver<WatchEvent>,
    /// Token used to signal cancellation of the watch.
    cancel_token: CancellationToken,
}

impl WatchHandle {
    /// Create a new `WatchHandle` from a receiver and cancellation token.
    pub fn new(receiver: mpsc::Receiver<WatchEvent>, cancel_token: CancellationToken) -> Self {
        Self {
            receiver,
            cancel_token,
        }
    }

    /// Receive the next watch event asynchronously.
    ///
    /// Returns `None` if the watch has been cancelled or the sender has been dropped.
    pub async fn recv(&mut self) -> Option<WatchEvent> {
        self.receiver.recv().await
    }

    /// Cancel the watch subscription, stopping event delivery and releasing resources.
    ///
    /// Addresses: Requirement 7 AC 4
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Returns a reference to the cancellation token.
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
    }
}
