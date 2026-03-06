use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Json,
    Human,
}

pub fn print_output<T: Serialize + HumanDisplay>(data: &T, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
        }
        OutputFormat::Human => {
            data.print_human();
        }
    }
}

pub fn print_list_output<T: Serialize + HumanDisplay>(data: &[T], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
        }
        OutputFormat::Human => {
            if data.is_empty() {
                println!("(empty)");
            } else {
                for item in data {
                    item.print_human();
                    println!();
                }
            }
        }
    }
}

pub trait HumanDisplay {
    fn print_human(&self);
}

impl HumanDisplay for crate::db::WatchlistItem {
    fn print_human(&self) {
        println!(
            "{:<10} {:<20} {:<15} {}",
            self.ticker,
            self.name,
            self.sector.as_deref().unwrap_or("-"),
            self.notes.as_deref().unwrap_or("")
        );
    }
}

impl HumanDisplay for crate::cmd::scan::ScanResult {
    fn print_human(&self) {
        println!("--- {} ({}) ---", self.ticker, self.name);
        if let Some(price) = self.latest_close {
            println!("  Close: {:.0}", price);
        }
        println!("  Data points: {}", self.data_points);
        if let Some(ref ta) = self.indicators {
            if !ta.signals.is_empty() {
                println!("  Signals: {}", ta.signals.join(", "));
            }
            for (k, v) in &ta.latest {
                println!("  {}: {:.2}", k, v);
            }
        }
    }
}

impl HumanDisplay for crate::cmd::eval::EvalResult {
    fn print_human(&self) {
        println!("--- {} ({}) ---", self.ticker, self.name);
        println!("  Decision: {} (Score: {})", self.decision, self.score);
        println!("  Summary: {}", self.rationale.summary);
        println!("  Technical: {}", self.rationale.technical);
        println!("  Risks: {}", self.rationale.risks);
    }
}

impl HumanDisplay for crate::cmd::fetch::FetchSummary {
    fn print_human(&self) {
        println!(
            "{:<10} {:<20} {} items saved",
            self.ticker, self.name, self.items_saved
        );
    }
}

impl HumanDisplay for crate::db::Evaluation {
    fn print_human(&self) {
        println!(
            "[{}] {} ({}) - {} (Score: {})",
            self.evaluated_at, self.ticker, self.name, self.decision, self.score
        );
        println!("  {}", self.rationale);
    }
}
