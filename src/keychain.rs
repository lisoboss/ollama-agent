//! Keychain support for macOS
//! 
//! This module handles storing and retrieving API keys from the macOS Keychain,
//! with support for different remote URLs.
//! It is only compiled when the "keychain" feature is enabled.

use anyhow::Result;

#[cfg(feature = "keychain")]
use security_framework::passwords::{
    set_generic_password, delete_generic_password, get_generic_password
};
#[cfg(feature = "keychain")]
use std::collections::hash_map::DefaultHasher;
#[cfg(feature = "keychain")]
use std::hash::{Hash, Hasher};

// Constants for keychain item identification
#[cfg(feature = "keychain")]
const SERVICE_NAME: &str = "ollama-agent";

/// Checks if the keychain feature is enabled
pub fn is_keychain_enabled() -> bool {
    cfg!(feature = "keychain")
}

/// Helper function to create an account name based on the remote URL
#[cfg(feature = "keychain")]
fn create_account_name(remote_url: &str) -> String {
    // Remove protocol and trailing slashes for cleaner account names
    let clean_url = remote_url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/');
    
    // For very long URLs, hash them to avoid keychain limits
    if clean_url.len() > 50 {
        let mut hasher = DefaultHasher::new();
        clean_url.hash(&mut hasher);
        format!("api-key-{}", hasher.finish())
    } else {
        format!("api-key-{}", clean_url)
    }
}

/// Saves an API key to the macOS Keychain
#[cfg(feature = "keychain")]
pub fn save_api_key(api_key: &str, remote_url: &str) -> Result<()> {
    use log::{debug, info};
    debug!("Attempting to save API key for {} to macOS Keychain", remote_url);
    
    // Check if the API key is empty
    if api_key.is_empty() {
        return Err(anyhow::anyhow!("Cannot save empty API key to keychain"));
    }
    
    let account_name = create_account_name(remote_url);
    
    // First try to delete any existing password
    let _ = delete_generic_password(SERVICE_NAME, &account_name);
    
    // Save the new password
    set_generic_password(
        SERVICE_NAME,
        &account_name,
        api_key.as_bytes(),
    )
    .map_err(|e| anyhow::anyhow!("Failed to save API key to macOS Keychain: {}", e))?;
    
    info!("API key for {} successfully saved to macOS Keychain", remote_url);
    Ok(())
}

/// Retrieves the API key from the macOS Keychain
#[cfg(feature = "keychain")]
pub fn get_api_key(remote_url: &str) -> Result<String> {
    use log::debug;
    debug!("Attempting to read API key for {} from macOS Keychain", remote_url);
    
    let account_name = create_account_name(remote_url);
    
    let password = get_generic_password(
        SERVICE_NAME,
        &account_name,
    )
    .map_err(|e| anyhow::anyhow!("Failed to retrieve API key from macOS Keychain for {}: {}", remote_url, e))?;
    
    // Convert password bytes to string
    let api_key = String::from_utf8(password.to_vec())
        .map_err(|e| anyhow::anyhow!("API key in keychain is not valid UTF-8: {}", e))?;
    
    debug!("API key for {} retrieved from macOS Keychain (length: {})", remote_url, api_key.len());
    Ok(api_key)
}

/// Removes the API key from the macOS Keychain
#[cfg(feature = "keychain")]
pub fn delete_api_key(remote_url: &str) -> Result<()> {
    use log::{debug, info};
    debug!("Attempting to delete API key for {} from macOS Keychain", remote_url);
    
    let account_name = create_account_name(remote_url);
    
    delete_generic_password(SERVICE_NAME, &account_name)
        .map_err(|e| anyhow::anyhow!("Failed to delete API key from macOS Keychain for {}: {}", remote_url, e))?;
    
    info!("API key for {} successfully deleted from macOS Keychain", remote_url);
    Ok(())
}

/// Lists all Ollama API keys stored in the keychain
/// 
/// Since security-framework doesn't provide a direct way to list all items,
/// we'll use a simpler approach by pre-populating a list of known URLs.
/// Users will see these URLs in the list after they've used them at least once.
#[cfg(feature = "keychain")]
pub fn list_saved_urls() -> Result<Vec<String>> {
    use log::{debug, info};
    use std::collections::HashSet;
    
    debug!("Attempting to list all saved API keys from macOS Keychain");

    // Commonly used URLs to check
    let urls_to_check = vec![
        "api.ollama.ai",
        "localhost:11434",
        "127.0.0.1:11434",
    ];
    
    // Use the predefined list of URLs to check
    let urls_to_check = urls_to_check;
    
    // Check each URL to see if we have an API key saved for it
    let mut found_urls = HashSet::new();
    for url in urls_to_check {
        let account_name = create_account_name(url);
        
        // Try to find a password for this account
        if get_generic_password(SERVICE_NAME, &account_name).is_ok() {
            found_urls.insert(url.to_string());
        }
    }
    
    info!("Found {} saved API keys in macOS Keychain", found_urls.len());
    Ok(found_urls.into_iter().collect())
}

/// Dummy implementations for when the keychain feature is disabled
#[cfg(not(feature = "keychain"))]
pub fn save_api_key(_api_key: &str, _remote_url: &str) -> Result<()> {
    Err(anyhow::anyhow!("Keychain support is not enabled. Compile with '--features keychain' to enable this functionality."))
}

#[cfg(not(feature = "keychain"))]
pub fn get_api_key(_remote_url: &str) -> Result<String> {
    Err(anyhow::anyhow!("Keychain support is not enabled. Compile with '--features keychain' to enable this functionality."))
}

#[cfg(not(feature = "keychain"))]
pub fn delete_api_key(_remote_url: &str) -> Result<()> {
    Err(anyhow::anyhow!("Keychain support is not enabled. Compile with '--features keychain' to enable this functionality."))
}

#[cfg(not(feature = "keychain"))]
pub fn list_saved_urls() -> Result<Vec<String>> {
    Err(anyhow::anyhow!("Keychain support is not enabled. Compile with '--features keychain' to enable this functionality."))
}