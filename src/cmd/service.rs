use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

const LABEL_PIPELINE: &str = "com.kekekabu.pipeline";
const LABEL_EXECUTE: &str = "com.kekekabu.execute";
const PLIST_FILENAME_PIPELINE: &str = "com.kekekabu.pipeline.plist";
const PLIST_FILENAME_EXECUTE: &str = "com.kekekabu.execute.plist";

/// Pipeline plist: workflow run at 08:00
const PLIST_TEMPLATE_PIPELINE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{bin}</string>
        <string>workflow</string>
        <string>run</string>
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

/// Execute plist: execute --live at 14:50 (before market close)
const PLIST_TEMPLATE_EXECUTE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{bin}</string>
        <string>execute</string>
        <string>--live</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>14</integer>
        <key>Minute</key>
        <integer>50</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>{log_dir}/kekekabu-execute.log</string>
    <key>StandardErrorPath</key>
    <string>{log_dir}/kekekabu-execute.err</string>
</dict>
</plist>
"#;

/// Abstraction over filesystem and process operations for testability.
pub trait ServiceRuntime {
    fn file_exists(&self, path: &Path) -> bool;
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    fn remove_file(&self, path: &Path) -> Result<()>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn current_exe(&self) -> Result<PathBuf>;
    fn run_command(&self, program: &str, args: &[&str]) -> Result<std::process::Output>;
}

/// Real implementation using std::fs and std::process::Command.
pub struct RealRuntime;

impl ServiceRuntime for RealRuntime {
    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        std::fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        std::fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory {}", path.display()))
    }

    fn current_exe(&self) -> Result<PathBuf> {
        std::env::current_exe().context("Cannot determine binary path")
    }

    fn run_command(&self, program: &str, args: &[&str]) -> Result<std::process::Output> {
        Command::new(program)
            .args(args)
            .output()
            .with_context(|| format!("Failed to run {} {}", program, args.join(" ")))
    }
}

fn home_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home))
}

fn ensure_macos() -> Result<()> {
    if !cfg!(target_os = "macos") {
        bail!("kabu service is macOS-only (launchd)");
    }
    Ok(())
}

fn plist_path_pipeline() -> Result<PathBuf> {
    let home = home_dir()?;
    Ok(home
        .join("Library/LaunchAgents")
        .join(PLIST_FILENAME_PIPELINE))
}

fn plist_path_execute() -> Result<PathBuf> {
    let home = home_dir()?;
    Ok(home
        .join("Library/LaunchAgents")
        .join(PLIST_FILENAME_EXECUTE))
}

fn log_dir_path() -> Result<PathBuf> {
    let home = home_dir()?;
    Ok(home.join("Library/Logs/kekekabu"))
}

fn uid(rt: &dyn ServiceRuntime) -> Result<String> {
    let output = rt.run_command("id", &["-u"])?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn generate_plist_pipeline(bin_path: &str, log_path: &str) -> String {
    PLIST_TEMPLATE_PIPELINE
        .replace("{label}", LABEL_PIPELINE)
        .replace("{bin}", bin_path)
        .replace("{log_dir}", log_path)
}

fn generate_plist_execute(bin_path: &str, log_path: &str) -> String {
    PLIST_TEMPLATE_EXECUTE
        .replace("{label}", LABEL_EXECUTE)
        .replace("{bin}", bin_path)
        .replace("{log_dir}", log_path)
}

pub fn install(rt: &dyn ServiceRuntime) -> Result<()> {
    ensure_macos()?;

    let bin = rt.current_exe()?;
    let bin_str = bin.to_string_lossy();
    let logs = log_dir_path()?;
    rt.create_dir_all(&logs)?;
    let log_str = logs.to_string_lossy();

    // Pipeline plist (workflow run at 08:00)
    let pipeline_content = generate_plist_pipeline(&bin_str, &log_str);
    let pipeline_path = plist_path_pipeline()?;
    if let Some(parent) = pipeline_path.parent() {
        rt.create_dir_all(parent)?;
    }
    rt.write_file(&pipeline_path, &pipeline_content)?;

    // Execute plist (execute --live at 14:50)
    let execute_content = generate_plist_execute(&bin_str, &log_str);
    let execute_path = plist_path_execute()?;
    rt.write_file(&execute_path, &execute_content)?;

    println!("Installed:");
    println!("  Pipeline: {} (daily at 08:00)", pipeline_path.display());
    println!("  Execute:  {} (daily at 14:50)", execute_path.display());
    println!("Binary: {bin_str}");
    println!();
    println!("Run `kabu service start` to activate.");

    Ok(())
}

pub fn uninstall(rt: &dyn ServiceRuntime) -> Result<()> {
    ensure_macos()?;

    let uid = uid(rt)?;

    for (label, path) in service_plists()? {
        if !rt.file_exists(&path) {
            continue;
        }
        let _ = rt.run_command("launchctl", &["bootout", &format!("gui/{uid}/{label}")]);
        rt.remove_file(&path)?;
        println!("Uninstalled: {}", path.display());
    }

    Ok(())
}

pub fn start(rt: &dyn ServiceRuntime) -> Result<()> {
    ensure_macos()?;

    let uid = uid(rt)?;
    let domain = format!("gui/{uid}");

    for (label, path) in service_plists()? {
        if !rt.file_exists(&path) {
            bail!("Service not installed. Run `kabu service install` first.");
        }
        let path_str = path.to_string_lossy().to_string();
        let output = rt.run_command("launchctl", &["bootstrap", &domain, &path_str])?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("37") {
                println!("{label}: already running");
                continue;
            }
            bail!("launchctl bootstrap failed for {label}: {}", stderr);
        }
        println!("{label}: started");
    }
    Ok(())
}

