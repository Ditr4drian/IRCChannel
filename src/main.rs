//! IRCChannel binary entrypoint.

mod runtime;

use tracing_subscriber::EnvFilter;

use ircchannel::Config;

#[tokio::main]
async fn main() {
    // Logs go to a file so the TUI owns stderr/stdout.
    let _ = init_tracing();

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("config error: {e}");
            std::process::exit(2);
        }
    };

    if let Err(e) = runtime::run(config).await {
        // Tui dropped — terminal restored
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn init_tracing() -> anyhow::Result<()> {
    use std::fs::OpenOptions;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if let Some(dir) = directories::ProjectDirs::from("mx", "ditra", "ircchannel") {
        let log_dir = dir.data_local_dir();
        std::fs::create_dir_all(log_dir)?;
        let path = log_dir.join("ircchannel.log");
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(file)
            .with_ansi(false)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::sink)
            .init();
    }
    Ok(())
}
