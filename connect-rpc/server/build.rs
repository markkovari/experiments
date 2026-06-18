fn main() {
    connectrpc_build::Config::new()
        .files(&["../proto/jobrunner/v1/jobrunner.proto"])
        .includes(&["../proto"])
        .include_file("_connectrpc.rs")
        .compile()
        .unwrap();
}
