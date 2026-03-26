//! Converts StaticConfig YAML data to Arrow RecordBatches.

use crate::model::{DataSourceConfigYaml, DataSourceFile, StaticColumnYaml};
use anyhow::{Context, Result, bail};
use arrow_array::{ArrayRef, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use std::sync::Arc;

/// Build an Arrow RecordBatch from a static data source config.
pub fn static_config_to_record_batch(ds: &DataSourceFile) -> Result<RecordBatch> {
    let columns = match &ds.config {
        DataSourceConfigYaml::Static { columns } => columns,
        _ => bail!("expected static data source config"),
    };

    let mut fields: Vec<Field> = Vec::new();
    let mut arrays: Vec<ArrayRef> = Vec::new();

    for col in columns {
        let (field, array) = static_column_to_arrow(col)?;
        fields.push(field);
        arrays.push(array);
    }

    let schema = Arc::new(Schema::new(fields));
    let batch = RecordBatch::try_new(schema, arrays)?;
    Ok(batch)
}

/// Parse a YAML value as a string, failing on non-string types.
pub fn yaml_value_to_string(v: &serde_yaml::Value, col_name: &str, idx: usize) -> Result<String> {
    match v {
        serde_yaml::Value::String(s) => Ok(s.clone()),
        other => bail!(
            "column '{}' row {}: expected string, got {:?}",
            col_name,
            idx,
            other
        ),
    }
}

/// Parse a YAML value as i64, failing on non-numeric types.
pub fn yaml_value_to_i64(v: &serde_yaml::Value, col_name: &str, idx: usize) -> Result<i64> {
    match v {
        serde_yaml::Value::Number(n) => n.as_i64().with_context(|| {
            format!(
                "column '{}' row {}: number {:?} is not a valid i64",
                col_name, idx, n
            )
        }),
        other => bail!(
            "column '{}' row {}: expected number, got {:?}",
            col_name,
            idx,
            other
        ),
    }
}

/// Parse a YAML value as f64, failing on non-numeric types.
pub fn yaml_value_to_f64(v: &serde_yaml::Value, col_name: &str, idx: usize) -> Result<f64> {
    match v {
        serde_yaml::Value::Number(n) => n.as_f64().with_context(|| {
            format!(
                "column '{}' row {}: number {:?} is not a valid f64",
                col_name, idx, n
            )
        }),
        other => bail!(
            "column '{}' row {}: expected number, got {:?}",
            col_name,
            idx,
            other
        ),
    }
}

/// Parse a YAML value as bool, failing on non-boolean types.
pub fn yaml_value_to_bool(v: &serde_yaml::Value, col_name: &str, idx: usize) -> Result<bool> {
    match v {
        serde_yaml::Value::Bool(b) => Ok(*b),
        other => bail!(
            "column '{}' row {}: expected boolean, got {:?}",
            col_name,
            idx,
            other
        ),
    }
}

fn static_column_to_arrow(col: &StaticColumnYaml) -> Result<(Field, ArrayRef)> {
    match col.arrow_type.as_str() {
        "utf8" => {
            let values: Vec<String> = col
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| yaml_value_to_string(v, &col.name, i))
                .collect::<Result<Vec<_>>>()?;
            let array = Arc::new(StringArray::from(values)) as ArrayRef;
            let field = Field::new(&col.name, DataType::Utf8, false);
            Ok((field, array))
        }
        "int64" => {
            let values: Vec<i64> = col
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| yaml_value_to_i64(v, &col.name, i))
                .collect::<Result<Vec<_>>>()?;
            let array = Arc::new(Int64Array::from(values)) as ArrayRef;
            let field = Field::new(&col.name, DataType::Int64, false);
            Ok((field, array))
        }
        "float64" => {
            let values: Vec<f64> = col
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| yaml_value_to_f64(v, &col.name, i))
                .collect::<Result<Vec<_>>>()?;
            let array = Arc::new(Float64Array::from(values)) as ArrayRef;
            let field = Field::new(&col.name, DataType::Float64, false);
            Ok((field, array))
        }
        "boolean" => {
            let values: Vec<bool> = col
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| yaml_value_to_bool(v, &col.name, i))
                .collect::<Result<Vec<_>>>()?;
            let array = Arc::new(BooleanArray::from(values)) as ArrayRef;
            let field = Field::new(&col.name, DataType::Boolean, false);
            Ok((field, array))
        }
        "date32" | "timestamp_micros" => {
            // Date and timestamp values are stored as strings in YAML.
            // We store them as Utf8 in Arrow, consistent with how the proto
            // conversion stores them as string_values. Downstream consumers
            // (widget rendering) handle the string-to-date interpretation.
            let values: Vec<String> = col
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| yaml_value_to_string(v, &col.name, i))
                .collect::<Result<Vec<_>>>()?;
            let array = Arc::new(StringArray::from(values)) as ArrayRef;
            let field = Field::new(&col.name, DataType::Utf8, false);
            Ok((field, array))
        }
        other => bail!("unsupported arrow_type: '{other}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_config_to_record_batch() {
        let ds = DataSourceFile {
            name: "test".to_string(),
            display_name: "Test".to_string(),
            description: String::new(),
            config: DataSourceConfigYaml::Static {
                columns: vec![
                    StaticColumnYaml {
                        name: "label".to_string(),
                        arrow_type: "utf8".to_string(),
                        values: vec![
                            serde_yaml::Value::String("Q1".to_string()),
                            serde_yaml::Value::String("Q2".to_string()),
                        ],
                    },
                    StaticColumnYaml {
                        name: "value".to_string(),
                        arrow_type: "float64".to_string(),
                        values: vec![
                            serde_yaml::Value::Number(serde_yaml::Number::from(100.5_f64)),
                            serde_yaml::Value::Number(serde_yaml::Number::from(200.0_f64)),
                        ],
                    },
                ],
            },
        };

        let batch = static_config_to_record_batch(&ds).unwrap();
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 2);
        assert_eq!(batch.schema().field(0).name(), "label");
        assert_eq!(batch.schema().field(1).name(), "value");
    }

    #[test]
    fn test_type_mismatch_fails() {
        let ds = DataSourceFile {
            name: "bad".to_string(),
            display_name: "Bad".to_string(),
            description: String::new(),
            config: DataSourceConfigYaml::Static {
                columns: vec![StaticColumnYaml {
                    name: "count".to_string(),
                    arrow_type: "int64".to_string(),
                    values: vec![serde_yaml::Value::String("not_a_number".to_string())],
                }],
            },
        };

        let result = static_config_to_record_batch(&ds);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("expected number"),
            "error should mention type mismatch"
        );
    }
}
