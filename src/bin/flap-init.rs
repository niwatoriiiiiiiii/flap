use anyhow::{Context, Result};
use colored::*;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use winreg::RegKey;
use winreg::enums::*;

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
    let home = env::var("USERPROFILE").context("USERPROFILE not set")?;
    let flap_dir = Path::new(&home).join(".flap");
    let bin_dir = flap_dir.join("bin");

    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir).context("Failed to create installation directory")?;
    }

    let dst_bin = bin_dir.join("flap.exe");

    // 2. Check for local binary (development or bundled)
    let current_exe = env::current_exe().context("Failed to get current exe path")?;
    let parent_dir = current_exe.parent().context("Failed to get parent dir")?;
    let local_bin = parent_dir.join("flap.exe");

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

    // 4. Update PATH
    println!("Updating PATH...");
    add_to_path(&bin_dir)?;

    println!("{}", "Installation complete!".green().bold());
    println!("Please restart your terminal to use 'flap' command.");
    println!("Try running: {}", "flap --version".cyan());

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

    let asset = latest
        .asset_for("x86_64-pc-windows-msvc", None)
        .ok_or_else(|| anyhow::anyhow!("No asset found for x86_64-pc-windows-msvc ZIP"))?;

    let tmp_zip = bin_dir.join("flap_download.zip");
    println!("Downloading {}...", asset.name);

    // Download to file
    let mut tmp_file = fs::File::create(&tmp_zip).context("Failed to create temp zip file")?;
    self_update::Download::from_url(&asset.download_url)
        .show_progress(true)
        .download_to(&mut tmp_file)
        .context("Failed to download asset")?;

    // Drop file handle before extraction
    drop(tmp_file);

    println!("Extracting...");
    // Try tar (available in Win10 1803+)
    let status = Command::new("tar")
        .args(&[
            "-xf",
            tmp_zip.to_str().unwrap(),
            "-C",
            bin_dir.to_str().unwrap(),
        ])
        .status();

    let success = match status {
        Ok(s) => s.success(),
        Err(_) => false,
    };

    if !success {
        println!("tar failed or not found, trying PowerShell...");
        let status = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    tmp_zip.display(),
                    bin_dir.display()
                ),
            ])
            .status()
            .context("Failed to run PowerShell for extraction")?;

        if !status.success() {
            anyhow::bail!("Failed to extract archive via PowerShell");
        }
    }

    println!("Cleaning up...");
    fs::remove_file(&tmp_zip).ok();

    Ok(())
}

fn add_to_path(path: &Path) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    let path_str = path.to_str().context("Invalid path encoding")?;

    let current_path: String = env.get_value("Path").unwrap_or_default();

    // Simple check if path implies existence
    // A robust check splits by literal semicolon
    // We should split by ; and check exact match to avoid substrings issues?
    // But normalized path comparison is hard.
    // Let's restart checking if it contains the string.

    if !current_path.contains(path_str) {
        let new_path = if current_path.is_empty() {
            path_str.to_string()
        } else {
            format!("{};{}", current_path, path_str)
        };

        env.set_value("Path", &new_path)
            .context("Failed to write Path registry value")?;
        println!("Added {} to PATH.", path_str);

        // Broadcast change?
        // Need unsafe code to broadcast WM_SETTINGCHANGE.
        // For CLI installer, usually asking user to restart terminal is acceptable or sending message.
        // Rustup does it properly.
        // We will skip broadcasting for this simple script, sticking to registry update.
    } else {
        println!("Path already configured.");
    }

    Ok(())
}
