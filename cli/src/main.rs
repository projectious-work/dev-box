mod cli;
mod config;
mod container;
mod context;
mod doctor;
mod generate;
mod output;
mod runtime;
mod seed;
mod update;

use clap::Parser;
use tracing_subscriber::EnvFilter;

fn main() {
    let cli = cli::Cli::parse();

    // Initialize tracing
    let filter = EnvFilter::try_new(&cli.log_level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let result = dispatch(cli);

    if let Err(e) = result {
        output::error(&format!("{:#}", e));
        std::process::exit(1);
    }
}

fn dispatch(cli: cli::Cli) -> anyhow::Result<()> {
    let config_path = &cli.config;

    match cli.command {
        cli::Commands::Init { name, image, process } => {
            container::cmd_init(config_path, name, image, process)
        }
        cli::Commands::Generate => container::cmd_generate(config_path),
        cli::Commands::Build { no_cache } => container::cmd_build(config_path, no_cache),
        cli::Commands::Start => container::cmd_start(config_path),
        cli::Commands::Stop => container::cmd_stop(config_path),
        cli::Commands::Attach => container::cmd_attach(config_path),
        cli::Commands::Status => container::cmd_status(config_path),
        cli::Commands::Doctor => doctor::cmd_doctor(config_path),
        cli::Commands::Update { check } => update::cmd_update(config_path, check),
    }
}
