use {
    relay_client::websocket,
    relay_client_helpers::create_ws_connect_options,
    relay_rpc::{
        auth::{
            ed25519_dalek::Keypair,
            rand::{rngs::StdRng, SeedableRng},
        },
        domain::{ProjectId, Topic},
    },
    relay_ws_client::{RelayClientEvent, RelayConnectionHandler},
    std::{sync::Arc, time::Duration},
    tokio::sync::mpsc::UnboundedReceiver,
    tracing::info,
    tracing_subscriber::fmt::format::FmtSpan,
    url::Url,
};

mod relay_client_helpers;
mod relay_ws_client;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("DEBUG")
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(std::env::var("ANSI_LOGS").is_ok())
        .try_init()
        .ok();

    let relay_url: Url = "wss://staging.relay.walletconnect.com".parse().unwrap();
    let project_id: ProjectId = std::env::var("PROJECT_ID").unwrap().into();
    let server_url: Url = "http://localhost".parse().unwrap();

    let (client1, mut rx1) =
        create_client(relay_url.clone(), project_id.clone(), server_url.clone()).await;

    let (client2, mut rx2) = create_client(relay_url, project_id, server_url).await;

    let topic = Topic::generate();

    client1.subscribe(topic.clone()).await.unwrap();
    tokio::task::spawn({
        let topic = topic.clone();
        async move {
            loop {
                let event = rx1.recv().await.unwrap();
                let msg = match event {
                    RelayClientEvent::Message(msg) => msg,
                    e => panic!("Expected message, got {e:?}"),
                };

                assert_eq!(msg.tag, 1000);
                info!("responding");
                client1
                    .publish(
                        topic.clone(),
                        "Response from client 1",
                        1001,
                        Duration::from_secs(600),
                        false,
                    )
                    .await
                    .unwrap();
                info!("responded");
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    client2.subscribe(topic.clone()).await.unwrap();

    client2
        .publish(
            topic.clone(),
            "Request from client 2",
            1000,
            Duration::from_secs(600),
            false,
        )
        .await
        .unwrap();

    match tokio::time::timeout(Duration::from_secs(5), async {
        let event = rx2.recv().await.unwrap();
        let msg = match event {
            RelayClientEvent::Message(msg) => msg,
            e => panic!("Expected message, got {e:?}"),
        };

        assert_eq!(msg.tag, 1001);
        info!("received message from client 1: {}", msg.message);
    })
    .await
    {
        Ok(_) => info!("== ✅✅✅ PASS ✅✅✅ =="),
        Err(e) => panic!("== ❌❌❌ FAIL ❌❌❌ == {e:?}"),
    }
}

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
