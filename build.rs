fn main() -> Result<(), Box<dyn std::error::Error>> {
	tonic_build::compile_protos("src/proto/adminctrl.proto")?;
	tonic_build::compile_protos("src/proto/civkitservice.proto")?;
	Ok(())
}
