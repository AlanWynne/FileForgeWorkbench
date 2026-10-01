use std::path::{Path, PathBuf};

static PAYLOAD: &[u8] = include_bytes!("payload.zip");

fn main() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ffmdx -- Installer")
            .with_inner_size([520.0, 300.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "ffmdx-setup",
        options,
        Box::new(|_cc| Ok(Box::new(InstallerApp::new()))),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}

/// The installer's current stage.
#[derive(PartialEq)]
enum Stage {
    /// Awaiting the user to start the installation.
    Ready,
    /// Installation succeeded, carrying a success message.
    Done(String),
    /// Installation failed, carrying the error text.
    Failed(String),
}

/// The installer application state.
struct InstallerApp {
    install_dir: String,
    stage: Stage,
}

impl InstallerApp {
    fn new() -> Self {
        let default_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Programs")
            .join("ffmdx");
        Self {
            install_dir: default_dir.display().to_string(),
            stage: Stage::Ready,
        }
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);
            ui.heading("Markdown Explorer (ffmdx) Installer");
            ui.add_space(12.0);
            ui.label("Install location:");
            ui.text_edit_singleline(&mut self.install_dir);
            ui.add_space(16.0);

            match &self.stage {
                Stage::Ready => {
                    if ui.button("  Install  ").clicked() {
                        let dir = PathBuf::from(&self.install_dir);
                        match install(dir) {
                            Ok(msg) => self.stage = Stage::Done(msg),
                            Err(e) => self.stage = Stage::Failed(e.to_string()),
                        }
                    }
                }
                Stage::Done(msg) => {
                    ui.label(
                        egui::RichText::new(format!("[OK]  {msg}"))
                            .color(egui::Color32::from_rgb(100, 200, 100)),
                    );
                    ui.add_space(8.0);
                    ui.label("Open a new terminal and run:  ffmdx");
                    if ui.button("Close").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                Stage::Failed(e) => {
                    ui.label(
                        egui::RichText::new(format!("[FAILED]  {e}"))
                            .color(egui::Color32::from_rgb(220, 80, 80)),
                    );
                    if ui.button("Retry").clicked() {
                        self.stage = Stage::Ready;
                    }
                }
            }
        });
    }
}

fn install(dir: PathBuf) -> anyhow::Result<String> {
    std::fs::create_dir_all(&dir)?;

    let cursor = std::io::Cursor::new(PAYLOAD);
    let mut archive = zip::ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().ends_with('/') {
            continue;
        }
        let out_path = dir.join(file.name());
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out = std::fs::File::create(&out_path)?;
        std::io::copy(&mut file, &mut out)?;
    }

    #[cfg(windows)]
    add_to_user_path(&dir)?;

    Ok(format!("Installed to {}", dir.display()))
}

/// Returns whether `dir` already appears among the `;`-separated entries of
/// `current`, matched case-insensitively after trimming surrounding whitespace.
fn path_already_contains(current: &str, dir: &str) -> bool {
    current
        .split(';')
        .any(|p| p.trim().eq_ignore_ascii_case(dir.trim()))
}

/// Append `dir` to the `;`-separated `current` PATH, joining with `;` only when
/// `current` is non-empty. If `dir` is already present (case-insensitive,
/// trimmed), `current` is returned unchanged.
fn append_to_path(current: &str, dir: &str) -> String {
    if path_already_contains(current, dir) {
        current.to_string()
    } else if current.is_empty() {
        dir.to_string()
    } else {
        format!("{current};{dir}")
    }
}

#[cfg(windows)]
fn add_to_user_path(dir: &Path) -> anyhow::Result<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
    let current: String = env.get_value("Path").unwrap_or_default();
    let dir_str = dir.display().to_string();
    let new_path = append_to_path(&current, &dir_str);
    if new_path != current {
        env.set_value("Path", &new_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{append_to_path, path_already_contains};

    #[test]
    fn existing_case_insensitive_trimmed_entry_is_detected() {
        // Validates: Requirement 17.1
        let current = r"C:\other; c:\programs\FFMDX ;C:\more";
        assert!(path_already_contains(current, r"C:\Programs\ffmdx"));
    }

    #[test]
    fn existing_entry_leaves_path_unchanged() {
        // Validates: Requirement 17.2
        let current = r"C:\other;C:\programs\ffmdx";
        assert_eq!(append_to_path(current, r"C:\programs\ffmdx"), current);
    }

    #[test]
    fn new_entry_is_appended_with_semicolon_when_path_non_empty() {
        // Validates: Requirement 17.3
        let current = r"C:\other";
        assert_eq!(
            append_to_path(current, r"C:\programs\ffmdx"),
            r"C:\other;C:\programs\ffmdx"
        );
    }

    #[test]
    fn new_entry_has_no_leading_semicolon_when_path_empty() {
        // Validates: Requirement 17.3
        assert_eq!(
            append_to_path("", r"C:\programs\ffmdx"),
            r"C:\programs\ffmdx"
        );
    }
}
