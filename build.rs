#[cfg(not(any(feature = "seabird-client", feature = "chat-ingest-client")))]
compile_error!(
    "You must enable at least one of the following features: seabird-client, chat-ingest-client"
);

#[allow(clippy::vec_init_then_push)]
fn main() {
    let mut protos: Vec<&str> = Vec::new();

    #[cfg(feature = "seabird-client")]
    protos.push("proto/seabird.proto");

    #[cfg(feature = "chat-ingest-client")]
    protos.push("proto/seabird_chat_ingest.proto");

    tonic_build::configure()
        .compile(&protos, &["proto/"])
        .unwrap();
}
