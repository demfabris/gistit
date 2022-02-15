fn main() -> std::io::Result<()> {
    prost_build::compile_protos(&["src/payload.proto", "src/ipc.proto"], &["src"])?;
    Ok(())
}
