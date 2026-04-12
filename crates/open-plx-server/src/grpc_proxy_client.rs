//! GrpcProxyClient: calls upstream gRPC services and converts responses to Arrow RecordBatch.
//!
//! This enables widgets to fetch data from plain gRPC services (not just Flight SQL).
//! The client pools channels per service and uses a configurable endpoint resolver.

use arrow_array::{ArrayRef, RecordBatch};
use arrow_schema::{DataType, Field, Schema, TimeUnit};
use open_plx_core::pb::{GrpcProxyConfig, ParamValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Channel;

/// A pooled gRPC client for upstream gRPC services.
/// Converts gRPC responses to Arrow RecordBatch.
pub struct GrpcProxyClient {
    /// Channels per service name + endpoint (key is "service@endpoint").
    channels: Arc<RwLock<HashMap<String, Channel>>>,
    /// Endpoint resolver: service name -> address.
    endpoint_resolver: Arc<dyn Fn(&str) -> String + Send + Sync>,
}

impl GrpcProxyClient {
    /// Create a new GrpcProxyClient.
    /// `endpoint_resolver` is called with the service name to get the gRPC endpoint address
    /// when no explicit endpoint is configured in GrpcProxyConfig.
    pub fn new(endpoint_resolver: Arc<dyn Fn(&str) -> String + Send + Sync>) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            endpoint_resolver,
        }
    }

    /// Fetch data from an upstream gRPC service and return as Arrow RecordBatch.
    pub async fn fetch(
        &self,
        config: &GrpcProxyConfig,
        params: &HashMap<String, ParamValue>,
    ) -> Result<RecordBatch, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Get or create channel for this service
        // Use explicit endpoint from config if non-empty, otherwise use resolver
        let endpoint_override = if config.endpoint.is_empty() {
            None
        } else {
            Some(config.endpoint.as_str())
        };
        let channel = self.get_channel(&config.service, endpoint_override).await?;

        // 2. Build request by interpolating template with params
        let request_body = self.build_request(&config.request_template, params)?;

        // 3. Make the gRPC call (stubbed for generic services)
        let response_body = self
            .call_grpc(&config.service, &config.method, request_body, channel)
            .await?;

        // 4. Convert response to RecordBatch
        let batch = self.struct_to_record_batch(&response_body, &config.response_schema)?;

        Ok(batch)
    }

    /// Get or create a channel for the given service.
    /// Uses `endpoint_override` if provided, otherwise falls back to endpoint_resolver.
    async fn get_channel(
        &self,
        service_name: &str,
        endpoint_override: Option<&str>,
    ) -> Result<Channel, Box<dyn std::error::Error + Send + Sync>> {
        // Determine the address to use
        let addr = if let Some(ep) = endpoint_override {
            ep.to_string()
        } else {
            (self.endpoint_resolver)(service_name)
        };

        // Key channels by service@addr so different endpoints for same service are separate
        let key = format!("{}@{}", service_name, addr);

        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.get(&key) {
            return Ok(channel.clone());
        }

        let channel = Channel::from_shared(addr.clone())?.connect().await?;
        channels.insert(key, channel.clone());
        Ok(channel)
    }

    /// Build a prost_types::Struct request from the template, interpolating
    /// variable references and ${var} patterns using the provided params.
    fn build_request(
        &self,
        template: &HashMap<String, ParamValue>,
        params: &HashMap<String, ParamValue>,
    ) -> Result<prost_types::Struct, Box<dyn std::error::Error + Send + Sync>> {
        let mut fields = prost_types::Struct::default();
        for (key, value_template) in template {
            let value = self.resolve_param_value(value_template, params)?;
            fields.fields.insert(key.clone(), value);
        }
        Ok(fields)
    }

    /// Resolve a ParamValue to a prost_types::Value.
    /// Handles direct values and variable_ref types, with ${var} interpolation
    /// in string values.
    fn resolve_param_value(
        &self,
        param_value: &ParamValue,
        params: &HashMap<String, ParamValue>,
    ) -> Result<prost_types::Value, Box<dyn std::error::Error + Send + Sync>> {
        use open_plx_core::pb::param_value::Value;

        let prost_value = match param_value.value.as_ref() {
            // Variable reference: look up in params and resolve recursively
            Some(Value::VariableRef(var_name)) => {
                let resolved = params
                    .get(var_name)
                    .ok_or_else(|| format!("undefined variable: {}", var_name))?;
                return self.resolve_param_value(resolved, params);
            }

            // String value: may contain ${var} patterns for interpolation
            Some(Value::StringValue(s)) => {
                let interpolated = self.interpolate_str(s, params)?;
                prost_types::Value {
                    kind: Some(prost_types::value::Kind::StringValue(interpolated)),
                }
            }

            // Direct scalar values: convert directly
            Some(Value::IntValue(i)) => prost_types::Value {
                kind: Some(prost_types::value::Kind::NumberValue(*i as f64)),
            },
            Some(Value::DoubleValue(d)) => prost_types::Value {
                kind: Some(prost_types::value::Kind::NumberValue(*d)),
            },
            Some(Value::BoolValue(b)) => prost_types::Value {
                kind: Some(prost_types::value::Kind::BoolValue(*b)),
            },

            // String list: join with comma separator
            Some(Value::StringList(list)) => prost_types::Value {
                kind: Some(prost_types::value::Kind::StringValue(list.values.join(","))),
            },

            // Date range: use start|end format
            Some(Value::DateRange(range)) => prost_types::Value {
                kind: Some(prost_types::value::Kind::StringValue(format!(
                    "{}|{}",
                    range.start, range.end
                ))),
            },

            None => prost_types::Value {
                kind: Some(prost_types::value::Kind::StringValue(String::new())),
            },
        };

        Ok(prost_value)
    }

    /// Interpolate ${var} patterns in a string with values from params.
    fn interpolate_str(
        &self,
        template: &str,
        params: &HashMap<String, ParamValue>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let re = regex::Regex::new(r"\$\{(\w+)\}")?;
        let result = re.replace_all(template, |caps: &regex::Captures| {
            let var_name = &caps[1];
            params
                .get(var_name)
                .and_then(|pv| self.param_value_to_string(pv, params).ok())
                .unwrap_or_default()
        });
        Ok(result.to_string())
    }

    /// Convert a ParamValue to its string representation.
    fn param_value_to_string(
        &self,
        param_value: &ParamValue,
        params: &HashMap<String, ParamValue>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use open_plx_core::pb::param_value::Value;

        match param_value.value.as_ref() {
            Some(Value::VariableRef(var_name)) => {
                let resolved = params
                    .get(var_name)
                    .ok_or_else(|| format!("undefined variable: {}", var_name))?;
                self.param_value_to_string(resolved, params)
            }
            Some(Value::StringValue(s)) => Ok(s.clone()),
            Some(Value::IntValue(i)) => Ok(i.to_string()),
            Some(Value::DoubleValue(d)) => Ok(d.to_string()),
            Some(Value::BoolValue(b)) => Ok(b.to_string()),
            Some(Value::StringList(list)) => Ok(list.values.join(",")),
            Some(Value::DateRange(range)) => Ok(format!("{}|{}", range.start, range.end)),
            None => Ok(String::new()),
        }
    }

    /// Make the actual gRPC call to the upstream service.
    ///
    /// STUB: This requires generated gRPC client types for the specific service.
    /// For a generic proxy, we'd need either:
    /// a) Generated clients per service (defeats the purpose of a generic proxy)
    /// b) gRPC reflection (not always available)
    /// c) A convention where all services accept/return prost_types::Struct
    ///
    /// In production, implement with generated client types for your specific services.
    async fn call_grpc(
        &self,
        _service: &str,
        method: &str,
        _request: prost_types::Struct,
        _channel: Channel,
    ) -> Result<prost_types::Struct, Box<dyn std::error::Error + Send + Sync>> {
        // TODO(refactor): Implement actual gRPC call forwarding.
        // To implement properly:
        // 1. Use the service name to load generated client types (e.g., via tonic-build)
        // 2. Construct the proper request type from the prost_types::Struct
        // 3. Make the unary RPC call
        // 4. Convert the response back to prost_types::Struct for RecordBatch conversion
        Err(format!(
            "gRPC proxy call not implemented: service=_ method={}. \
             Implement with generated client types for production use.",
            method
        )
        .into())
    }

    /// Convert a prost_types::Struct response to an Arrow RecordBatch.
    /// Uses explicit response_schema if provided, otherwise infers schema from response.
    fn struct_to_record_batch(
        &self,
        response: &prost_types::Struct,
        schema: &Option<open_plx_core::pb::ResponseSchema>,
    ) -> Result<RecordBatch, Box<dyn std::error::Error + Send + Sync>> {
        let (fields, arrays) = if let Some(s) = schema {
            self.build_arrays_from_schema(response, s)?
        } else {
            self.infer_arrays_from_response(response)?
        };

        let arrow_schema = Schema::new(fields);
        RecordBatch::try_new(Arc::new(arrow_schema), arrays)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    /// Build Arrow arrays using an explicit response schema.
    fn build_arrays_from_schema(
        &self,
        response: &prost_types::Struct,
        schema: &open_plx_core::pb::ResponseSchema,
    ) -> Result<(Vec<Field>, Vec<ArrayRef>), Box<dyn std::error::Error + Send + Sync>> {
        use arrow_array::{BooleanArray, Float64Array, Int64Array, StringArray};

        let mut fields = Vec::with_capacity(schema.columns.len());
        let mut arrays = Vec::with_capacity(schema.columns.len());

        for col in &schema.columns {
            let data_type = match col.r#type {
                1 => DataType::Utf8,                                  // STRING
                2 => DataType::Int64,                                 // INT64
                3 => DataType::Float64,                               // DOUBLE
                4 => DataType::Boolean,                               // BOOL
                5 => DataType::Timestamp(TimeUnit::Nanosecond, None), // TIMESTAMP
                _ => DataType::Utf8,
            };

            // Get the field value from the response
            let field_value = response.fields.get(&col.field);
            let kind = field_value.and_then(|v| v.kind.as_ref());

            let arr: ArrayRef = match data_type {
                DataType::Utf8 => {
                    let s = match kind {
                        Some(prost_types::value::Kind::StringValue(sv)) => sv.clone(),
                        Some(prost_types::value::Kind::NumberValue(n)) => n.to_string(),
                        Some(prost_types::value::Kind::BoolValue(b)) => b.to_string(),
                        _ => String::new(),
                    };
                    let arr: StringArray = vec![s].into();
                    Arc::new(arr) as ArrayRef
                }
                DataType::Int64 => {
                    let n = match kind {
                        Some(prost_types::value::Kind::NumberValue(nv)) => *nv as i64,
                        Some(prost_types::value::Kind::StringValue(s)) => s.parse().unwrap_or(0),
                        _ => 0,
                    };
                    let arr: Int64Array = vec![n].into();
                    Arc::new(arr) as ArrayRef
                }
                DataType::Float64 => {
                    let n = match kind {
                        Some(prost_types::value::Kind::NumberValue(nv)) => *nv,
                        Some(prost_types::value::Kind::StringValue(s)) => s.parse().unwrap_or(0.0),
                        _ => 0.0,
                    };
                    let arr: Float64Array = vec![n].into();
                    Arc::new(arr) as ArrayRef
                }
                DataType::Boolean => {
                    let b = match kind {
                        Some(prost_types::value::Kind::BoolValue(bv)) => *bv,
                        Some(prost_types::value::Kind::StringValue(s)) => {
                            s.parse().unwrap_or(false)
                        }
                        _ => false,
                    };
                    let arr: BooleanArray = vec![b].into();
                    Arc::new(arr) as ArrayRef
                }
                DataType::Timestamp(..) => {
                    // Timestamp: expect numeric epoch in nanoseconds, or ISO 8601 string
                    let n = match kind {
                        Some(prost_types::value::Kind::StringValue(s)) => {
                            // Try to parse as integer (epoch nanos) or just pass as-is
                            s.parse::<i64>().unwrap_or(0)
                        }
                        Some(prost_types::value::Kind::NumberValue(nv)) => *nv as i64,
                        _ => 0,
                    };
                    let arr: arrow_array::TimestampNanosecondArray = vec![n].into();
                    Arc::new(arr) as ArrayRef
                }
                _ => {
                    let arr: StringArray = vec![""].into();
                    Arc::new(arr) as ArrayRef
                }
            };

            fields.push(Field::new(&col.field, data_type, false));
            arrays.push(arr);
        }

        Ok((fields, arrays))
    }

    /// Infer Arrow schema and arrays from a prost_types::Struct response.
    /// Each field in the struct becomes a column with a single value.
    fn infer_arrays_from_response(
        &self,
        response: &prost_types::Struct,
    ) -> Result<(Vec<Field>, Vec<ArrayRef>), Box<dyn std::error::Error + Send + Sync>> {
        use arrow_array::{BooleanArray, Float64Array, StringArray};

        let mut fields = Vec::new();
        let mut arrays = Vec::new();

        for (key, value) in &response.fields {
            let (field, arr): (Field, ArrayRef) = match value.kind.as_ref() {
                Some(prost_types::value::Kind::StringValue(s)) => {
                    let arr: StringArray = vec![s.as_str()].into();
                    (
                        Field::new(key, DataType::Utf8, false),
                        Arc::new(arr) as ArrayRef,
                    )
                }
                Some(prost_types::value::Kind::NumberValue(n)) => {
                    let arr: Float64Array = vec![*n].into();
                    (
                        Field::new(key, DataType::Float64, false),
                        Arc::new(arr) as ArrayRef,
                    )
                }
                Some(prost_types::value::Kind::BoolValue(b)) => {
                    let arr: BooleanArray = vec![*b].into();
                    (
                        Field::new(key, DataType::Boolean, false),
                        Arc::new(arr) as ArrayRef,
                    )
                }
                _ => {
                    let arr: StringArray = vec![""].into();
                    (
                        Field::new(key, DataType::Utf8, false),
                        Arc::new(arr) as ArrayRef,
                    )
                }
            };
            fields.push(field);
            arrays.push(arr);
        }

        Ok((fields, arrays))
    }
}
