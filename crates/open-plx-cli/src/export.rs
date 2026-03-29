use anyhow::{Context, Result, bail};
use open_plx_config::ConfigLoader;
use std::collections::BTreeSet;
use std::path::Path;

use crate::manifest::Manifest;
use crate::output::ExportOutput;

/// Export one or more dashboards and their referenced data sources to a bundle directory.
pub fn export(
    config_path: &Path,
    dashboard_names: &[String],
    all: bool,
    output_dir: &Path,
) -> Result<ExportOutput> {
    let loader = ConfigLoader::load(config_path)?;

    // Determine which dashboards to export.
    let names: Vec<&str> = if all {
        loader.dashboards.keys().map(|s| s.as_str()).collect()
    } else {
        for name in dashboard_names {
            if !loader.dashboards.contains_key(name) {
                let available: Vec<&str> =
                    loader.dashboards.keys().map(|s| s.as_str()).collect();
                bail!(
                    "dashboard '{}' not found. Available: {:?}",
                    name,
                    available
                );
            }
        }
        dashboard_names.iter().map(|s| s.as_str()).collect()
    };

    if names.is_empty() {
        bail!("no dashboards to export");
    }

    // Collect all referenced data source names.
    let mut ds_names = BTreeSet::new();
    for name in &names {
        let dashboard = &loader.dashboards[*name];
        for widget in &dashboard.widgets {
            ds_names.insert(widget.data_source.data_source.clone());
        }
    }

    // Create output directory structure.
    let dash_dir = output_dir.join("dashboards");
    let ds_dir = output_dir.join("data_sources");
    std::fs::create_dir_all(&dash_dir)
        .with_context(|| format!("failed to create {}", dash_dir.display()))?;
    std::fs::create_dir_all(&ds_dir)
        .with_context(|| format!("failed to create {}", ds_dir.display()))?;

    // Copy dashboard YAML files (raw copy to preserve formatting).
    let src_dash_dir = &loader.config.dashboards_dir;
    for name in &names {
        let file = find_yaml_by_name(src_dash_dir, name)
            .with_context(|| format!("could not find YAML file for dashboard '{}'", name))?;
        let dest = dash_dir.join(file.file_name().ok_or_else(|| anyhow::anyhow!("path has no file name: {}", file.display()))?);
        std::fs::copy(&file, &dest)
            .with_context(|| format!("failed to copy {} -> {}", file.display(), dest.display()))?;
        eprintln!("  exported dashboard: {}", name);
    }

    // Copy data source YAML files.
    let src_ds_dir = &loader.config.data_sources_dir;
    let mut exported_ds: Vec<String> = Vec::new();
    for ds_name in &ds_names {
        match find_yaml_by_name(src_ds_dir, ds_name) {
            Ok(file) => {
                let dest = ds_dir.join(file.file_name().ok_or_else(|| anyhow::anyhow!("path has no file name: {}", file.display()))?);
                std::fs::copy(&file, &dest).with_context(|| {
                    format!("failed to copy {} -> {}", file.display(), dest.display())
                })?;
                exported_ds.push(ds_name.clone());
                eprintln!("  exported data source: {}", ds_name);
            }
            Err(e) => {
                eprintln!("  warning: data source '{}' referenced but not found: {}", ds_name, e);
            }
        }
    }

    // Write manifest.
    let manifest = Manifest {
        version: 1,
        created: chrono::Utc::now().to_rfc3339(),
        dashboards: names.iter().map(|s| s.to_string()).collect(),
        data_sources: exported_ds.clone(),
    };
    let manifest_yaml = serde_yaml::to_string(&manifest)?;
    std::fs::write(output_dir.join("manifest.yaml"), manifest_yaml)?;

    eprintln!(
        "exported {} dashboard(s), {} data source(s) to {}",
        names.len(),
        exported_ds.len(),
        output_dir.display(),
    );

    Ok(ExportOutput {
        output_dir: output_dir.display().to_string(),
        dashboards: names.iter().map(|s| s.to_string()).collect(),
        data_sources: exported_ds,
    })
}

/// Find a YAML file in a directory whose parsed `name` field matches the given name.
fn find_yaml_by_name(dir: &Path, name: &str) -> Result<std::path::PathBuf> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
            let content = std::fs::read_to_string(&path)?;
            if content.contains(&format!("name: {}", name))
                || content.contains(&format!("name: \"{}\"", name))
            {
                return Ok(path);
            }
        }
    }
    bail!("no YAML file with name '{}' found in {}", name, dir.display())
}
