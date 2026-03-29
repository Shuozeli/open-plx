use anyhow::{Context, Result, bail};
use std::path::Path;

use crate::manifest::Manifest;
use crate::output::ImportOutput;

/// Import a bundle directory into the config.
pub fn import(bundle_dir: &Path, config_path: &Path, force: bool) -> Result<ImportOutput> {
    // First validate the bundle.
    let validation = crate::validate::validate(bundle_dir)?;
    if !validation.valid {
        bail!("bundle validation failed with {} error(s)", validation.errors.len());
    }

    // Load server config to find target directories.
    let config_str = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let config: open_plx_config::OpenPlxConfig = serde_yaml::from_str(&config_str)
        .with_context(|| format!("failed to parse {}", config_path.display()))?;

    let manifest: Manifest = serde_yaml::from_str(&std::fs::read_to_string(
        bundle_dir.join("manifest.yaml"),
    )?)?;

    let mut imported: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    // Import dashboards.
    let src_dash = bundle_dir.join("dashboards");
    if src_dash.exists() {
        for entry in std::fs::read_dir(&src_dash)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
                let dest = config.dashboards_dir.join(path.file_name().ok_or_else(|| anyhow::anyhow!("path has no file name: {}", path.display()))?);
                if dest.exists() && !force {
                    let msg = format!("{} (already exists)", dest.display());
                    eprintln!("  skipped: {}", msg);
                    skipped.push(msg);
                } else {
                    std::fs::copy(&path, &dest)?;
                    let msg = dest.display().to_string();
                    eprintln!("  imported dashboard: {}", msg);
                    imported.push(msg);
                }
            }
        }
    }

    // Import data sources.
    let src_ds = bundle_dir.join("data_sources");
    if src_ds.exists() {
        for entry in std::fs::read_dir(&src_ds)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
                let dest = config.data_sources_dir.join(path.file_name().ok_or_else(|| anyhow::anyhow!("path has no file name: {}", path.display()))?);
                if dest.exists() && !force {
                    let msg = format!("{} (already exists)", dest.display());
                    eprintln!("  skipped: {}", msg);
                    skipped.push(msg);
                } else {
                    std::fs::copy(&path, &dest)?;
                    let msg = dest.display().to_string();
                    eprintln!("  imported data source: {}", msg);
                    imported.push(msg);
                }
            }
        }
    }

    eprintln!(
        "imported {} file(s) from bundle '{}' ({} skipped)",
        imported.len(),
        manifest.dashboards.join(", "),
        skipped.len(),
    );

    if !skipped.is_empty() && !force {
        bail!("{} file(s) skipped due to conflicts. Use --force to overwrite.", skipped.len());
    }

    Ok(ImportOutput { imported, skipped })
}
