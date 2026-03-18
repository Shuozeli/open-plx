fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../proto";

    let protos = [
        format!("{proto_root}/open_plx/v1/widget_spec.proto"),
        format!("{proto_root}/open_plx/v1/dashboard.proto"),
        format!("{proto_root}/open_plx/v1/data_source.proto"),
        format!("{proto_root}/open_plx/v1/data.proto"),
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(
            std::path::PathBuf::from(std::env::var("OUT_DIR")?)
                .join("open_plx_descriptor.bin"),
        )
        .compile_protos(
            &protos.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            &[proto_root],
        )?;

    Ok(())
}
