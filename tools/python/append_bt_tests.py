LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\append_bt_tests.txt"
PATH = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\session_manager.rs"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

with open(PATH, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# Detect line ending
sep = b"\r\n" if b"\r\n" in data else b"\n"
log(f"Line ending: {repr(sep)}")

# The closing brace of the test module is the last `}` followed by a newline
# We insert our new tests before the final `}\n`
new_tests = (
    sep +
    b"    // Validates: global-search Requirement 6.2 -- search history persisted in session" + sep +
    b"    #[test]" + sep +
    b"    fn search_history_round_trips_through_session() {" + sep +
    b"        let tmp = TempDir::new().expect(\"tempdir\");" + sep +
    b"        let mgr = SessionManager::with_path(make_session_file(&tmp));" + sep +
    sep +
    b"        let history = vec![" + sep +
    b"            \"fn main\".to_string()," + sep +
    b"            \"TODO\".to_string()," + sep +
    b"            \"use std\".to_string()," + sep +
    b"        ];" + sep +
    b"        let state = ff_session::SessionState {" + sep +
    b"            search_history: history.clone()," + sep +
    b"            ..ff_session::SessionState::empty()" + sep +
    b"        };" + sep +
    b"        mgr.session_file.save(&state).expect(\"save\");" + sep +
    b"        let loaded = mgr.load();" + sep +
    b"        assert_eq!(loaded.search_history, history);" + sep +
    b"    }" + sep +
    sep +
    b"    // Validates: global-search Requirement 6.2 -- empty history survives round-trip" + sep +
    b"    #[test]" + sep +
    b"    fn empty_search_history_round_trips_through_session() {" + sep +
    b"        let tmp = TempDir::new().expect(\"tempdir\");" + sep +
    b"        let mgr = SessionManager::with_path(make_session_file(&tmp));" + sep +
    sep +
    b"        let state = ff_session::SessionState::empty();" + sep +
    b"        mgr.session_file.save(&state).expect(\"save\");" + sep +
    b"        let loaded = mgr.load();" + sep +
    b"        assert!(loaded.search_history.is_empty());" + sep +
    b"    }" + sep
)

# Find the last `}` + newline (closing brace of the test module)
closing = b"}" + sep
last_pos = data.rfind(closing)
if last_pos == -1:
    log("ERROR: could not find closing brace")
else:
    log(f"Inserting before position {last_pos}")
    data = data[:last_pos] + new_tests + data[last_pos:]
    with open(PATH, "wb") as f:
        f.write(data)
    log(f"Written. New size: {len(data)} bytes")

log("Done.")