pub fn stop(rt: &dyn ServiceRuntime) -> Result<()> {
    ensure_macos()?;

    let uid = uid(rt)?;

    for (label, path) in service_plists()? {
        if !rt.file_exists(&path) {
            continue;
        }
        let target = format!("gui/{uid}/{label}");
        let output = rt.run_command("launchctl", &["bootout", &target])?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("3") {
                println!("{label}: not running");
                continue;
            }
            bail!("launchctl bootout failed for {label}: {}", stderr);
        }
        println!("{label}: stopped");
    }
    Ok(())
}

pub fn status(rt: &dyn ServiceRuntime) -> Result<()> {
    ensure_macos()?;

    let uid = uid(rt)?;
    let plists = service_plists()?;
    let any_installed = plists.iter().any(|(_, p)| rt.file_exists(p));

    if !any_installed {
        println!("Status: Not installed");
        println!();
        println!("Run `kabu service install` to set up the launchd services.");
        return Ok(());
    }

    for (label, path) in &plists {
        if !rt.file_exists(path) {
            println!("{label}: Not installed");
            continue;
        }
        let target = format!("gui/{uid}/{label}");
        let output = rt.run_command("launchctl", &["print", &target])?;
        if output.status.success() {
            println!("{label}: Running");
        } else {
            println!("{label}: Installed (not running)");
        }
    }

    Ok(())
}

