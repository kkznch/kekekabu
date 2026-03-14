use anyhow::Result;
use clap::{Parser, Subcommand};
use kekekabu::output::OutputFormat;
use kekekabu::{cmd, config, db, jquants, output};

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
        /// Refresh the stock master data from J-Quants API before scanning
        #[arg(long)]
        refresh_master: bool,
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
    /// Manage launchd service (macOS)
    #[command(subcommand)]
    Service(ServiceCommand),
    /// Run the full pipeline as a single process with per-stock error isolation
    #[command(subcommand)]
    Workflow(WorkflowCommand),
}

#[derive(Subcommand)]
enum WorkflowCommand {
    /// Run discover → scan → fetch → eval pipeline
    Run {
        /// Steps to skip (discover, scan, fetch)
        #[arg(long)]
        skip: Vec<String>,
    },
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
    /// LLM prompt/response logs
    LlmLogs {
        /// Number of logs to show
        #[arg(long, default_value = "20")]
        limit: i64,
        /// Filter by ticker
        #[arg(long)]
        ticker: Option<String>,
    },
    /// Order history
    Orders {
        /// Number of orders to show
        #[arg(long, default_value = "20")]
        limit: i64,
        /// Filter by status (pending, filled, expired, rejected, cancelled)
        #[arg(long)]
        status: Option<String>,
    },
}

#[derive(Subcommand)]
enum ServiceCommand {
    /// Install launchd plist to ~/Library/LaunchAgents/
    Install,
    /// Remove launchd plist
    Uninstall,
    /// Start the launchd service
    Start,
    /// Stop the launchd service
    Stop,
    /// Show service status
    Status,
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

    // service subcommands don't need DB or config
    if let Command::Service(sub) = cli.command {
        let rt = cmd::service::RealRuntime;
        match sub {
            ServiceCommand::Install => cmd::service::install(&rt)?,
            ServiceCommand::Uninstall => cmd::service::uninstall(&rt)?,
            ServiceCommand::Start => cmd::service::start(&rt)?,
            ServiceCommand::Stop => cmd::service::stop(&rt)?,
            ServiceCommand::Status => cmd::service::status(&rt)?,
        }
        return Ok(());
    }

    let db = db::SqliteClient::open().await?;

    // show subcommands don't need config
    if let Command::Show(sub) = cli.command {
        let format = cli.format;
        match sub {
            ShowCommand::Watchlist => cmd::show::watchlist(&db, format).await?,
            ShowCommand::Events { ticker } => {
                cmd::show::events(&db, ticker.as_deref(), format).await?
            }
            ShowCommand::Positions => cmd::show::positions(&db, format).await?,
            ShowCommand::Evaluations { limit } => {
                cmd::show::evaluations(&db, limit, format).await?
            }
            ShowCommand::Stocks => cmd::show::stocks(&db, format).await?,
            ShowCommand::Tables => cmd::show::tables(&db, format).await?,
            ShowCommand::Summary => cmd::show::summary(&db, format).await?,
            ShowCommand::Trades { limit } => cmd::show::trades(&db, limit, format).await?,
            ShowCommand::LlmLogs { limit, ticker } => {
                cmd::show::llm_logs(&db, limit, ticker.as_deref(), format).await?
            }
            ShowCommand::Orders { limit, status } => {
                cmd::show::orders(&db, limit, status.as_deref(), format).await?
            }
        }
        return Ok(());
    }

    let config = config::AppConfig::load()?;

    // workflow subcommand — single-process pipeline
    if let Command::Workflow(sub) = cli.command {
        match sub {
            WorkflowCommand::Run { skip } => {
                let api_key =
                    config::AppConfig::require_key(&config.api.jquants_api_key, "JQUANTS_API_KEY")?;
                let stock_api = jquants::JQuantsClient::new(api_key);
                let report = cmd::workflow::run(&db, &config, &stock_api, &skip).await?;
                output::print_output(&report, cli.format);
            }
        }
        return Ok(());
    }

    match cli.command {
        Command::Config(_) | Command::Show(_) | Command::Service(_) | Command::Workflow(_) => {
            unreachable!()
        }
        Command::Discover => {
            let result = cmd::discover::run(&db, &config).await?;
            output::print_output(&result, cli.format);
        }
        Command::Scan {
            days,
            refresh_master,
        } => {
            let api_key =
                config::AppConfig::require_key(&config.api.jquants_api_key, "JQUANTS_API_KEY")?;
            let stock_api = jquants::JQuantsClient::new(api_key);
            let results = cmd::scan::run(&db, &config, &stock_api, days, refresh_master).await?;
            output::print_list_output(&results, cli.format);
        }
        Command::Fetch { tickers } => {
            let results = cmd::fetch::run(&db, &config, &tickers).await?;
            output::print_list_output(&results, cli.format);
        }
        Command::Eval { tickers } => {
            let results = cmd::eval::run(&db, &config, &tickers).await?;
            output::print_list_output(&results, cli.format);
        }
        Command::Execute { dry_run } => {
            let result = cmd::execute::run(&db, &config, dry_run).await?;
            output::print_output(&result, cli.format);
        }
        Command::Report { date, output: out } => {
            let md = cmd::report::run(&db, date.as_deref()).await?;
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
