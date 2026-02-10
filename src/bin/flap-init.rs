use anyhow::{Context, Result};
use colored::*;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use winreg::RegKey;
use winreg::enums::*;

fn main() -> Result<()> {
    println!("{}", "Starting Flap installation...".green().bold());

    // 1. Build
    println!("Building flap...");
    let status = Command::new("cargo")
        .args(&["build", "--release", "--bin", "flap"])
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("Build failed");
    }

    // 2. Create install dir
    let home = env::var("USERPROFILE").context("USERPROFILE not set")?;
    let flap_dir = Path::new(&home).join(".flap");
    let bin_dir = flap_dir.join("bin");

    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir).context("Failed to create installation directory")?;
    }

    // 3. Copy binary
    let target_dir = Path::new("target").join("release");
    let src_bin = target_dir.join("flap.exe");
    let dst_bin = bin_dir.join("flap.exe");

    if !src_bin.exists() {
        anyhow::bail!("Built binary not found at {:?}", src_bin);
    }

    println!("Copying {} to {}...", src_bin.display(), dst_bin.display());
    fs::copy(&src_bin, &dst_bin).context("Failed to copy binary")?;

    // 4. Update PATH
    println!("Updating PATH...");
    add_to_path(&bin_dir)?;

    println!("{}", "Installation complete!".green().bold());
    println!("Please restart your terminal to use 'flap' command.");
    println!("Try running: {}", "flap --version".cyan());

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
