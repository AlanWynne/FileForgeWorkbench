use std::path::{Path, PathBuf};
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    pub rx: Receiver<PathBuf>,
}

impl FileWatcher {
    /// Watch `root` for `.md` changes. Emits debounced paths (~400 ms).
    pub fn new(root: &Path) -> anyhow::Result<Self> {
        let (raw_tx, raw_rx): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();
        let (debounced_tx, debounced_rx): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        for path in event.paths {
                            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                                let _ = raw_tx.send(path);
                            }
                        }
                    }
                }
            },
            Config::default(),
        )?;

        watcher.watch(root, RecursiveMode::Recursive)?;

        std::thread::spawn(move || {
            while let Ok(path) = raw_rx.recv() {
                let mut last = path;
                while let Ok(p) = raw_rx.recv_timeout(Duration::from_millis(400)) {
                    last = p;
                }
                let _ = debounced_tx.send(last);
            }
        });

        Ok(Self { _watcher: watcher, rx: debounced_rx })
    }
}
