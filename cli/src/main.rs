mod addon_cmd;
mod addon_loader;
mod kit;
#[allow(dead_code)]
mod addon_registry;
mod dirs;

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
#[allow(dead_code)]
mod lock;
mod migration;
mod output;
#[allow(dead_code)]
mod content_diff;
mod content_init;
#[allow(dead_code)]
mod content_install;
mod content_migration;
#[allow(dead_code)]
mod content_source;
mod claude_commands;
mod mcp_registration;
mod processkit_vocab;
mod reset;
mod runtime;
mod seed;
mod sync_perimeter;
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
    // Initialize addon definitions from YAML files.
    // Commands that don't need addons (completions, help) still work if this fails.
    if let Err(e) = addon_loader::init() {
        // Only fail for commands that actually need addons
        match &cli.command {
            cli::Commands::Completions { .. } => {} // doesn't need addons
            _ => {
                output::error(&format!("Failed to load addon definitions: {:#}", e));
                std::process::exit(1);
            }
        }
    }

    let config_path = &cli.config;
    let global_yes = cli.yes;

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
            addon_tool,
            processkit_source,
            processkit_version,
            processkit_branch,
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
                addon_tool,
                processkit_source,
                processkit_version,
                processkit_branch,
            },
        ),
        cli::Commands::Sync { no_cache, no_build } => container::cmd_sync(config_path, no_cache, no_build),
        cli::Commands::Start { layout } => {
            let config = crate::config::AiboxConfig::from_cli_option(config_path)?;
            let resolved_layout = layout
                .map(|l| l.to_string())
                .unwrap_or_else(|| config.customization.layout.to_string());
            container::cmd_start(config_path, &resolved_layout)
        }
        cli::Commands::Stop => container::cmd_stop(config_path),
        cli::Commands::Remove => container::cmd_remove(config_path),
        cli::Commands::Status { format } => container::cmd_status(config_path, format),
        cli::Commands::Doctor => doctor::cmd_doctor(config_path),
        cli::Commands::Completions { shell } => {
            let mut cmd = cli::Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
            Ok(())
        }
        cli::Commands::Update { check, dry_run } => {
            update::cmd_update(config_path, check, dry_run, global_yes)
        }
        cli::Commands::Env { action } => match action {
            cli::EnvAction::Create { name } => env::cmd_env_create(config_path, &name),
            cli::EnvAction::Switch { name, yes } => {
                env::cmd_env_switch(config_path, &name, yes || global_yes)
            }
            cli::EnvAction::List { format } => env::cmd_env_list(format),
            cli::EnvAction::Delete { name, yes } => env::cmd_env_delete(&name, yes || global_yes),
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
        } => reset::cmd_reset(config_path, no_backup, dry_run, yes || global_yes),
        cli::Commands::Uninstall { dry_run, purge } => {
            reset::cmd_uninstall(dry_run, purge, global_yes)
        }
        cli::Commands::Audit => audit::cmd_audit(config_path),
        cli::Commands::Audio { action } => match action {
            cli::AudioAction::Check { port } => audio::cmd_audio_check(port),
            cli::AudioAction::Setup { port } => audio::cmd_audio_setup(port),
        },
        cli::Commands::Addon { action } => match action {
            cli::AddonAction::List { format } => addon_cmd::cmd_addon_list(config_path, format),
            cli::AddonAction::Add { name, no_build } => addon_cmd::cmd_addon_add(config_path, &name, no_build),
            cli::AddonAction::Remove { name, no_build } => addon_cmd::cmd_addon_remove(config_path, &name, no_build),
            cli::AddonAction::Info { name, format } => addon_cmd::cmd_addon_info(&name, format),
        },
        cli::Commands::Migrate { action } => {
            let cwd = std::env::current_dir()?;
            match action {
                cli::MigrateAction::Continue => {
                    content_migration::cmd_migrate_continue(&cwd)
                }
                cli::MigrateAction::Start { id } => {
                    content_migration::cmd_migrate_start(&cwd, &id)
                }
                cli::MigrateAction::Apply { id } => {
                    content_migration::cmd_migrate_apply(&cwd, &id)
                }
                cli::MigrateAction::Reject { id, reason } => {
                    content_migration::cmd_migrate_reject(&cwd, &id, &reason)
                }
            }
        }
        cli::Commands::Kit { action } => match action {
            cli::KitAction::List { format } => kit::cmd_kit_list(config_path, format),
            cli::KitAction::Skill { action } => match action {
                cli::KitSkillAction::List { all, category, format } => {
                    kit::cmd_kit_skill_list(config_path, category.as_deref(), all, format)
                }
                cli::KitSkillAction::Categories { format } => {
                    kit::cmd_kit_skill_categories(config_path, format)
                }
                cli::KitSkillAction::Info { name, format } => {
                    kit::cmd_kit_skill_info(config_path, &name, format)
                }
                cli::KitSkillAction::Install { name } => {
                    kit::cmd_kit_skill_install(config_path, &name)
                }
                cli::KitSkillAction::Uninstall { name } => {
                    kit::cmd_kit_skill_uninstall(config_path, &name)
                }
            },
            cli::KitAction::Process { action } => match action {
                cli::KitProcessAction::List { all, format } => {
                    kit::cmd_kit_process_list(config_path, all, format)
                }
                cli::KitProcessAction::Info { name, format } => {
                    kit::cmd_kit_process_info(config_path, &name, format)
                }
            },
        },
    }
}
