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

    let relay_url: Url = "wss://staging.relay.walletconnect.com".parse().unwrap();
    let project_id: ProjectId = std::env::var("PROJECT_ID").unwrap().into();
    let server_url: Url = "http://localhost".parse().unwrap();

    let (client1, mut rx1) =
        create_client(relay_url.clone(), project_id.clone(), server_url.clone()).await;

    let (client2, mut rx2) = create_client(relay_url, project_id, server_url).await;

    let topic = Topic::generate();
    info!("topic: {topic}");

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
