//! Notify event → VFS WatchEvent conversion.
//!
//! Converts raw `notify::Event` values into the VFS `WatchEvent` type.
//!
//! Addresses: Requirement 3, criteria 2–3, 11

use ff_vfs::uri::ResourceUri;
use ff_vfs::watch::WatchEvent;

/// Convert a `notify::Event` to a VFS `WatchEvent`.
///
/// Returns `None` if the event kind doesn't map to a meaningful VFS event.
///
/// Validates: Requirement 3 AC 2, AC 3, AC 11
pub fn convert_notify_event(event: &notify::Event) -> Option<WatchEvent> {
    use notify::event::{ModifyKind, RenameMode};
    use notify::EventKind;

    let paths = &event.paths;
    if paths.is_empty() {
        return None;
    }

    let primary_path = &paths[0];
    let uri_path = primary_path.to_string_lossy().replace('\\', "/");
    let uri = ResourceUri::new("local", format!("/{}", uri_path.trim_start_matches('/')));

    match &event.kind {
        EventKind::Create(_) => Some(WatchEvent::Created(uri)),
        EventKind::Modify(ModifyKind::Name(rename_mode)) => {
            // Rename events in notify 6.x come through as Modify(Name(...))
            match rename_mode {
                RenameMode::Both => {
                    if paths.len() >= 2 {
                        let new_path = &paths[1];
                        let new_uri_path = new_path.to_string_lossy().replace('\\', "/");
                        let new_uri = ResourceUri::new(
                            "local",
                            format!("/{}", new_uri_path.trim_start_matches('/')),
                        );
                        Some(WatchEvent::Renamed {
                            old_uri: uri,
                            new_uri,
                        })
                    } else {
                        Some(WatchEvent::Modified(uri))
                    }
                }
                RenameMode::From => Some(WatchEvent::Deleted(uri)),
                RenameMode::To => Some(WatchEvent::Created(uri)),
                _ => Some(WatchEvent::Modified(uri)),
            }
        }
        EventKind::Modify(_) => Some(WatchEvent::Modified(uri)),
        EventKind::Remove(_) => Some(WatchEvent::Deleted(uri)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn convert_create_event() {
        let event = notify::Event {
            kind: notify::EventKind::Create(notify::event::CreateKind::File),
            paths: vec![PathBuf::from("/home/user/new_file.txt")],
            attrs: Default::default(),
        };

        let result = convert_notify_event(&event).unwrap();
        match result {
            WatchEvent::Created(uri) => {
                assert_eq!(uri.scheme(), "local");
                assert!(uri.path().contains("new_file.txt"));
            }
            _ => panic!("expected Created event"),
        }
    }

    #[test]
    fn convert_modify_event() {
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/home/user/modified.txt")],
            attrs: Default::default(),
        };

        let result = convert_notify_event(&event).unwrap();
        match result {
            WatchEvent::Modified(uri) => {
                assert!(uri.path().contains("modified.txt"));
            }
            _ => panic!("expected Modified event"),
        }
    }

    #[test]
    fn convert_delete_event() {
        let event = notify::Event {
            kind: notify::EventKind::Remove(notify::event::RemoveKind::File),
            paths: vec![PathBuf::from("/home/user/deleted.txt")],
            attrs: Default::default(),
        };

        let result = convert_notify_event(&event).unwrap();
        match result {
            WatchEvent::Deleted(uri) => {
                assert!(uri.path().contains("deleted.txt"));
            }
            _ => panic!("expected Deleted event"),
        }
    }

    #[test]
    fn convert_rename_event_with_both_paths() {
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Name(
                notify::event::RenameMode::Both,
            )),
            paths: vec![
                PathBuf::from("/home/user/old_name.txt"),
                PathBuf::from("/home/user/new_name.txt"),
            ],
            attrs: Default::default(),
        };

        let result = convert_notify_event(&event).unwrap();
        match result {
            WatchEvent::Renamed { old_uri, new_uri } => {
                assert!(old_uri.path().contains("old_name.txt"));
                assert!(new_uri.path().contains("new_name.txt"));
            }
            _ => panic!("expected Renamed event"),
        }
    }

    #[test]
    fn empty_paths_returns_none() {
        let event = notify::Event {
            kind: notify::EventKind::Create(notify::event::CreateKind::File),
            paths: vec![],
            attrs: Default::default(),
        };

        assert!(convert_notify_event(&event).is_none());
    }
}
