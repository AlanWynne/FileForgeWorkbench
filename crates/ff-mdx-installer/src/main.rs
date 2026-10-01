use std::path::{Path, PathBuf};

static PAYLOAD: &[u8] = include_bytes!("payload.zip");

fn main() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ffmdx — Installer")
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

#[derive(PartialEq)]
enum Stage {
    Ready,
    Done(String),
    Failed(String),
}

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
                        egui::RichText::new(format!("✅  {msg}"))
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
                        egui::RichText::new(format!("❌  {e}"))
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

#[cfg(windows)]
fn add_to_user_path(dir: &Path) -> anyhow::Result<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
    let current: String = env.get_value("Path").unwrap_or_default();
    let dir_str = dir.display().to_string();
    if !current
        .split(';')
        .any(|p| p.trim().eq_ignore_ascii_case(&dir_str))
    {
        let new_path = if current.is_empty() {
            dir_str
        } else {
            format!("{current};{dir_str}")
        };
        env.set_value("Path", &new_path)?;
    }
    Ok(())
}
