mod circuit_breaker;
mod cmd;
mod config;
mod db;
mod indicators;
mod jquants;
mod llm;
mod output;
#[allow(dead_code)]
mod portfolio;
mod spec;

use anyhow::Result;
use clap::{Parser, Subcommand};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "kabu", about = "JP stock investment CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Output format
    #[arg(long, short, global = true, default_value = "json")]
    format: OutputFormat,
}

#[derive(Subcommand)]
enum Command {
    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Discover stock candidates via LLM and update watchlist
    Discover,
    /// Fetch price data and compute TA indicators for watchlist stocks
    Scan {
        /// Number of days of historical data to fetch
        #[arg(long, default_value = "60")]
        days: u32,
    },
    /// Gather latest information for stocks via LLM (Gemini)
    Fetch {
        /// Specific tickers to fetch (default: all watchlist)
        #[arg()]
        tickers: Vec<String>,
    },
    /// Run investment evaluation (Buy/Hold/Avoid) via LLM
    Eval {
        /// Specific tickers to evaluate (default: all watchlist)
        #[arg()]
        tickers: Vec<String>,
    },
    /// Execute trades based on today's evaluations
    Execute {
        /// Dry run (don't actually place orders)
        #[arg(long, default_value = "true")]
        dry_run: bool,
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
    /// View database contents
    #[command(subcommand)]
    Show(ShowCommand),
}

#[derive(Subcommand)]
enum ShowCommand {
    /// Current watchlist
    Watchlist,
    /// Watchlist change events (add/remove/keep history)
    Events {
        /// Filter by ticker
        #[arg(long)]
        ticker: Option<String>,
    },
    /// Active portfolio positions
    Positions,
    /// Past evaluations
    Evaluations {
        /// Number of evaluations to show
        #[arg(long, default_value = "20")]
        limit: i64,
    },
    /// Registered stocks
    Stocks,
    /// Table row counts
    Tables,
    /// Portfolio summary
    Summary,
    /// Trade history
    Trades {
        #[arg(long, default_value = "20")]
        limit: i64,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Initialize config directory and template
    Init {
        /// Overwrite existing config
        #[arg(long)]
        force: bool,
    },
    /// Validate config and investment spec
    Validate,
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

    // config subcommands don't need DB
    if let Command::Config(sub) = cli.command {
        match sub {
            ConfigCommand::Init { force } => cmd::config::init(force)?,
            ConfigCommand::Validate => cmd::config::validate()?,
        }
        return Ok(());
    }

    let conn = db::init_db().await?;

    // show subcommands don't need config
    if let Command::Show(sub) = cli.command {
        let format = cli.format;
        match sub {
            ShowCommand::Watchlist => cmd::show::watchlist(&conn, format).await?,
            ShowCommand::Events { ticker } => {
                cmd::show::events(&conn, ticker.as_deref(), format).await?
            }
            ShowCommand::Positions => cmd::show::positions(&conn, format).await?,
            ShowCommand::Evaluations { limit } => {
                cmd::show::evaluations(&conn, limit, format).await?
            }
            ShowCommand::Stocks => cmd::show::stocks(&conn, format).await?,
            ShowCommand::Tables => cmd::show::tables(&conn, format).await?,
            ShowCommand::Summary => cmd::show::summary(&conn, format).await?,
            ShowCommand::Trades { limit } => cmd::show::trades(&conn, limit, format).await?,
        }
        return Ok(());
    }

    let config = config::AppConfig::load()?;

    match cli.command {
        Command::Config(_) | Command::Show(_) => unreachable!(),
        Command::Discover => {
            let result = cmd::discover::run(&conn, &config).await?;
            output::print_output(&result, cli.format);
        }
        Command::Scan { days } => {
            let results = cmd::scan::run(&conn, &config, days).await?;
            output::print_list_output(&results, cli.format);
        }
        Command::Fetch { tickers } => {
            let results = cmd::fetch::run(&conn, &config, &tickers).await?;
            output::print_list_output(&results, cli.format);
        }
        Command::Eval { tickers } => {
            let results = cmd::eval::run(&conn, &config, &tickers).await?;
            output::print_list_output(&results, cli.format);
        }
        Command::Execute { dry_run } => {
            let result = cmd::execute::run(&conn, &config, dry_run).await?;
            output::print_output(&result, cli.format);
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
    }

    Ok(())
}
