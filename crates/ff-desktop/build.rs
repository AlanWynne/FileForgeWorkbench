use std::path::{Path, PathBuf};

fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("ffwb.manifest");
        res.compile().expect("failed to compile Windows resources");
    }

    stage_i18n_catalogue();
}

/// Stage the shipped `i18n/` catalogue tree next to the built binary
/// (CR-NR-103 Fix 2b).
///
/// At runtime `shell::i18n_bootstrap::resolve_i18n_root` prefers `exe_dir/i18n`
/// (the installed layout). Nothing previously produced that directory, so a
/// shipped build found no catalogue and fell back to English. This copies
/// `crates/ff-i18n/i18n` into `target/<profile>/i18n` after every build so the
/// run-from-target binary (and a packaged copy made from the target dir) resolves
/// the `af`/`fr`/`de` overlays. Mirrors how the help loader relies on a tree
/// beside the binary.
///
/// Failures are non-fatal: a missing source or a copy error prints a cargo
/// warning and leaves the build succeeding, exactly as the runtime resolver
/// degrades to the dev-tree fallback and then the key fallback.
fn stage_i18n_catalogue() {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            println!("cargo:warning=[i18n] CARGO_MANIFEST_DIR unset -- skipping i18n staging");
            return;
        }
    };
    // crates/ff-desktop -> crates/ff-i18n/i18n
    let source = manifest_dir.join("..").join("ff-i18n").join("i18n");
    if !source.is_dir() {
        println!(
            "cargo:warning=[i18n] source catalogue {} not found -- skipping staging",
            source.display()
        );
        return;
    }

    let profile_dir = match profile_output_dir() {
        Some(dir) => dir,
        None => {
            println!(
                "cargo:warning=[i18n] could not locate target profile dir -- skipping staging"
            );
            return;
        }
    };
    let dest = profile_dir.join("i18n");

    // Re-run staging when the source tree changes.
    println!("cargo:rerun-if-changed={}", source.display());

    if let Err(e) = copy_dir_recursive(&source, &dest) {
        println!(
            "cargo:warning=[i18n] failed to stage catalogue into {}: {}",
            dest.display(),
            e
        );
    }
}

/// Resolve the `target/<profile>/` directory from `OUT_DIR`.
///
/// `OUT_DIR` has the shape `target/<profile>/build/<pkg-hash>/out`; three parent
/// hops reach `target/<profile>/`. Returns `None` when `OUT_DIR` is unset or too
/// shallow to contain the expected layout.
fn profile_output_dir() -> Option<PathBuf> {
    let out_dir = std::env::var("OUT_DIR").ok()?;
    let out_dir = PathBuf::from(out_dir);
    // out -> <pkg-hash> -> build -> <profile>
    out_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

/// Recursively copy `src` into `dst`, creating directories as needed and
/// overwriting existing files so a stale staged catalogue is refreshed.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
