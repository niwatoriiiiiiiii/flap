use anyhow::{Context, Result};
use colored::*;
use std::env;
use std::fs;
#[cfg(unix)]
use std::io::Write;
use std::path::Path;

#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::*;

#[cfg(windows)]
const BIN_NAME: &str = "flap.exe";
#[cfg(unix)]
const BIN_NAME: &str = "flap";

#[cfg(windows)]
const INSTALL_DIR_BASE: &str = "USERPROFILE";
#[cfg(unix)]
const INSTALL_DIR_BASE: &str = "HOME";

fn main() {
    if let Err(e) = real_main() {
        println!("{} {:?}", "Error:".red().bold(), e);
    }

    println!();
    println!("Press Enter to exit...");
    let _ = std::io::stdin().read_line(&mut String::new());
}

fn real_main() -> Result<()> {
    println!("{}", "Starting Flap installation...".green().bold());

    // 1. Create install dir
    let home = env::var(INSTALL_DIR_BASE).context(format!("{} not set", INSTALL_DIR_BASE))?;
    let flap_dir = Path::new(&home).join(".flap");
    let bin_dir = flap_dir.join("bin");

    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir).context("Failed to create installation directory")?;
    }

    let dst_bin = bin_dir.join(BIN_NAME);

    // 2. Check for local binary (development or bundled)
    let current_exe = env::current_exe().context("Failed to get current exe path")?;
    let parent_dir = current_exe.parent().context("Failed to get parent dir")?;
    let local_bin = parent_dir.join(BIN_NAME);

    if local_bin.exists() {
        println!("Found local binary at: {:?}", local_bin);
        println!(
            "Copying {} to {}...",
            local_bin.display(),
            dst_bin.display()
        );
        fs::copy(&local_bin, &dst_bin).context("Failed to copy local binary")?;
    } else {
        // 3. Download from GitHub
        println!("No local binary found. Downloading from GitHub...");
        download_from_github(&bin_dir)?;
    }

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dst_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dst_bin, perms).context("Failed to set executable permissions")?;
    }

    // 4. Update PATH
    println!("Updating PATH...");
    add_to_path(&bin_dir)?;

    println!("{}", "Installation complete!".green().bold());
    println!("Please restart your terminal to use 'flap' command.");
    println!("Try running: {}", format!("{} --version", BIN_NAME).cyan());

    Ok(())
}

fn download_from_github(bin_dir: &Path) -> Result<()> {
    println!("Fetching latest release information...");
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("niwatoriiiiiiiii")
        .repo_name("flap")
        .build()?
        .fetch()?;

    let latest = releases
        .first()
        .ok_or_else(|| anyhow::anyhow!("No releases found on GitHub"))?;
    println!("Latest version: {}", latest.version);
    let target = if cfg!(target_os = "windows") {
        "x86_64-pc-windows-msvc"
    } else if cfg!(target_os = "macos") {
        "x86_64-apple-darwin"
    } else {
        "x86_64-unknown-linux-gnu"
    };

    // Try to find a direct binary asset first (e.g. flap-target.exe)
    let binary_asset = latest.assets.iter().find(|a| {
        a.name.contains(target)
            && (a.name.ends_with(".exe") || !a.name.contains('.'))
            && !a.name.contains("installer")
    });

    let (asset_name, download_url, is_archive) = if let Some(ba) = binary_asset {
        println!("Found direct binary asset: {}", ba.name);
        (ba.name.clone(), ba.download_url.clone(), false)
    } else {
        let ext = if cfg!(target_os = "windows") {
            ".zip"
        } else {
            ".tar.gz"
        };
        let archive = latest.asset_for(target, Some(ext)).ok_or_else(|| {
            anyhow::anyhow!(
                "No asset found for target {} with extension {}",
                target,
                ext
            )
        })?;
        (archive.name.clone(), archive.download_url.clone(), true)
    };

    let tmp_file_path = bin_dir.join(&asset_name);
    println!("Downloading from: {}", download_url);
    println!("Downloading {}...", asset_name);

    // Download to file
    let mut tmp_file = fs::File::create(&tmp_file_path).context("Failed to create temp file")?;
    self_update::Download::from_url(&download_url)
        .show_progress(true)
        .set_header(
            reqwest::header::ACCEPT,
            "application/octet-stream".parse().unwrap(),
        )
        .download_to(&mut tmp_file)
        .context("Failed to download asset")?;

    drop(tmp_file);

    // Verify file size
    let metadata = fs::metadata(&tmp_file_path).context("Failed to get asset metadata")?;
    let size_kb = metadata.len() / 1024;
    println!("Downloaded file size: {} KiB", size_kb);

    if metadata.len() < 10 * 1024 {
        anyhow::bail!(
            "Downloaded file is too small ({} KiB). This usually means the release asset is corrupted or the download URL pointed to an error page.\nURL: {}",
            size_kb,
            download_url
        );
    }

    if is_archive {
        println!("Extracting archive...");
        let tmp_dir = bin_dir.join("tmp_extract");
        if tmp_dir.exists() {
            fs::remove_dir_all(&tmp_dir).ok();
        }
        fs::create_dir_all(&tmp_dir).context("Failed to create temp extraction directory")?;

        if let Err(e) = extract_archive(&tmp_file_path, &tmp_dir) {
            println!(
                "{} Native extraction failed ({:?}). Trying fallback...",
                "Warning:".yellow(),
                e
            );
            fallback_extract(&tmp_file_path, &tmp_dir)?;
        }

        // Find binary in extracted files and move it
        if let Some(src_path) = find_binary(&tmp_dir, BIN_NAME) {
            fs::rename(&src_path, bin_dir.join(BIN_NAME)).context("Failed to move binary")?;
        } else {
            anyhow::bail!("Binary '{}' not found in the downloaded archive.", BIN_NAME);
        }

        println!("Cleaning up...");
        fs::remove_dir_all(&tmp_dir).ok();
        fs::remove_file(&tmp_file_path).ok();
    } else {
        // Direct move
        fs::rename(&tmp_file_path, bin_dir.join(BIN_NAME)).context("Failed to install binary")?;
    }

    Ok(())
}