/// Return all service plist (label, path) pairs.
fn service_plists() -> Result<Vec<(&'static str, PathBuf)>> {
    Ok(vec![
        (LABEL_PIPELINE, plist_path_pipeline()?),
        (LABEL_EXECUTE, plist_path_execute()?),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockRuntime {
        exe_path: PathBuf,
        existing_files: RefCell<Vec<PathBuf>>,
        written_files: RefCell<Vec<(PathBuf, String)>>,
        removed_files: RefCell<Vec<PathBuf>>,
        created_dirs: RefCell<Vec<PathBuf>>,
        command_outputs: RefCell<Vec<std::process::Output>>,
    }

    impl MockRuntime {
        fn new(exe_path: &str) -> Self {
            Self {
                exe_path: PathBuf::from(exe_path),
                existing_files: RefCell::new(Vec::new()),
                written_files: RefCell::new(Vec::new()),
                removed_files: RefCell::new(Vec::new()),
                created_dirs: RefCell::new(Vec::new()),
                command_outputs: RefCell::new(Vec::new()),
            }
        }

        fn with_existing_file(self, path: &str) -> Self {
            self.existing_files.borrow_mut().push(PathBuf::from(path));
            self
        }

        fn with_command_output(self, stdout: &str, success: bool) -> Self {
            use std::os::unix::process::ExitStatusExt;
            self.command_outputs
                .borrow_mut()
                .push(std::process::Output {
                    status: std::process::ExitStatus::from_raw(if success { 0 } else { 256 }),
                    stdout: stdout.as_bytes().to_vec(),
                    stderr: Vec::new(),
                });
            self
        }
    }

    impl ServiceRuntime for MockRuntime {
        fn file_exists(&self, path: &Path) -> bool {
            self.existing_files.borrow().iter().any(|p| p == path)
        }

        fn write_file(&self, path: &Path, content: &str) -> Result<()> {
            self.written_files
                .borrow_mut()
                .push((path.to_path_buf(), content.to_string()));
            Ok(())
        }

        fn remove_file(&self, path: &Path) -> Result<()> {
            self.removed_files.borrow_mut().push(path.to_path_buf());
            Ok(())
        }

        fn create_dir_all(&self, path: &Path) -> Result<()> {
            self.created_dirs.borrow_mut().push(path.to_path_buf());
            Ok(())
        }

        fn current_exe(&self) -> Result<PathBuf> {
            Ok(self.exe_path.clone())
        }

        fn run_command(&self, _program: &str, _args: &[&str]) -> Result<std::process::Output> {
            let mut outputs = self.command_outputs.borrow_mut();
            if outputs.is_empty() {
                use std::os::unix::process::ExitStatusExt;
                Ok(std::process::Output {
                    status: std::process::ExitStatus::from_raw(0),
                    stdout: b"501\n".to_vec(),
                    stderr: Vec::new(),
                })
            } else {
                Ok(outputs.remove(0))
            }
        }
    }

    #[test]
    fn test_generate_plist_pipeline() {
        let plist = generate_plist_pipeline("/usr/local/bin/kabu", "/tmp/logs");
        assert!(plist.contains("/usr/local/bin/kabu"));
        assert!(plist.contains(LABEL_PIPELINE));
        assert!(plist.contains("<string>workflow</string>"));
        assert!(plist.contains("<string>run</string>"));
        assert!(plist.contains("<integer>8</integer>"));
        assert!(plist.contains("/tmp/logs/kekekabu.log"));
    }

    #[test]
    fn test_generate_plist_execute() {
        let plist = generate_plist_execute("/usr/local/bin/kabu", "/tmp/logs");
        assert!(plist.contains("/usr/local/bin/kabu"));
        assert!(plist.contains(LABEL_EXECUTE));
        assert!(plist.contains("<string>execute</string>"));
        assert!(plist.contains("<string>--live</string>"));
        assert!(plist.contains("<integer>14</integer>"));
        assert!(plist.contains("<integer>50</integer>"));
        assert!(plist.contains("/tmp/logs/kekekabu-execute.log"));
    }

    #[test]
    fn test_generate_plist_valid_xml_structure() {
        let plist = generate_plist_pipeline("/bin/kabu", "/tmp");
        assert!(plist.starts_with("<?xml version=\"1.0\""));
        assert!(plist.contains("<plist version=\"1.0\">"));
        assert!(plist.contains("</plist>"));
    }

    #[test]
    fn test_install_writes_both_plists() {
        let rt = MockRuntime::new("/usr/local/bin/kabu").with_command_output("501\n", true);
        install(&rt).unwrap();

        let written = rt.written_files.borrow();
        assert_eq!(written.len(), 2);
        assert!(
            written[0]
                .0
                .to_string_lossy()
                .contains(PLIST_FILENAME_PIPELINE)
        );
        assert!(
            written[1]
                .0
                .to_string_lossy()
                .contains(PLIST_FILENAME_EXECUTE)
        );
    }

    #[test]
    fn test_uninstall_removes_both_plists() {
        let pipeline = plist_path_pipeline().unwrap();
        let execute = plist_path_execute().unwrap();
        let rt = MockRuntime::new("/usr/local/bin/kabu")
            .with_existing_file(&pipeline.to_string_lossy())
            .with_existing_file(&execute.to_string_lossy())
            .with_command_output("501\n", true) // uid
            .with_command_output("", true) // bootout pipeline
            .with_command_output("", true); // bootout execute
        uninstall(&rt).unwrap();

        let removed = rt.removed_files.borrow();
        assert_eq!(removed.len(), 2);
    }

    #[test]
    fn test_uninstall_not_installed() {
        let rt = MockRuntime::new("/usr/local/bin/kabu").with_command_output("501\n", true); // uid
        uninstall(&rt).unwrap();
        assert!(rt.removed_files.borrow().is_empty());
    }
}
