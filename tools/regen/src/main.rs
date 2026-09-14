fn main() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let proto = format!("{root}/proto");
    let files = [format!("{proto}/qcloud.proto"), format!("{proto}/qconnect.proto")];
    let descriptors = protox::compile(files, [proto]).expect("protos compile");
    prost_build::Config::new()
        .out_dir(format!("{root}/src/proto"))
        .compile_fds(descriptors)
        .expect("prost output");
}
