use alloy::signers::local::LocalSigner;
use alloy::signers::{Signer, k256::ecdsa::SigningKey};
use anyhow::{Result, anyhow};
use polymarket_client_sdk::{
    POLYGON,
    auth::{Normal, state::Authenticated},
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
/// - Authentication fails
pub async fn create_authenticated_clob_client(
    api_base_url: String,
    private_key: String,
    funder_address: String,
) -> Result<(Client<Authenticated<Normal>>, LocalSigner<SigningKey>)> {
    const CHAIN_ID: u64 = POLYGON;

    // Create signer from private key
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(CHAIN_ID));

    // Parse funder address
    let funder_addr = Address::from_str(funder_address.trim_start_matches("0x"))
        .map_err(|e| anyhow!("Invalid funder_address format: {}", e))?;

    // Get signer address to compare with funder
    let signer_addr = signer.address();

    // Create unauthenticated client
    let unauthenticated_client = Client::new(&api_base_url, Config::default())?;

    // We need to authenticate temporarily to build the order, but we'll post without API keys
    // Funder is a Gnosis Safe (proxy) address, signer is private key that can sign for the Safe
    let authenticated_client = if funder_addr == signer_addr {
        // Funder and signer are the same - SDK will use Eoa automatically
        unauthenticated_client
            .authentication_builder(&signer)
            .authenticate()
            .await?
    } else {
        // Funder is a Gnosis Safe address (proxy wallet), signer is private key
        // Use GnosisSafe signature type for Gnosis Safe wallets
        unauthenticated_client
            .authentication_builder(&signer)
            .funder(funder_addr)
            .signature_type(SignatureType::GnosisSafe)
            .authenticate()
            .await?
    };

    Ok((authenticated_client, signer))
}
