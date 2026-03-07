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
        println!("--- {} ({}) [{}] ---", self.ticker, self.name, self.status);
        println!("  Decision: {} (Score: {})", self.decision, self.score);
        println!("  Catalyst: {}", self.analysis.catalyst_check);
        println!("  Risk: {}", self.analysis.risk_assessment);
        println!("  Spec: {}", self.analysis.spec_compliance);
        if !self.execution_instruction.action.is_empty() {
            println!("  Action: {}", self.execution_instruction.action);
        }
        if !self.execution_instruction.reason_for_exit.is_empty() {
            println!("  Exit Reason: {}", self.execution_instruction.reason_for_exit);
        }
    }
}

impl HumanDisplay for crate::cmd::discover::DiscoverResult {
    fn print_human(&self) {
        if !self.added.is_empty() {
            println!("Added: {}", self.added.join(", "));
        }
        if !self.removed.is_empty() {
            println!("Removed: {}", self.removed.join(", "));
        }
        if !self.kept.is_empty() {
            println!("Kept (held): {}", self.kept.join(", "));
        }
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

impl HumanDisplay for crate::cmd::execute::ExecuteResult {
    fn print_human(&self) {
        if self.circuit_breaker_triggered {
            println!("!! CIRCUIT BREAKER TRIGGERED !!");
            for reason in &self.circuit_breaker_reasons {
                println!("  - {}", reason);
            }
            return;
        }
        if self.actions.is_empty() {
            println!("No actions to execute.");
        }
        for a in &self.actions {
            println!("[{}] {} ({}) - {}", a.action, a.ticker, a.name, a.detail);
        }
    }
}

impl HumanDisplay for crate::portfolio::PositionView {
    fn print_human(&self) {
        println!(
            "{:<10} {:<20} qty:{} avg:{} pnl:{}",
            self.ticker,
            self.name,
            self.quantity,
            self.avg_cost,
            self.unrealized_pnl
                .map(|p| format!("{}", p))
                .unwrap_or_else(|| "-".to_string())
        );
    }
}

impl HumanDisplay for crate::portfolio::PortfolioSummary {
    fn print_human(&self) {
        println!("Positions: {}", self.position_count);
        println!("Invested:  {}", self.total_invested);
        println!("Value:     {}", self.total_current_value);
        println!("P&L:       {}", self.total_unrealized_pnl);
        if let Some(pct) = self.total_unrealized_pnl_pct {
            println!("P&L %:     {}%", pct);
        }
    }
}

impl HumanDisplay for crate::db::WatchlistEvent {
    fn print_human(&self) {
        let action_display = match self.action.as_str() {
            "add" => "+add   ",
            "remove" => "-remove",
            "keep" => " keep  ",
            _ => &self.action,
        };
        println!(
            "  {} {:<6} {}  {}",
            &self.discovered_at[..10],
            self.ticker,
            action_display,
            self.reason.as_deref().unwrap_or("")
        );
    }
}

impl HumanDisplay for crate::db::StockInfo {
    fn print_human(&self) {
        println!(
            "  {:<6} {:<20} {}",
            self.ticker,
            self.name,
            self.sector.as_deref().unwrap_or("-"),
        );
    }
}

impl HumanDisplay for crate::db::TableStat {
    fn print_human(&self) {
        println!("  {:<25} {:>8} rows", self.table_name, self.row_count);
    }
}

impl HumanDisplay for crate::portfolio::TradeRecord {
    fn print_human(&self) {
        println!(
            "{:<10} {:<5} {} x {} @ {} {}",
            self.ticker,
            self.side,
            self.date,
            self.quantity,
            self.price,
            self.pnl
                .map(|p| format!("P&L: {}", p))
                .unwrap_or_default()
        );
    }
}
