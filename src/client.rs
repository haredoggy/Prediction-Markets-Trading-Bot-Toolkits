use alloy::signers::Signer;
use alloy::signers::local::LocalSigner;
use anyhow::Result;
use polymarket_client_sdk::{
    POLYGON,
    auth::state::Authenticated,
    clob::{Client, Config, types::SignatureType},
    types::Address,
};
use std::str::FromStr;

/// Creates an authenticated ClobClient with API key credentials.
///
/// This function:
/// 1. Creates an unauthenticated client
/// 2. Attempts to create an API key, falling back to deriving if creation fails
/// 3. Returns an authenticated client with the credentials
pub async fn create_clob_client() -> Result<Client<Authenticated>> {
    let chain_id = POLYGON; // 137
    let host =
        std::env::var("CLOB_HTTP_URL").expect("CLOB_HTTP_URL environment variable must be set");
    let private_key =
        std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable must be set");
    let proxy_wallet =
        std::env::var("PROXY_WALLET").expect("PROXY_WALLET environment variable must be set");

    // Create signer from private key
    let wallet = LocalSigner::from_str(&private_key)?.with_chain_id(Some(chain_id));

    // Parse proxy wallet address
    let funder = Address::from_str(&proxy_wallet)?;

    // Create unauthenticated client
    let unauthenticated_client = Client::new(&host, Config::default())?;

    // Try to create API key, fall back to derive if it fails
    // This matches the TypeScript behavior: try create_api_key first, then derive_api_key
    let creds = match unauthenticated_client.create_api_key(&wallet, None).await {
        Ok(creds) => {
            println!("API Key created: {:?}", creds);
            creds
        }
        Err(_) => {
            // If create fails (e.g., key already exists), try to derive
            let creds = unauthenticated_client.derive_api_key(&wallet, None).await?;
            println!("API Key derived: {:?}", creds);
            creds
        }
    };

    // Create authenticated client with credentials, signature type, and funder
    let authenticated_client = Client::new(&host, Config::default())?
        .authentication_builder(&wallet)
        .credentials(creds)
        .signature_type(SignatureType::Proxy)
        .funder(funder)
        .authenticate()
        .await?;

    println!("{:?}", authenticated_client);
    Ok(authenticated_client)
}
