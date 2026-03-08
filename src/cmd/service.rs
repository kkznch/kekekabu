use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::process::Command;

fn home_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home))
}

const LABEL: &str = "com.kekekabu.pipeline";
const PLIST_FILENAME: &str = "com.kekekabu.pipeline.plist";

const PLIST_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/sh</string>
        <string>-c</string>
        <string>{bin} discover &amp;&amp; {bin} scan --days 60 &amp;&amp; {bin} fetch &amp;&amp; {bin} eval</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>8</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>{log_dir}/kekekabu.log</string>
    <key>StandardErrorPath</key>
    <string>{log_dir}/kekekabu.err</string>
</dict>
</plist>
"#;

fn ensure_macos() -> Result<()> {
    if !cfg!(target_os = "macos") {
        bail!("kabu service is macOS-only (launchd)");
    }
    Ok(())
}

fn plist_path() -> Result<PathBuf> {
    let home = home_dir()?;
    Ok(home.join("Library/LaunchAgents").join(PLIST_FILENAME))
}

fn log_dir() -> Result<PathBuf> {
    let home = home_dir()?;
    let dir = home.join("Library/Logs/kekekabu");
    std::fs::create_dir_all(&dir).context("Failed to create log directory")?;
    Ok(dir)
}

fn uid() -> Result<String> {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .context("Failed to run id -u")?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn generate_plist(bin_path: &str, log_path: &str) -> String {
    PLIST_TEMPLATE
        .replace("{label}", LABEL)
        .replace("{bin}", bin_path)
        .replace("{log_dir}", log_path)
}

pub fn install() -> Result<()> {
    ensure_macos()?;

    let bin = std::env::current_exe().context("Cannot determine binary path")?;
    let bin_str = bin.to_string_lossy();
    let logs = log_dir()?;
    let log_str = logs.to_string_lossy();

    let plist_content = generate_plist(&bin_str, &log_str);
    let path = plist_path()?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create LaunchAgents directory")?;
    }

    std::fs::write(&path, plist_content).context("Failed to write plist")?;

    println!("Installed: {}", path.display());
    println!("Label: {LABEL}");
    println!("Schedule: daily at 08:00");
    println!("Binary: {bin_str}");
    println!();
    println!("Run `kabu service start` to activate.");

    Ok(())
}

pub fn uninstall() -> Result<()> {
    ensure_macos()?;

    let path = plist_path()?;
    if !path.exists() {
        println!("Not installed (no plist found)");
        return Ok(());
    }

    // Try to stop first (ignore errors if not loaded)
    let uid = uid()?;
    let _ = Command::new("launchctl")
        .args(["bootout", &format!("gui/{uid}/{LABEL}")])
        .output();

    std::fs::remove_file(&path).context("Failed to remove plist")?;
    println!("Uninstalled: {}", path.display());

    Ok(())
}

pub fn start() -> Result<()> {
    ensure_macos()?;

    let path = plist_path()?;
    if !path.exists() {
        bail!("Service not installed. Run `kabu service install` first.");
    }

    let uid = uid()?;
    let output = Command::new("launchctl")
        .args(["bootstrap", &format!("gui/{uid}"), &path.to_string_lossy()])
        .output()
        .context("Failed to run launchctl bootstrap")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Error 37 = already loaded
        if stderr.contains("37") {
            println!("Service is already running");
            return Ok(());
        }
        bail!("launchctl bootstrap failed: {}", stderr);
    }

    println!("Service started ({LABEL})");
    Ok(())
}

pub fn stop() -> Result<()> {
    ensure_macos()?;

    let path = plist_path()?;
    if !path.exists() {
        bail!("Service not installed. Run `kabu service install` first.");
    }

    let uid = uid()?;
    let output = Command::new("launchctl")
        .args(["bootout", &format!("gui/{uid}/{LABEL}")])
        .output()
        .context("Failed to run launchctl bootout")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("3") {
            println!("Service is not running");
            return Ok(());
        }
        bail!("launchctl bootout failed: {}", stderr);
    }

    println!("Service stopped ({LABEL})");
    Ok(())
}

pub fn status() -> Result<()> {
    ensure_macos()?;

    let path = plist_path()?;
    if !path.exists() {
        println!("Status: Not installed");
        println!();
        println!("Run `kabu service install` to set up the launchd service.");
        return Ok(());
    }

    println!("Label: {LABEL}");
    println!("Plist: {}", path.display());

    let uid = uid()?;
    let output = Command::new("launchctl")
        .args(["print", &format!("gui/{uid}/{LABEL}")])
        .output()
        .context("Failed to run launchctl print")?;

    if output.status.success() {
        println!("Status: Running");
    } else {
        println!("Status: Installed (not running)");
        println!();
        println!("Run `kabu service start` to activate.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_plist_contains_binary_path() {
        let plist = generate_plist("/usr/local/bin/kabu", "/tmp/logs");
        assert!(plist.contains("/usr/local/bin/kabu"));
        assert!(plist.contains(LABEL));
    }

    #[test]
    fn test_generate_plist_contains_schedule() {
        let plist = generate_plist("/usr/local/bin/kabu", "/tmp/logs");
        assert!(plist.contains("<key>Hour</key>"));
        assert!(plist.contains("<integer>8</integer>"));
        assert!(plist.contains("<key>Minute</key>"));
        assert!(plist.contains("<integer>0</integer>"));
    }

    #[test]
    fn test_generate_plist_contains_pipeline_commands() {
        let plist = generate_plist("/path/to/kabu", "/tmp/logs");
        assert!(plist.contains(
            "/path/to/kabu discover &amp;&amp; /path/to/kabu scan --days 60 &amp;&amp; /path/to/kabu fetch &amp;&amp; /path/to/kabu eval"
        ));
    }

    #[test]
    fn test_generate_plist_contains_log_paths() {
        let plist = generate_plist("/usr/local/bin/kabu", "/home/user/logs");
        assert!(plist.contains("/home/user/logs/kekekabu.log"));
        assert!(plist.contains("/home/user/logs/kekekabu.err"));
    }

    #[test]
    fn test_generate_plist_valid_xml_structure() {
        let plist = generate_plist("/bin/kabu", "/tmp");
        assert!(plist.starts_with("<?xml version=\"1.0\""));
        assert!(plist.contains("<plist version=\"1.0\">"));
        assert!(plist.contains("</plist>"));
    }
}
