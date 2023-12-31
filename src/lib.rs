use {
    relay_client::websocket,
    relay_client_helpers::create_ws_connect_options,
    relay_rpc::{
        auth::{
            ed25519_dalek::Keypair,
            rand::{rngs::StdRng, SeedableRng},
        },
        domain::ProjectId,
    },
    relay_ws_client::{RelayClientEvent, RelayConnectionHandler},
    std::sync::Arc,
    tokio::sync::mpsc::UnboundedReceiver,
    url::Url,
};

pub mod relay_client_helpers;
pub mod relay_ws_client;

pub async fn create_client(
    relay_url: Url,
    relay_project_id: ProjectId,
    notify_url: Url,
) -> (
    Arc<relay_client::websocket::Client>,
    UnboundedReceiver<RelayClientEvent>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let connection_handler = RelayConnectionHandler::new("notify-client", tx);
    let relay_ws_client = Arc::new(websocket::Client::new(connection_handler));

    let keypair = Keypair::generate(&mut StdRng::from_entropy());
    let opts = create_ws_connect_options(&keypair, relay_url, notify_url, relay_project_id);
    relay_ws_client.connect(&opts).await.unwrap();

    // Eat up the "connected" message
    _ = rx.recv().await.unwrap();

    (relay_ws_client, rx)
}
