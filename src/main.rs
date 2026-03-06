mod cmd;
mod config;
mod db;
mod indicators;
mod jquants;
mod llm;
mod output;
mod spec;

use anyhow::Result;
use clap::{Parser, Subcommand};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "kktd", about = "JP stock investment CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Output format
    #[arg(long, short, global = true, default_value = "json")]
    format: OutputFormat,
}

#[derive(Subcommand)]
enum Command {
    /// Fetch price data and compute TA indicators for watchlist stocks
    Scan {
        /// Number of days of historical data to fetch
        #[arg(long, default_value = "60")]
        days: u32,
    },
    /// Run investment evaluation (Buy/Hold/Avoid) via LLM
    Eval {
        /// Specific tickers to evaluate (default: all watchlist)
        #[arg()]
        tickers: Vec<String>,
    },
    /// Manage watchlist
    #[command(subcommand)]
    Watchlist(WatchlistCommand),
    /// Gather latest information for stocks via LLM (Gemini)
    Fetch {
        /// Specific tickers to fetch (default: all watchlist)
        #[arg()]
        tickers: Vec<String>,
    },
    /// Generate investment report as Markdown
    Report {
        /// Date filter (YYYY-MM-DD, default: today)
        #[arg(long)]
        date: Option<String>,
        /// Output file path (default: stdout)
        #[arg(long, short)]
        output: Option<String>,
    },
    /// List past evaluations
    History {
        /// Number of evaluations to show
        #[arg(long, default_value = "20")]
        limit: i64,
    },
}

#[derive(Subcommand)]
enum WatchlistCommand {
    /// Add a stock to watchlist
    Add {
        ticker: String,
        #[arg(long)]
        notes: Option<String>,
    },
    /// Remove a stock from watchlist
    Remove { ticker: String },
    /// List watchlist stocks
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let config = config::AppConfig::load()?;
    let conn = db::init_db().await?;

    match cli.command {
        Command::Scan { days } => {
            let results = cmd::scan::run(&conn, &config, days).await?;
            output::print_list_output(&results, cli.format);
        }
        Command::Eval { tickers } => {
            let results = cmd::eval::run(&conn, &config, &tickers).await?;
            output::print_list_output(&results, cli.format);
        }
        Command::Fetch { tickers } => {
            let results = cmd::fetch::run(&conn, &config, &tickers).await?;
            output::print_list_output(&results, cli.format);
        }
        Command::Report { date, output: out } => {
            let md = cmd::report::run(&conn, date.as_deref()).await?;
            if let Some(path) = out {
                std::fs::write(&path, &md)?;
                eprintln!("Report written to {}", path);
            } else {
                print!("{}", md);
            }
        }
        Command::Watchlist(sub) => match sub {
            WatchlistCommand::Add { ticker, notes } => {
                cmd::watchlist::add(&conn, &ticker, notes.as_deref()).await?;
                eprintln!("Added {} to watchlist", ticker);
            }
            WatchlistCommand::Remove { ticker } => {
                cmd::watchlist::remove(&conn, &ticker).await?;
                eprintln!("Removed {} from watchlist", ticker);
            }
            WatchlistCommand::List => {
                let items = cmd::watchlist::list(&conn).await?;
                output::print_list_output(&items, cli.format);
            }
        },
        Command::History { limit } => {
            let evals = db::list_evaluations(&conn, limit).await?;
            output::print_list_output(&evals, cli.format);
        }
    }

    Ok(())
}
