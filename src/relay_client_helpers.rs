use {
    relay_client::ConnectionOptions,
    relay_rpc::{
        auth::{ed25519_dalek::Keypair, AuthToken},
        domain::ProjectId,
        user_agent::ValidUserAgent,
    },
    std::time::Duration,
    url::Url,
};

pub fn create_ws_connect_options(
    keypair: &Keypair,
    relay_url: Url,
    notify_url: Url,
    project_id: ProjectId,
) -> ConnectionOptions {
    create_connect_options(
        keypair,
        &relay_url,
        notify_url,
        project_id,
        Some(Duration::from_secs(60 * 60)),
    )
    .with_address(relay_url)
}

fn create_connect_options(
    keypair: &Keypair,
    relay_url: &Url,
    notify_url: Url,
    project_id: ProjectId,
    ttl: Option<Duration>,
) -> ConnectionOptions {
    let auth = AuthToken::new(notify_url.clone())
        .aud(relay_url.origin().ascii_serialization())
        .ttl(ttl)
        .as_jwt(keypair)
        .unwrap();

    let user_agent = relay_rpc::user_agent::UserAgent::ValidUserAgent(ValidUserAgent {
        protocol: relay_rpc::user_agent::Protocol {
            kind: relay_rpc::user_agent::ProtocolKind::WalletConnect,
            version: 2,
        },
        sdk: relay_rpc::user_agent::Sdk {
            language: relay_rpc::user_agent::SdkLanguage::Rust,
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        os: relay_rpc::user_agent::OsInfo {
            os_family: "ECS".into(),
            ua_family: None,
            version: None,
        },
        id: Some(relay_rpc::user_agent::Id {
            environment: relay_rpc::user_agent::Environment::Unknown("Notify Server".into()),
            host: Some(notify_url.to_string()),
        }),
    });

    ConnectionOptions::new(project_id, auth).with_user_agent(user_agent)
}
