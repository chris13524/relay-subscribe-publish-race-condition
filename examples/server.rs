use {
    relay_rpc::domain::{ProjectId, Topic},
    relay_subscribe_publish_race_condition::{create_client, relay_ws_client::RelayClientEvent},
    std::time::Duration,
    tracing::info,
    tracing_subscriber::fmt::format::FmtSpan,
    url::Url,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("DEBUG")
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(std::env::var("ANSI_LOGS").is_ok())
        .try_init()
        .ok();

    let relay_url: Url = "wss://relay.walletconnect.com".parse().unwrap();
    let project_id: ProjectId = std::env::var("PROJECT_ID").unwrap().into();
    let server_url: Url = "http://localhost".parse().unwrap();

    let (client, mut rx) =
        create_client(relay_url.clone(), project_id.clone(), server_url.clone()).await;

    let topic: Topic = "b964e2d7a9b8d3684df79f880e3973e543c53811d45b7d8e331de01cf6e98211".into();

    client.subscribe(topic.clone()).await.unwrap();
    loop {
        let event = rx.recv().await.unwrap();
        let msg = match event {
            RelayClientEvent::Message(msg) => msg,
            e => panic!("Expected message, got {e:?}"),
        };

        assert_eq!(msg.tag, 1000);
        info!("responding");
        client
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
