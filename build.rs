fn main() {
    tonic_build::configure()
        .compile(
            &["proto/seabird.proto", "proto/seabird_chat_ingest.proto"],
            &["proto/"],
        )
        .unwrap();
}
