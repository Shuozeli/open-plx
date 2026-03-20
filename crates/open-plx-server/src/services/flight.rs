//! Arrow Flight service that serves widget data from data sources.

use crate::state::AppState;
use arrow_flight::{
    encode::FlightDataEncoderBuilder,
    flight_service_server::FlightService, Action, ActionType, Criteria, Empty,
    FlightData, FlightDescriptor, FlightEndpoint, FlightInfo, HandshakeRequest,
    HandshakeResponse, PollInfo, PutResult, SchemaResult, Ticket,
};
use futures::stream::BoxStream;
use futures::StreamExt;
use open_plx_core::pb::WidgetDataRequest;
use prost::Message;
use std::sync::Arc;
use tonic::{Request, Response, Status, Streaming};

pub struct FlightServiceImpl {
    state: Arc<AppState>,
}

impl FlightServiceImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl FlightService for FlightServiceImpl {
    type HandshakeStream = BoxStream<'static, Result<HandshakeResponse, Status>>;
    type ListFlightsStream = BoxStream<'static, Result<FlightInfo, Status>>;
    type DoGetStream = BoxStream<'static, Result<FlightData, Status>>;
    type DoPutStream = BoxStream<'static, Result<PutResult, Status>>;
    type DoActionStream = BoxStream<'static, Result<arrow_flight::Result, Status>>;
    type ListActionsStream = BoxStream<'static, Result<ActionType, Status>>;
    type DoExchangeStream = BoxStream<'static, Result<FlightData, Status>>;

    async fn handshake(
        &self,
        _request: Request<Streaming<HandshakeRequest>>,
    ) -> Result<Response<Self::HandshakeStream>, Status> {
        Err(Status::unimplemented("handshake not needed"))
    }

    async fn list_flights(
        &self,
        _request: Request<Criteria>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented("list_flights not yet implemented"))
    }

    async fn get_flight_info(
        &self,
        request: Request<FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        let descriptor = request.into_inner();
        let widget_req = WidgetDataRequest::decode(descriptor.cmd.as_ref())
            .map_err(|e| Status::invalid_argument(format!("invalid WidgetDataRequest: {e}")))?;

        tracing::debug!(
            "get_flight_info: dashboard={}, widget={}",
            widget_req.dashboard,
            widget_req.widget_id
        );

        let batch = self.state.resolve_widget_data(&widget_req).await?;
        let schema = batch.schema();

        let ticket = Ticket {
            ticket: descriptor.cmd.clone(),
        };

        let flight_info = FlightInfo::new()
            .try_with_schema(&schema)
            .map_err(|e| Status::internal(format!("schema error: {e}")))?
            .with_endpoint(FlightEndpoint::new().with_ticket(ticket))
            .with_total_records(batch.num_rows() as i64);

        Ok(Response::new(flight_info))
    }

    async fn poll_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<PollInfo>, Status> {
        Err(Status::unimplemented("poll_flight_info not supported"))
    }

    async fn get_schema(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<SchemaResult>, Status> {
        Err(Status::unimplemented("use get_flight_info instead"))
    }

    async fn do_get(
        &self,
        request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        let ticket = request.into_inner();
        let widget_req = WidgetDataRequest::decode(ticket.ticket.as_ref())
            .map_err(|e| Status::invalid_argument(format!("invalid ticket: {e}")))?;

        tracing::debug!(
            "do_get: dashboard={}, widget={}",
            widget_req.dashboard,
            widget_req.widget_id
        );

        let batch = self.state.resolve_widget_data(&widget_req).await?;
        let schema = batch.schema();

        // Use FlightDataEncoderBuilder to stream schema + batches
        let flight_stream = FlightDataEncoderBuilder::new()
            .with_schema(schema)
            .build(futures::stream::once(async { Ok(batch) }))
            .map(|result| result.map_err(|e| Status::internal(format!("encoding error: {e}"))))
            .boxed();

        Ok(Response::new(flight_stream))
    }

    async fn do_put(
        &self,
        _request: Request<Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        Err(Status::unimplemented("do_put not supported"))
    }

    async fn do_action(
        &self,
        _request: Request<Action>,
    ) -> Result<Response<Self::DoActionStream>, Status> {
        Err(Status::unimplemented("do_action not supported"))
    }

    async fn list_actions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented("list_actions not supported"))
    }

    async fn do_exchange(
        &self,
        _request: Request<Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        Err(Status::unimplemented("do_exchange not supported"))
    }
}
