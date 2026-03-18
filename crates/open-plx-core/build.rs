fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../proto";

    let protos = &[
        &format!("{proto_root}/open_plx/v1/widget_spec.proto") as &str,
        &format!("{proto_root}/open_plx/v1/dashboard.proto"),
        &format!("{proto_root}/open_plx/v1/data_source.proto"),
        &format!("{proto_root}/open_plx/v1/data.proto"),
    ];

    let descriptor_path = std::path::PathBuf::from(std::env::var("OUT_DIR")?)
        .join("open_plx_descriptor.bin");

    tonic_prost_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(protos, &[proto_root])?;

    Ok(())
}
