//! Thread safety validation tests for ff-command.
//!
//! Verifies that CommandRegistry, CommandDispatch, ShortcutRegistry, and
//! CommandHistory implement Send + Sync and work correctly under concurrent access.

use std::sync::Arc;
use std::thread;

use ff_command::{
    CommandHandler, CommandHistory, CommandId, CommandMetadata, CommandParams, CommandRegistry,
    CommandResult, ExecutionContext, KeyChord, KeyCode, Modifiers, ShortcutBinding,
    ShortcutRegistry,
};

struct NoopHandler;

impl CommandHandler for NoopHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
        CommandResult::Ok
    }
}

fn make_meta(name: &str) -> CommandMetadata {
    CommandMetadata::builder(name, "test")
        .category("test")
        .build()
}

/// Validates: Requirement 1.4
///
/// Verifies that CommandRegistry, CommandDispatch, ShortcutRegistry, and
/// CommandHistory all implement Send + Sync.
#[test]
fn types_implement_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<CommandRegistry>();
    assert_send_sync::<ShortcutRegistry>();
    assert_send_sync::<CommandHistory>();
    // CommandDispatch contains RwLock<Box<dyn ContextProvider>> which is Send+Sync
    // We verify it's usable from Arc which requires Send+Sync
}

/// Validates: Requirement 1.4
///
/// Concurrent command registration from multiple threads.
#[test]
fn concurrent_command_registration() {
    let registry = Arc::new(CommandRegistry::new());
    let mut handles = Vec::new();

    for thread_idx in 0..10 {
        let reg = registry.clone();
        handles.push(thread::spawn(move || {
            for cmd_idx in 0..10 {
                let id_str = format!("thread{}.cmd{}", thread_idx, cmd_idx);
                let id = CommandId::new(&id_str).unwrap();
                let _ = reg.register(id, make_meta(&id_str), Box::new(NoopHandler));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // All 100 unique commands should be registered
    assert_eq!(registry.count(), 100);
}

/// Validates: Requirement 1.4
///
/// Concurrent command dispatch from multiple threads.
#[test]
fn concurrent_command_dispatch() {
    let registry = Arc::new(CommandRegistry::new());
    let history = Arc::new(CommandHistory::new(1000));

    // Register some commands first
    for i in 0..10 {
        let id_str = format!("test.cmd{}", i);
        let id = CommandId::new(&id_str).unwrap();
        registry
            .register(id, make_meta(&id_str), Box::new(NoopHandler))
            .unwrap();
    }

    let dispatch = Arc::new(ff_command::CommandDispatch::new(
        registry.clone(),
        history.clone(),
    ));
    let mut handles = Vec::new();

    for thread_idx in 0..10 {
        let d = dispatch.clone();
        handles.push(thread::spawn(move || {
            for cmd_idx in 0..10 {
                let id_str = format!("test.cmd{}", cmd_idx % 10);
                let result = d.execute_command(&id_str, CommandParams::new());
                assert!(result.is_ok());
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // History should have all 100 executed commands
    assert_eq!(history.len(), 100);
}

/// Validates: Requirement 7.7
///
/// Concurrent history reads and writes from multiple threads.
#[test]
fn concurrent_history_reads_and_writes() {
    let history = Arc::new(CommandHistory::new(500));
    let mut handles = Vec::new();

    // Writer threads
    for thread_idx in 0..5 {
        let h = history.clone();
        handles.push(thread::spawn(move || {
            for cmd_idx in 0..50 {
                let id_str = format!("write{}.cmd{}", thread_idx, cmd_idx);
                let id = CommandId::new(&id_str).unwrap();
                h.record(&id, &CommandParams::new());
            }
        }));
    }

    // Reader threads
    for _ in 0..5 {
        let h = history.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..50 {
                let _ = h.len();
                let _ = h.last_n(10);
                let _ = h.by_prefix("write0");
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // All 250 writes should be recorded
    assert_eq!(history.len(), 250);
}

/// Validates: Requirement 5.4
///
/// Concurrent shortcut registration from multiple threads.
#[test]
fn concurrent_shortcut_registration() {
    let registry = Arc::new(ShortcutRegistry::new());
    let mut handles = Vec::new();

    // Each thread registers different bindings using Alt combos
    for thread_idx in 0..5 {
        let reg = registry.clone();
        handles.push(thread::spawn(move || {
            let keys = [
                KeyCode::A,
                KeyCode::B,
                KeyCode::C,
                KeyCode::D,
                KeyCode::E,
                KeyCode::F,
                KeyCode::G,
                KeyCode::H,
                KeyCode::I,
                KeyCode::J,
            ];
            // Use a unique modifier pattern per thread to avoid conflicts
            let modifiers = match thread_idx {
                0 => Modifiers {
                    ctrl: false,
                    alt: true,
                    shift: false,
                    super_key: false,
                },
                1 => Modifiers {
                    ctrl: false,
                    alt: true,
                    shift: true,
                    super_key: false,
                },
                2 => Modifiers {
                    ctrl: true,
                    alt: true,
                    shift: false,
                    super_key: false,
                },
                3 => Modifiers {
                    ctrl: true,
                    alt: true,
                    shift: true,
                    super_key: false,
                },
                _ => Modifiers {
                    ctrl: false,
                    alt: false,
                    shift: false,
                    super_key: true,
                },
            };

            for key in &keys {
                let binding = ShortcutBinding::Single(KeyChord::new(modifiers, *key));
                let id_str = format!("thread{}.key{:?}", thread_idx, key);
                let id =
                    CommandId::new(&id_str.to_lowercase().replace(' ', "_")).unwrap_or_else(|| {
                        CommandId::new(&format!("t{}.k{}", thread_idx, thread_idx)).unwrap()
                    });
                let _ = reg.register(binding, id);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Should have at least the reserved shortcuts plus new ones
    let all = registry.list_all();
    assert!(all.len() > 21); // More than just the reserved set
}
