mod addon_cmd;
#[allow(dead_code)]
mod addon_registry;

mod addons;
mod audit;
mod audio;
mod cli;
mod config;
mod container;
mod context;
mod doctor;
mod env;
mod generate;
mod migration;
mod output;
#[allow(dead_code)]
mod process_registry;
mod reset;
mod runtime;
mod seed;
mod skill_cmd;
mod themes;
mod update;

use clap::{CommandFactory, Parser};
use tracing_subscriber::EnvFilter;

fn main() {
    let cli = cli::Cli::parse();

    let filter = EnvFilter::try_new(&cli.log_level).unwrap_or_else(|_| EnvFilter::new("info"));

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
        cli::Commands::Init {
            name,
            base,
            process,
            ai,
            user,
            theme,
            prompt,
            addons,
        } => container::cmd_init(
            config_path,
            container::InitParams {
                name,
                base,
                process,
                ai,
                user,
                theme,
                prompt,
                addons,
            },
        ),
        cli::Commands::Sync { no_cache } => container::cmd_sync(config_path, no_cache),
        cli::Commands::Start { layout } => container::cmd_start(config_path, &layout.to_string()),
        cli::Commands::Stop => container::cmd_stop(config_path),
        cli::Commands::Remove => container::cmd_remove(config_path),
        cli::Commands::Status => container::cmd_status(config_path),
        cli::Commands::Doctor => doctor::cmd_doctor(config_path),
        cli::Commands::Completions { shell } => {
            let mut cmd = cli::Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
            Ok(())
        }
        cli::Commands::Update { check, dry_run } => {
            update::cmd_update(config_path, check, dry_run)
        }
        cli::Commands::Env { action } => match action {
            cli::EnvAction::Create { name } => env::cmd_env_create(config_path, &name),
            cli::EnvAction::Switch { name, yes } => env::cmd_env_switch(config_path, &name, yes),
            cli::EnvAction::List => env::cmd_env_list(),
            cli::EnvAction::Delete { name, yes } => env::cmd_env_delete(&name, yes),
            cli::EnvAction::Status => env::cmd_env_status(config_path),
        },
        cli::Commands::Backup {
            output_dir,
            dry_run,
        } => reset::cmd_backup(config_path, output_dir, dry_run),
        cli::Commands::Reset {
            no_backup,
            dry_run,
            yes,
        } => reset::cmd_reset(config_path, no_backup, dry_run, yes),
        cli::Commands::Audit => audit::cmd_audit(config_path),
        cli::Commands::Audio { action } => match action {
            cli::AudioAction::Check { port } => audio::cmd_audio_check(port),
            cli::AudioAction::Setup { port } => audio::cmd_audio_setup(port),
        },
        cli::Commands::Addon { action } => match action {
            cli::AddonAction::List => addon_cmd::cmd_addon_list(config_path),
            cli::AddonAction::Add { name } => addon_cmd::cmd_addon_add(config_path, &name),
            cli::AddonAction::Remove { name } => addon_cmd::cmd_addon_remove(config_path, &name),
            cli::AddonAction::Info { name } => addon_cmd::cmd_addon_info(&name),
        },
        cli::Commands::Skill { action } => match action {
            cli::SkillAction::List => skill_cmd::cmd_skill_list(config_path),
            cli::SkillAction::Add { name } => skill_cmd::cmd_skill_add(config_path, &name),
            cli::SkillAction::Remove { name } => skill_cmd::cmd_skill_remove(config_path, &name),
            cli::SkillAction::Info { name } => skill_cmd::cmd_skill_info(&name),
        },
    }
}
