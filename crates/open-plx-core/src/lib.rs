/// Generated protobuf types and gRPC service definitions for open-plx.
///
/// All types are generated from proto files in `proto/open_plx/v1/`.
/// Do not hand-write types that duplicate proto definitions.
pub mod proto {
    pub mod open_plx {
        pub mod v1 {
            tonic::include_proto!("open_plx.v1");
        }
    }
}

// Re-export for convenience.
pub use proto::open_plx::v1 as pb;

/// File descriptor set for gRPC reflection.
pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/open_plx_descriptor.bin"));
