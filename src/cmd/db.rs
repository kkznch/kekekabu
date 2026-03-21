use anyhow::{Context, Result};
use rand::Rng;
use std::path::Path;
use tracing::info;

use crate::db;
use crate::output::{self, HumanDisplay, OutputFormat};

#[derive(serde::Serialize)]
struct DbStatus {
    path: String,
    size_bytes: u64,
    migrations: Vec<db::MigrationInfo>,
}

impl HumanDisplay for DbStatus {
    fn print_human(&self) {
        println!("Database: {}", self.path);
        println!("Size:     {} bytes", self.size_bytes);
        println!("Migrations ({}):", self.migrations.len());
        for m in &self.migrations {
            m.print_human();
        }
    }
}

pub async fn migrate(db_path: &Path, format: OutputFormat) -> Result<()> {
    info!("Running database migrations");
    let db = db::SqliteClient::open_or_create(db_path).await?;
    let migrations = db.migration_status().await?;
    output::print_list_output(&migrations, format);
    info!(count = migrations.len(), "Migrations applied");
    Ok(())
}

pub async fn status(db_path: &Path, format: OutputFormat) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found at {}", db_path.display());
    }

    let db = db::SqliteClient::open(db_path).await?;
    let migrations = db.migration_status().await?;

    let size = std::fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);
    let status = DbStatus {
        path: db_path.display().to_string(),
        size_bytes: size,
        migrations,
    };

    output::print_output(&status, format);
    Ok(())
}

pub fn reset(db_path: &Path, force: bool) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found at {}", db_path.display());
    }

    if !force {
        let code: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        eprintln!("⚠ This will permanently delete the database at:");
        eprintln!("  {}", db_path.display());
        eprintln!();
        eprintln!("All data (watchlist, evaluations, positions, trades, orders) will be lost.");
        eprintln!();
        eprintln!("To confirm, type this code: {code}");
        eprint!("> ");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Failed to read confirmation")?;

        if input.trim() != code {
            eprintln!("Code mismatch. Aborting.");
            return Ok(());
        }
    }

    // Remove main DB file + WAL/SHM files
    std::fs::remove_file(db_path)
        .with_context(|| format!("Failed to delete {}", db_path.display()))?;

    let wal = db_path.with_extension("db-wal");
    if wal.exists() {
        let _ = std::fs::remove_file(&wal);
    }
    let shm = db_path.with_extension("db-shm");
    if shm.exists() {
        let _ = std::fs::remove_file(&shm);
    }

    eprintln!("Database deleted: {}", db_path.display());
    eprintln!("Run `kabu db migrate` to recreate with fresh schema.");
    Ok(())
}
