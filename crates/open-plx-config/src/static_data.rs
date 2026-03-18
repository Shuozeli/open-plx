//! Converts StaticConfig YAML data to Arrow RecordBatches.

use crate::model::{DataSourceConfigYaml, DataSourceFile, StaticColumnYaml};
use anyhow::{bail, Result};
use arrow_array::{ArrayRef, Float64Array, Int64Array, RecordBatch, StringArray};
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

fn static_column_to_arrow(col: &StaticColumnYaml) -> Result<(Field, ArrayRef)> {
    match col.arrow_type.as_str() {
        "utf8" => {
            let values: Vec<String> = col
                .values
                .iter()
                .map(|v| match v {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => format!("{other:?}"),
                })
                .collect();
            let array = Arc::new(StringArray::from(values)) as ArrayRef;
            let field = Field::new(&col.name, DataType::Utf8, false);
            Ok((field, array))
        }
        "int64" => {
            let values: Vec<i64> = col
                .values
                .iter()
                .map(|v| match v {
                    serde_yaml::Value::Number(n) => n.as_i64().unwrap_or(0),
                    _ => 0,
                })
                .collect();
            let array = Arc::new(Int64Array::from(values)) as ArrayRef;
            let field = Field::new(&col.name, DataType::Int64, false);
            Ok((field, array))
        }
        "float64" => {
            let values: Vec<f64> = col
                .values
                .iter()
                .map(|v| match v {
                    serde_yaml::Value::Number(n) => n.as_f64().unwrap_or(0.0),
                    _ => 0.0,
                })
                .collect();
            let array = Arc::new(Float64Array::from(values)) as ArrayRef;
            let field = Field::new(&col.name, DataType::Float64, false);
            Ok((field, array))
        }
        other => bail!("unsupported arrow_type: {other}"),
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
}
