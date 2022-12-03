mod protos;

use futures::StreamExt;
use futures_util::stream;
use log::{debug, info, warn};
use protos::geyser::{
    geyser_client::GeyserClient, subscribe_update::UpdateOneof, SubscribeRequest,
    SubscribeRequestFilterAccounts, SubscribeRequestFilterBlocks, SubscribeRequestFilterSlots,
    SubscribeRequestFilterTransactions,
};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tonic::{
    codegen::InterceptedService,
    service::Interceptor,
    transport::{Channel, ClientTlsConfig, Endpoint},
    Request, Status,
};

// Constants
const AUTHENTICATION_HEADER: &str = "x-token";
const HTTPS_SCHEME: &str = "https";

// Parameters (generally provided through app-specific configs)
const SERVER_URL: &str = "<your geyser url>";
const AUTHENTICATION_TOKEN: &str = "<your auth token>";
const PYTH_SOLUSDC_FEED: &str = "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG"; // Sample account to follow

// Struct in charge of injecting the credentials header
// Needed to avoid the type mess that ensues when using a closure directly in `GeyserClient::with_interceptor`
struct RequestInterceptor {
    auth_token: String,
}

impl Interceptor for RequestInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request
            .metadata_mut()
            .insert(AUTHENTICATION_HEADER, self.auth_token.parse().unwrap());
        Ok(request)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let url = SERVER_URL.to_string();
    let mut endpoint = Endpoint::from_shared(url.clone()).expect("Invalid Geyser URL");

    if url.contains(HTTPS_SCHEME) {
        // // Use 👇 to point to the machine's certificate manually if encountering certificate issues during connection
        // let pem = tokio::fs::read("/etc/ssl/cert.pem").await.expect("Issue loading SSL certificate");
        // let certificate = Certificate::from_pem(pem);
        // ClientTlsConfig::new().ca_certificate(cert)

        let certificate = ClientTlsConfig::new();
        endpoint = endpoint.tls_config(certificate).unwrap();
    }
    let channel = endpoint.connect().await.unwrap();

    let mut geyser_client: GeyserClient<InterceptedService<Channel, RequestInterceptor>> =
        GeyserClient::with_interceptor(
            channel,
            RequestInterceptor {
                auth_token: AUTHENTICATION_TOKEN.to_string(),
            },
        );

    let mut stream = geyser_client
        .subscribe(stream::iter([SubscribeRequest {
            accounts: HashMap::from([(
                "feed_account".to_string(),
                SubscribeRequestFilterAccounts {
                    account: vec![PYTH_SOLUSDC_FEED.to_string()],
                    owner: vec![],
                },
            )]),
            slots: HashMap::from([("slots".to_string(), SubscribeRequestFilterSlots {})]),
            transactions: HashMap::from([(
                "feed_transactions".to_string(),
                SubscribeRequestFilterTransactions {
                    vote: None,
                    failed: None,
                    account_include: vec![PYTH_SOLUSDC_FEED.to_string()],
                    account_exclude: vec![],
                },
            )]),
            blocks: HashMap::from([("blocks".to_string(), SubscribeRequestFilterBlocks {})]),
        }]))
        .await
        .unwrap()
        .into_inner();

    while let Some(received) = stream.next().await {
        let received = received.unwrap();
        if let Some(update) = received.update_oneof {
            match update {
                UpdateOneof::Transaction(update) => {
                    info!(
                        "New tx: {:?} (slot: {})",
                        bs58::encode(update.transaction.unwrap().signature).into_string(),
                        update.slot
                    );
                }
                UpdateOneof::Account(update) => {
                    let account_info = update.account.unwrap();
                    let price_account =
                        pyth_sdk_solana::state::load_price_account(&account_info.data).unwrap();
                    let price_feed =
                        price_account.to_price_feed(&Pubkey::new(&account_info.pubkey));

                    let sol_price = price_feed.get_price_unchecked().price as f32
                        * (10f32.powi(price_feed.get_price_unchecked().expo)) as f32; // 🤌
                    warn!(
                        "SOL/USDC feed updated to {:?} in slot: {:?}",
                        sol_price, update.slot
                    );
                }
                UpdateOneof::Block(update) => {
                    debug!("Block update: {} (slot {})", update.blockhash, update.slot);
                }
                UpdateOneof::Slot(update) => {
                    let status = match update.status {
                        0 => "PROCESSED",
                        1 => "CONFIRMED",
                        2 => "FINALIZED",
                        _ => panic!("huh"),
                    };
                    debug!("Slot {} update: status {}", update.slot, status);
                }
            }
        }
    }
}
