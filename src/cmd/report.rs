use anyhow::Result;
use tokio_rusqlite::Connection;

use crate::db;

pub async fn run(conn: &Connection, date: Option<&str>) -> Result<String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let date_str = date.unwrap_or(&today);

    let evals = db::get_evaluations_for_date(conn, date_str).await?;

    let mut md = String::new();
    md.push_str(&format!("# Investment Report ({})\n\n", date_str));

    if evals.is_empty() {
        md.push_str("No evaluations found for this date.\n");
        return Ok(md);
    }

    // Summary table
    md.push_str("## Summary\n\n");
    md.push_str("| Ticker | Name | Decision | Score |\n");
    md.push_str("|--------|------|----------|-------|\n");
    for eval in &evals {
        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            eval.ticker, eval.name, eval.decision, eval.score
        ));
    }
    md.push('\n');

    // Buy recommendations
    let buys: Vec<_> = evals.iter().filter(|e| e.decision == "Buy").collect();
    if !buys.is_empty() {
        md.push_str("## Buy Recommendations\n\n");
        for eval in &buys {
            md.push_str(&format!("### {} ({})\n\n", eval.ticker, eval.name));
            md.push_str(&format!("- **Score**: {}\n", eval.score));
            md.push_str(&format!("- **Rationale**: {}\n", eval.rationale));

            // Fetch results for this stock
            if let Ok(Some(stock_id)) = db::get_stock_id(conn, &eval.ticker).await {
                let fetch_results = db::get_fetch_results_for_stock(conn, stock_id).await?;
                if !fetch_results.is_empty() {
                    md.push_str("\n**Recent Information:**\n\n");
                    for fr in fetch_results.iter().take(5) {
                        md.push_str(&format!("- [{}] {}", fr.category, fr.title));
                        if let Some(ref url) = fr.url {
                            md.push_str(&format!(" ([source]({}))", url));
                        }
                        md.push('\n');
                    }
                }
            }
            md.push('\n');
        }
    }

    // Hold
    let holds: Vec<_> = evals.iter().filter(|e| e.decision == "Hold").collect();
    if !holds.is_empty() {
        md.push_str("## Hold\n\n");
        for eval in &holds {
            md.push_str(&format!(
                "- **{} ({})** — Score: {} — {}\n",
                eval.ticker, eval.name, eval.score, eval.rationale
            ));
        }
        md.push('\n');
    }

    // Avoid
    let avoids: Vec<_> = evals.iter().filter(|e| e.decision == "Avoid").collect();
    if !avoids.is_empty() {
        md.push_str("## Avoid\n\n");
        for eval in &avoids {
            md.push_str(&format!(
                "- **{} ({})** — Score: {} — {}\n",
                eval.ticker, eval.name, eval.score, eval.rationale
            ));
        }
        md.push('\n');
    }

    // TA Summary section
    md.push_str("## Technical Analysis Details\n\n");
    for eval in &evals {
        if let Some(ref ta) = eval.ta_summary {
            md.push_str(&format!("### {} ({})\n\n", eval.ticker, eval.name));
            md.push_str(&format!("```json\n{}\n```\n\n", ta));
        }
    }

    md.push_str(&format!(
        "---\n*Generated at {}*\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    Ok(md)
}
