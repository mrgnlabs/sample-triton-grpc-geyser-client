// This build script will run prior to crate compilation, and generate required rust code in the `target/debug/build/sample_client-<hash>` folder

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile(&["protos/geyser.proto"], &["protos"])
        .unwrap();

    Ok(())
}
