use std::io::Result;

fn main() -> Result<()> {
    let mut protos = vec!["foods.proto"];

    #[cfg(feature = "payloads")]
    protos.push("payloads.proto");

    prost_build::compile_protos(&protos, &["../../"])?;

    Ok(())
}