fn extract_archive(archive_path: &Path, dst: &Path) -> Result<()> {
    if archive_path.extension().map_or(false, |ext| ext == "zip") {
        self_update::Extract::from_source(archive_path)
            .archive(self_update::ArchiveKind::Zip)
            .extract_into(dst)
            .context("Native ZIP extraction failed")?;
    } else {
        self_update::Extract::from_source(archive_path)
            .archive(self_update::ArchiveKind::Tar(Some(
                self_update::Compression::Gz,
            )))
            .extract_into(dst)
            .context("Native tar.gz extraction failed")?;
    }
    Ok(())
}

fn fallback_extract(archive_path: &Path, dst: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        let status = std::process::Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive_path.display(),
                    dst.display()
                ),
            ])
            .status()
            .context("Failed to run PowerShell fallback extraction")?;

        if status.success() {
            return Ok(());
        }
    }

    // Try standard tar (available on modern Windows and Unix)
    let mut cmd = std::process::Command::new("tar");
    if archive_path.extension().map_or(false, |ext| ext == "zip") {
        cmd.args(&["-xf", archive_path.to_str().unwrap()]);
    } else {
        cmd.args(&["-xzf", archive_path.to_str().unwrap()]);
    }
    cmd.args(&["-C", dst.to_str().unwrap()]);

    let status = cmd
        .status()
        .context("Failed to run tar fallback extraction")?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Both native and fallback extraction failed.")
    }
}

fn find_binary(dir: &Path, bin_name: &str) -> Option<std::path::PathBuf> {
    // 1. Direct check
    let direct = dir.join(bin_name);
    if direct.is_file() {
        return Some(direct);
    }

    // 2. Recursive search
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = find_binary(&path, bin_name) {
                    return Some(found);
                }
            } else if path.is_file() {
                if path.file_name().and_then(|n| n.to_str()) == Some(bin_name) {
                    return Some(path);
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn add_to_path(path: &Path) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    let path_str = path.to_str().context("Invalid path encoding")?;
    let current_path: String = env.get_value("Path").unwrap_or_default();

    if !current_path.contains(path_str) {
        let new_path = if current_path.is_empty() {
            path_str.to_string()
        } else {
            format!("{};{}", current_path, path_str)
        };

        env.set_value("Path", &new_path)
            .context("Failed to write Path registry value")?;
        println!("Added {} to PATH.", path_str);
    } else {
        println!("Path already configured.");
    }

    Ok(())
}

#[cfg(unix)]
fn add_to_path(path: &Path) -> Result<()> {
    let home = env::var("HOME").context("HOME not set")?;
    let path_str = path.to_str().context("Invalid path encoding")?;

    let profiles = [".bashrc", ".zshrc", ".profile"];
    let mut updated = false;

    for profile in profiles.iter() {
        let profile_path = Path::new(&home).join(profile);
        if profile_path.exists() {
            let content = fs::read_to_string(&profile_path).unwrap_or_default();
            if !content.contains(path_str) {
                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .open(&profile_path)
                    .context(format!("Failed to open {:?}", profile_path))?;

                writeln!(file, "\n# Flap installation").ok();
                writeln!(file, "export PATH=\"{}:$PATH\"", path_str).ok();
                println!("Updated {} with Flap bin path.", profile);
                updated = true;
            }
        }
    }

    if !updated {
        println!("Manual PATH configuration recommended:");
        println!(
            "Please add 'export PATH=\"{}:$PATH\"' to your shell profile.",
            path_str
        );
    } else {
        println!("PATH updated in shell profiles.");
    }

    Ok(())
}
