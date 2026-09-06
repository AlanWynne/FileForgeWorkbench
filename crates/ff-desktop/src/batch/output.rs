use std::fs::OpenOptions;
use std::io::{self, Write};

/// Where batch command output is written.
pub enum BatchOutputSink {
    Stdout,
    File(String),
    Append(String),
}

impl BatchOutputSink {
    pub fn write_line(&self, line: &str) {
        match self {
            BatchOutputSink::Stdout => {
                let _ = writeln!(io::stdout(), "{}", line);
            }
            BatchOutputSink::File(path) => {
                if let Ok(mut f) = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                {
                    let _ = writeln!(f, "{}", line);
                }
            }
            BatchOutputSink::Append(path) => {
                if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
                    let _ = writeln!(f, "{}", line);
                }
            }
        }
    }

    pub fn write_command_echo(&self, cmd: &str) {
        self.write_line(&format!("===> {}", cmd));
    }
}
