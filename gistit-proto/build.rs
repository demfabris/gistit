fn main() -> std::io::Result<()> {
    prost_build::compile_protos(&["src/gistit.proto", "src/ipc.proto"], &["src"])?;
    Ok(())
}
