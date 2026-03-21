use anyhow::Result;
use clap::{Parser, Subcommand};
use kekekabu::output::OutputFormat;
use kekekabu::{cmd, config, db, jquants, output, spec, tachibana};

#[derive(Parser)]
#[command(name = "kabu", about = "JP stock investment CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Output format
    #[arg(long, short, global = true, default_value = "json")]
    format: OutputFormat,

    /// Use Tachibana demo environment (demo API + separate DB)
    #[arg(long, global = true)]
    demo: bool,
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
        /// Dry run (simulate without placing orders)
        #[arg(long, conflicts_with = "live")]
        dry_run: bool,
        /// Place real orders via Tachibana API
        #[arg(long, conflicts_with = "dry_run")]
        live: bool,
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
    /// Manage database (migrate, status, reset)
    #[command(subcommand)]
    Db(DbCommand),
    /// Manage launchd service (macOS)
    #[command(subcommand)]
    Service(ServiceCommand),
    /// Watch for fill notifications via Tachibana WebSocket (long-running)
    Watch,
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
enum DbCommand {
    /// Run pending database migrations
    Migrate,
    /// Show database info and migration history
    Status,
    /// Delete database and start fresh (interactive confirmation required)
    Reset {
        /// Skip interactive confirmation (dangerous!)
        #[arg(long)]
        force: bool,
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

    let env = if cli.demo {
        config::Environment::Demo
    } else {
        config::Environment::Production
    };

    // config subcommands don't need DB
    if let Command::Config(sub) = cli.command {
        match sub {
            ConfigCommand::Init { force } => cmd::config::init(force)?,
            ConfigCommand::Validate => cmd::config::validate()?,
        }
        return Ok(());
    }

    // db subcommands manage DB directly
    if let Command::Db(sub) = cli.command {
        match sub {
            DbCommand::Migrate => cmd::db::migrate(env, cli.format).await?,
            DbCommand::Status => cmd::db::status(env, cli.format).await?,
            DbCommand::Reset { force } => cmd::db::reset(env, force)?,
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

    let db = db::SqliteClient::open(env).await?;

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

    let mut config = config::AppConfig::load()?;

    // --demo flag overrides tachibana environment
    if cli.demo {
        let tc = config.tachibana.get_or_insert(config::TachibanaConfig {
            user_id: None,
            password: None,
            second_password: None,
            event_timeout_secs: 30,
            environment: config::Environment::Demo,
        });
        tc.environment = config::Environment::Demo;
    }

    // watch subcommand — long-running WebSocket fill monitor
    if let Command::Watch = cli.command {
        cmd::watch::run(&db, &config).await?;
        return Ok(());
    }

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
        Command::Config(_)
        | Command::Show(_)
        | Command::Db(_)
        | Command::Service(_)
        | Command::Watch
        | Command::Workflow(_) => {
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
        Command::Execute { dry_run, live } => {
            if !dry_run && !live {
                eprintln!("Usage: kabu execute --dry-run  (simulate)");
                eprintln!("       kabu execute --live     (place real orders)");
                eprintln!();
                eprintln!("You must specify either --dry-run or --live.");
                std::process::exit(1);
            }

            let investment_spec = spec::load_spec(&config.spec.path)?;
            let mut broker = if live {
                let tc_config = config.tachibana.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "[tachibana] config is required for --live execute. \
                         Set it in ~/.config/kabu/config.toml or use TACHIBANA_* env vars."
                    )
                })?;
                Some(tachibana::TachibanaClient::new(tc_config))
            } else {
                None
            };
            let broker_ref: Option<&mut dyn tachibana::BrokerClient> = broker
                .as_mut()
                .map(|b| b as &mut dyn tachibana::BrokerClient);
            let result =
                cmd::execute::run(&db, &config, &investment_spec, broker_ref, dry_run).await?;
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
