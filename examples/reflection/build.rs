fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .file_descriptor_set_path("src/echo_descriptor.bin")
        .build_client(false)
        .build_server(true)
        .out_dir("src")
        .compile(&["echo.proto"], &["proto"])?;
    Ok(())
}
