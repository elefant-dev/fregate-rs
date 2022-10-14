use std::fs;

const OUT_FOLDER: &str = "src/proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if fs::metadata(OUT_FOLDER).is_ok() {
        fs::remove_dir_all(OUT_FOLDER)?;
    }
    fs::create_dir(OUT_FOLDER)?;

    tonic_build::configure()
        .include_file("mod.rs")
        .file_descriptor_set_path(format!("{OUT_FOLDER}/description.bin"))
        .build_server(true)
        .build_client(true)
        .out_dir(OUT_FOLDER)
        .compile(&["./proto/echo.proto", "./proto/hello.proto"], &["./proto"])?;

    Ok(())
}
