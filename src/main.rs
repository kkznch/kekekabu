mod circuit_breaker;
mod cmd;
mod config;
mod db;
mod indicators;
mod jquants;
mod llm;
mod output;
mod portfolio;
mod spec;

use anyhow::Result;
use clap::{Parser, Subcommand};
use output::OutputFormat;
use rust_decimal::Decimal;

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
    Discover {
        /// List current watchlist instead of running discovery
        #[arg(long)]
        list: bool,
    },
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
    /// Manage portfolio positions
    #[command(subcommand)]
    Portfolio(PortfolioCommand),
    /// List past evaluations
    History {
        /// Number of evaluations to show
        #[arg(long, default_value = "20")]
        limit: i64,
    },
}

#[derive(Subcommand)]
enum PortfolioCommand {
    /// Record a buy
    Buy {
        ticker: String,
        #[arg(long)]
        quantity: Decimal,
        #[arg(long)]
        price: Decimal,
        #[arg(long)]
        strategy: Option<String>,
    },
    /// Record a sell
    Sell {
        ticker: String,
        #[arg(long)]
        quantity: Decimal,
        #[arg(long)]
        price: Decimal,
        #[arg(long)]
        strategy: Option<String>,
    },
    /// List active positions
    Positions,
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

    let config = config::AppConfig::load()?;
    let conn = db::init_db().await?;

    match cli.command {
        Command::Config(_) => unreachable!(),
        Command::Discover { list } => {
            if list {
                let items = cmd::discover::list(&conn).await?;
                output::print_list_output(&items, cli.format);
            } else {
                let result = cmd::discover::run(&conn, &config).await?;
                output::print_output(&result, cli.format);
            }
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
        Command::Portfolio(sub) => match sub {
            PortfolioCommand::Buy {
                ticker,
                quantity,
                price,
                strategy,
            } => {
                portfolio::buy(&conn, &ticker, quantity, price, strategy.as_deref()).await?;
                eprintln!("Recorded buy: {} x {} @ {}", ticker, quantity, price);
            }
            PortfolioCommand::Sell {
                ticker,
                quantity,
                price,
                strategy,
            } => {
                portfolio::sell(&conn, &ticker, quantity, price, strategy.as_deref()).await?;
                eprintln!("Recorded sell: {} x {} @ {}", ticker, quantity, price);
            }
            PortfolioCommand::Positions => {
                let positions = portfolio::list_positions(&conn).await?;
                output::print_list_output(&positions, cli.format);
            }
            PortfolioCommand::Summary => {
                let sum = portfolio::summary(&conn).await?;
                output::print_output(&sum, cli.format);
            }
            PortfolioCommand::Trades { limit } => {
                let trades = portfolio::trade_history(&conn, limit).await?;
                output::print_list_output(&trades, cli.format);
            }
        },
        Command::History { limit } => {
            let evals = db::list_evaluations(&conn, limit).await?;
            output::print_list_output(&evals, cli.format);
        }
    }

    Ok(())
}
