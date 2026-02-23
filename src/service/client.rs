use alloy::signers::Signer;
use alloy::signers::local::LocalSigner;
use anyhow::Result;
use polymarket_client_sdk::{
    POLYGON,
    auth::{Credentials, Normal, state::Authenticated},
    clob::{Client, Config, types::SignatureType},
    types::Address,
};
use std::str::FromStr;

/// Creates an authenticated CLOB client with API key credentials.
///
/// This function performs the following steps:
/// 1. Creates an unauthenticated client
/// 2. Attempts to create an API key, falling back to deriving if creation fails
/// 3. Returns an authenticated client with the credentials
///
/// # Arguments
///
/// * `api_base_url` - The API base URL for the client
/// * `private_key` - The private key string for wallet authentication
/// * `funder_address` - The proxy wallet address (funder) for the account
///
/// # Returns
///
/// A tuple containing the authenticated client and credentials
///
/// # Errors
///
/// Returns an error if:
/// - Private key parsing fails
/// - Funder address parsing fails
/// - Client creation fails
/// - API key creation/derivation fails
/// - Authentication fails
pub async fn create_authenticated_clob_client(
    api_base_url: String,
    private_key: String,
    funder_address: String,
) -> Result<Client<Authenticated<Normal>>> {
    const CHAIN_ID: u64 = POLYGON;

    // Create signer from private key
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(CHAIN_ID));

    // Parse proxy wallet address
    let funder_address_parsed = Address::from_str(&funder_address)?;

    // Create unauthenticated client
    let unauthenticated_client = Client::new(&api_base_url, Config::default())?;

    // Try to create API key, fall back to derive if it fails
    // This matches the TypeScript behavior: try create_api_key first, then derive_api_key
    let credentials = match unauthenticated_client.create_api_key(&signer, None).await {
        Ok(creds) => {
            println!("API Key created successfully");
            creds
        }
        Err(_) => {
            // If create fails (e.g., key already exists), try to derive
            let creds = unauthenticated_client.derive_api_key(&signer, None).await?;
            println!("API Key derived successfully");
            creds
        }
    };

    // Create authenticated client with credentials, signature type, and funder
    let authenticated_client = Client::new(&api_base_url, Config::default())?
        .authentication_builder(&signer)
        .credentials(credentials)
        .signature_type(SignatureType::Proxy)
        .funder(funder_address_parsed)
        .authenticate()
        .await?;
    // let authenticated_client = Client::new(&host, Config::default())?
    //     .authentication_builder(&signer)
    //     .credentials(credentials)
    //     .signature_type(SignatureType::GnosisSafe) // Funder auto-derived via CREATE2
    //     .authenticate()
    //     .await?;s

    Ok(authenticated_client)
}
