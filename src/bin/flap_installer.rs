use anyhow::{Context, Result};
use colored::*;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

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

    let asset = latest
        .asset_for(target, None)
        .ok_or_else(|| anyhow::anyhow!("No asset found for target {}", target))?;

    let tmp_file_path = bin_dir.join(&asset.name);
    println!("Downloading {}...", asset.name);

    // Download to file
    let mut tmp_file = fs::File::create(&tmp_file_path).context("Failed to create temp file")?;
    self_update::Download::from_url(&asset.download_url)
        .show_progress(true)
        .download_to(&mut tmp_file)
        .context("Failed to download asset")?;

    // Drop file handle before extraction
    drop(tmp_file);

    if asset.name.ends_with(".zip") {
        extract_zip(&tmp_file_path, bin_dir)?;
    } else if asset.name.ends_with(".tar.gz") {
        extract_tar_gz(&tmp_file_path, bin_dir)?;
    } else {
        // Assume direct binary
        fs::rename(&tmp_file_path, bin_dir.join(BIN_NAME)).context("Failed to rename binary")?;
    }

    println!("Cleaning up...");
    fs::remove_file(&tmp_file_path).ok();

    Ok(())
}

fn extract_zip(zip_path: &Path, bin_dir: &Path) -> Result<()> {
    println!("Extracting ZIP...");
    #[cfg(windows)]
    {
        let status = Command::new("tar")
            .args(&[
                "-xf",
                zip_path.to_str().unwrap(),
                "-C",
                bin_dir.to_str().unwrap(),
            ])
            .status();

        if status.is_ok() && status.unwrap().success() {
            return Ok(());
        }

        let status = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    zip_path.display(),
                    bin_dir.display()
                ),
            ])
            .status()
            .context("Failed to run PowerShell for extraction")?;

        if !status.success() {
            anyhow::bail!("Failed to extract archive via PowerShell");
        }
    }
    #[cfg(unix)]
    {
        let status = Command::new("unzip")
            .args(&[
                "-o",
                zip_path.to_str().unwrap(),
                "-d",
                bin_dir.to_str().unwrap(),
            ])
            .status()
            .context("Failed to run unzip")?;

        if !status.success() {
            anyhow::bail!("Failed to extract ZIP via unzip");
        }
    }
    Ok(())
}

fn extract_tar_gz(tar_path: &Path, bin_dir: &Path) -> Result<()> {
    println!("Extracting tar.gz...");
    let status = Command::new("tar")
        .args(&[
            "-xzf",
            tar_path.to_str().unwrap(),
            "-C",
            bin_dir.to_str().unwrap(),
        ])
        .status()
        .context("Failed to run tar")?;

    if !status.success() {
        anyhow::bail!("Failed to extract tar.gz");
    }
    Ok(())
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
