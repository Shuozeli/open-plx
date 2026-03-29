mod export;
mod import;
mod manifest;
mod output;
mod validate;

use anyhow::Result;
use clap::{Parser, Subcommand};
use output::{CommandOutput, DashboardEntry, ListOutput, OutputData};
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "plx", about = "open-plx dashboard import/export CLI")]
struct Cli {
    /// Output JSON to stdout (for agent consumption).
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List dashboards in the config.
    List {
        /// Path to open-plx.yaml config file.
        #[arg(long)]
        config: PathBuf,
    },

    /// Export dashboard(s) and their data sources to a bundle directory.
    Export {
        /// Dashboard name(s) to export (e.g., "dashboards/company-financials").
        #[arg()]
        dashboards: Vec<String>,

        /// Export all dashboards.
        #[arg(long)]
        all: bool,

        /// Path to open-plx.yaml config file.
        #[arg(long)]
        config: PathBuf,

        /// Output directory for the bundle.
        #[arg(long, short)]
        output: PathBuf,
    },

    /// Validate a bundle directory without importing.
    Validate {
        /// Path to the bundle directory.
        #[arg()]
        bundle: PathBuf,
    },

    /// Import a bundle directory into the config.
    Import {
        /// Path to the bundle directory.
        #[arg()]
        bundle: PathBuf,

        /// Path to open-plx.yaml config file.
        #[arg(long)]
        config: PathBuf,

        /// Overwrite existing files.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let json = cli.json;

    let result = run(cli);

    match result {
        Ok(output) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            if !output.success {
                std::process::exit(1);
            }
            Ok(())
        }
        Err(e) => {
            if json {
                let output = CommandOutput {
                    command: "error".to_string(),
                    success: false,
                    error: Some(format!("{e:#}")),
                    warnings: vec![],
                    data: OutputData::Validate(output::ValidateOutput {
                        valid: false,
                        dashboard_count: 0,
                        data_source_count: 0,
                        errors: vec![format!("{e:#}")],
                    }),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
                std::process::exit(1);
            }
            Err(e)
        }
    }
}

fn run(cli: Cli) -> Result<CommandOutput> {
    match cli.command {
        Command::List { config } => {
            let loader = open_plx_config::ConfigLoader::load(&config)?;
            let mut entries: Vec<DashboardEntry> = loader
                .dashboards
                .values()
                .map(|d| {
                    let mut ds_refs = BTreeSet::new();
                    for w in &d.widgets {
                        ds_refs.insert(w.data_source.data_source.clone());
                    }
                    DashboardEntry {
                        name: d.name.clone(),
                        title: d.title.clone(),
                        description: d.description.clone(),
                        widget_count: d.widgets.len(),
                        variable_count: d.variables.len(),
                        data_sources: ds_refs.into_iter().collect(),
                    }
                })
                .collect();
            entries.sort_by(|a, b| a.name.cmp(&b.name));

            if !cli.json {
                for e in &entries {
                    println!("{:40} {:30} ({} widgets, {} vars)",
                        e.name, e.title, e.widget_count, e.variable_count);
                }
                eprintln!("{} dashboard(s)", entries.len());
            }

            Ok(CommandOutput {
                command: "list".to_string(),
                success: true,
                error: None,
                warnings: vec![],
                data: OutputData::List(ListOutput { dashboards: entries }),
            })
        }
        Command::Export {
            dashboards,
            all,
            config,
            output,
        } => {
            let result = export::export(&config, &dashboards, all, &output)?;
            Ok(CommandOutput {
                command: "export".to_string(),
                success: true,
                error: None,
                warnings: vec![],
                data: OutputData::Export(result),
            })
        }
        Command::Validate { bundle } => {
            let result = validate::validate(&bundle)?;
            let success = result.valid;
            Ok(CommandOutput {
                command: "validate".to_string(),
                success,
                error: if success {
                    None
                } else {
                    Some(format!("{} validation error(s)", result.errors.len()))
                },
                warnings: vec![],
                data: OutputData::Validate(result),
            })
        }
        Command::Import {
            bundle,
            config,
            force,
        } => {
            let result = import::import(&bundle, &config, force)?;
            Ok(CommandOutput {
                command: "import".to_string(),
                success: true,
                error: None,
                warnings: vec![],
                data: OutputData::Import(result),
            })
        }
    }
}
