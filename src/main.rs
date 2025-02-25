use analytics::Analytics;
use chrono::Utc;
use clap::{Parser, Subcommand};
use std::time::Duration;
use std::sync::Arc;

mod analytics;
mod blocker;
mod config;

#[derive(Parser)]
#[command(name = "Yarra")]
#[command(version = "0.1.0")]
#[command(about = "Focus tool to block distractions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a focus session
    Start {
        #[arg(short, long, default_value = "25")]
        focus: u64,
    },
    /// Block a specific website
    Block {
        #[arg(short, long)]
        site: String,
    },
    /// Unblock all websites
    Unblock,
    /// Show statistics
    Stats,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let analytics = Arc::new(Analytics::new()?);

    match cli.command {
        Commands::Start { focus } => {
            println!("Starting {}min focus session...", focus);
            let start_time = Utc::now();
            let analytics_clone = Arc::clone(&analytics);

            // Enable blocking and set up monitoring
            blocker::enable_blocking()?;
            
            // Monitor blocked attempts in background
            tokio::spawn(async move {
                let blocked_sites = config::load_blocked_sites().unwrap_or_default();
                for site in blocked_sites {
                    let _ = analytics_clone.log_blocked_attempt(&site);
                }
            });

            // Run the timer
            tokio::time::sleep(Duration::from_secs(focus * 60)).await;

            // Log session and cleanup
            analytics.log_session(start_time, focus as i64)?;
            blocker::disable_blocking()?;
            println!("Session completed! 🎉");
        }
        Commands::Block { site } => {
            config::add_blocked_site(&site)?;
            if blocker::enable_blocking().is_ok() {
                println!("Blocked {}", site);
            } else {
                println!("Site added to block list but failed to update hosts file. Try running with sudo/admin privileges.");
            }
        }
        Commands::Unblock => {
            if blocker::disable_blocking().is_ok() {
                println!("Unblocked all sites");
            } else {
                println!("Failed to unblock sites. Try running with sudo/admin privileges.");
            }
        }
        Commands::Stats => {
            let total_time = analytics.total_focus_time()?;
            let blocked_today = analytics.todays_blocked_attempts()?;
            println!("Total focus time: {} minutes", total_time);
            println!("Blocked attempts today: {}", blocked_today);
        }
    }

    Ok(())
}