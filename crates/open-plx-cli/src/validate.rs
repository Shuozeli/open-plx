use anyhow::{Context, Result, bail};
use open_plx_config::model::{DashboardFile, DataSourceFile};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::manifest::Manifest;
use crate::output::ValidateOutput;

/// Validate a bundle directory. Returns structured result.
pub fn validate(bundle_dir: &Path) -> Result<ValidateOutput> {
    let manifest_path = bundle_dir.join("manifest.yaml");
    if !manifest_path.exists() {
        bail!(
            "manifest.yaml not found in {}. Is this a valid bundle?",
            bundle_dir.display()
        );
    }

    let manifest: Manifest = serde_yaml::from_str(
        &std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?,
    )?;

    eprintln!("bundle version: {}", manifest.version);
    eprintln!("created: {}", manifest.created);

    let mut errors: Vec<String> = Vec::new();

    // Parse all dashboard YAML files.
    let dash_dir = bundle_dir.join("dashboards");
    let mut dashboards: HashMap<String, DashboardFile> = HashMap::new();
    if dash_dir.exists() {
        for entry in std::fs::read_dir(&dash_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
                match parse_yaml::<DashboardFile>(&path) {
                    Ok(d) => {
                        dashboards.insert(d.name.clone(), d);
                    }
                    Err(e) => errors.push(format!("invalid dashboard {}: {}", path.display(), e)),
                }
            }
        }
    }

    // Parse all data source YAML files.
    let ds_dir = bundle_dir.join("data_sources");
    let mut data_sources: HashMap<String, DataSourceFile> = HashMap::new();
    if ds_dir.exists() {
        for entry in std::fs::read_dir(&ds_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
                match parse_yaml::<DataSourceFile>(&path) {
                    Ok(ds) => {
                        data_sources.insert(ds.name.clone(), ds);
                    }
                    Err(e) => {
                        errors.push(format!("invalid data source {}: {}", path.display(), e));
                    }
                }
            }
        }
    }

    // Check manifest dashboards exist.
    for name in &manifest.dashboards {
        if !dashboards.contains_key(name) {
            errors.push(format!(
                "manifest references dashboard '{}' but no file found",
                name
            ));
        }
    }

    // Check all data source references resolve.
    let ds_names: HashSet<&str> = data_sources.keys().map(|s| s.as_str()).collect();
    for (dash_name, dashboard) in &dashboards {
        let var_names: HashSet<&str> = dashboard.variables.iter().map(|v| v.name.as_str()).collect();

        for widget in &dashboard.widgets {
            let ref_name = &widget.data_source.data_source;
            if !ds_names.contains(ref_name.as_str()) {
                errors.push(format!(
                    "dashboard '{}' widget '{}' references data source '{}' which is not in the bundle",
                    dash_name, widget.id, ref_name,
                ));
            }

            for ci in &widget.click_interactions {
                if !var_names.contains(ci.target_variable.as_str()) {
                    errors.push(format!(
                        "dashboard '{}' widget '{}' click_interaction targets variable '{}' not declared",
                        dash_name, widget.id, ci.target_variable,
                    ));
                }
            }

            for cond in &widget.visible_when {
                if !var_names.contains(cond.variable.as_str()) {
                    errors.push(format!(
                        "dashboard '{}' widget '{}' visible_when references variable '{}' not declared",
                        dash_name, widget.id, cond.variable,
                    ));
                }
            }
        }
    }

    let valid = errors.is_empty();
    let output = ValidateOutput {
        valid,
        dashboard_count: dashboards.len(),
        data_source_count: data_sources.len(),
        errors: errors.clone(),
    };

    if valid {
        eprintln!(
            "valid: {} dashboard(s), {} data source(s)",
            dashboards.len(),
            data_sources.len()
        );
    } else {
        for err in &errors {
            eprintln!("  error: {}", err);
        }
    }

    Ok(output)
}

fn parse_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))
}
