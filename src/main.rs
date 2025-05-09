use anyhow::{Context, Result};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::client::HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, HeaderMap, Request, Response, Server, StatusCode};
use hyper_tls::HttpsConnector;
use log::{debug, error, info, warn};

mod keychain;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Local address to bind to
    #[arg(short, long, default_value = "127.0.0.1:11434")]
    local_addr: String,

    /// Remote Ollama API URL
    #[arg(short, long, default_value = "https://api.ollama.ai")]
    remote_url: String,

    /// API key for authentication
    #[arg(short, long, env("OLLAMA_API_KEY"))]
    api_key: Option<String>,

    /// Save API key to macOS Keychain for the specified remote URL (requires keychain feature)
    #[arg(long)]
    save_key: bool,

    /// Use API key from macOS Keychain for the specified remote URL if not provided (requires keychain feature)
    #[arg(long, default_value = "true")]
    use_keychain: bool,

    /// Delete saved API key from macOS Keychain for the specified remote URL (requires keychain feature)
    #[arg(long)]
    delete_key: bool,

    /// List all remote URLs with saved API keys in macOS Keychain (requires keychain feature)
    #[arg(long)]
    list_keys: bool,
}

type HttpClient = Client<HttpsConnector<HttpConnector>>;

struct AppState {
    client: HttpClient,
    args: Args,
}

// Helper function to check if a request might be streaming
fn is_streaming_request(uri: &hyper::Uri) -> bool {
    let path = uri.path();
    // Check for Ollama streaming endpoints
    path.contains("/api/chat") || path.contains("/api/generate")
}

async fn proxy_handler(
    req: Request<Body>,
    state: Arc<AppState>,
) -> Result<Response<Body>, hyper::Error> {
    let args = &state.args;
    let client = &state.client;

    // Get the path and query from the request
    let uri = req.uri();
    let path_and_query = uri.path_and_query().map(|x| x.as_str()).unwrap_or("/");

    // Check if this is a streaming request
    let is_stream = is_streaming_request(uri);

    // Construct the remote URL
    let remote_url = format!("{}{}", args.remote_url, path_and_query);

    // Create a new request with the same method, headers, and body
    let (parts, body) = req.into_parts();
    let method_clone = parts.method.clone();
    let uri_clone = parts.uri.clone();
    let mut builder = Request::builder()
        .method(parts.method)
        .uri(remote_url.clone());

    // Add all the original headers
    let mut headers = HeaderMap::new();
    for (name, value) in parts.headers {
        if let Some(name) = name {
            // Skip host header as it will be set by the client
            if name != hyper::header::HOST {
                headers.insert(name, value);
            }
        }
    }

    // Add the API key header for authentication if provided
    if let Some(api_key) = &args.api_key {
        match format!("Bearer {}", api_key).parse() {
            Ok(auth_value) => {
                headers.insert("Authorization", auth_value);
            }
            Err(e) => {
                error!("Failed to create Authorization header: {}", e);
                // Continue without the header, the remote server will reject the request
            }
        }
    }

    *builder.headers_mut().unwrap() = headers;

    // Log the outgoing request (excluding sensitive headers)
    info!(
        "Proxying request: {} {} -> {} {}",
        method_clone,
        uri_clone,
        remote_url,
        if is_stream { "[STREAMING]" } else { "" }
    );

    // Build and send the request to the remote server
    let remote_req = match builder.body(body) {
        Ok(req) => req,
        Err(err) => {
            error!("Failed to build remote request: {}", err);
            let mut response = Response::new(Body::from("Internal Server Error"));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(response);
        }
    };

    // Send the request to the remote server with a timeout
    match tokio::time::timeout(
        std::time::Duration::from_secs(300), // Increase timeout for streaming responses
        client.request(remote_req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            let status = resp.status();
            let content_type = resp
                .headers()
                .get(hyper::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            info!(
                "Received response: {} {} (Content-Type: {})",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown"),
                content_type
            );

            // Debug log for streaming responses
            if content_type.contains("stream") || content_type.contains("event-stream") {
                info!("Detected streaming response, preserving chunked encoding");
            }

            Ok(resp)
        }
        Ok(Err(err)) => {
            // Return a 502 Bad Gateway error if the proxy request fails
            error!("Proxy request failed: {}", err);
            let mut response = Response::new(Body::from("Bad Gateway"));
            *response.status_mut() = StatusCode::BAD_GATEWAY;
            Ok(response)
        }
        Err(_) => {
            // Return a 504 Gateway Timeout error if the request times out
            error!("Proxy request timed out");
            let mut response = Response::new(Body::from("Gateway Timeout"));
            *response.status_mut() = StatusCode::GATEWAY_TIMEOUT;
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Parse command-line arguments
    let mut args = Args::parse();

    // Check if keychain feature is enabled
    if (args.save_key || args.delete_key || args.use_keychain) && !keychain::is_keychain_enabled() {
        warn!("macOS Keychain operations requested but keychain feature is not enabled");
        warn!("Compile with '--features keychain' to enable keychain integration");

        // If save-key was requested but not available, warn the user their key won't be saved
        if args.save_key && args.api_key.is_some() {
            warn!("API key will NOT be saved to keychain due to missing feature");
        }

        // If use-keychain was requested but not available, warn the user
        if args.use_keychain && args.api_key.is_none() {
            warn!("Unable to retrieve API key from keychain due to missing feature");
        }
    }

    // Handle keychain operations
    if keychain::is_keychain_enabled() {
        // List saved keys if requested
        if args.list_keys {
            match keychain::list_saved_urls() {
                Ok(urls) => {
                    if urls.is_empty() {
                        info!("No saved API keys found in macOS Keychain");
                    } else {
                        info!("Saved API keys found for the following remote URLs:");
                        for (i, url) in urls.iter().enumerate() {
                            println!("  {}. {}", i + 1, url);
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    error!("Failed to list saved API keys: {}", e);
                    return Err(e);
                }
            }
        }

        // Delete key if requested
        if args.delete_key {
            match keychain::delete_api_key(&args.remote_url) {
                Ok(_) => {
                    info!(
                        "✅ API key successfully deleted from macOS Keychain for {}",
                        args.remote_url
                    );
                    if args.api_key.is_none() {
                        // Exit if we're only deleting the key
                        return Ok(());
                    }
                }
                Err(e) => warn!(
                    "❌ Failed to delete API key from keychain for {}: {}",
                    args.remote_url, e
                ),
            }
        }

        // Save key if provided and save requested
        if let Some(key) = &args.api_key {
            if args.save_key {
                match keychain::save_api_key(key, &args.remote_url) {
                    Ok(_) => info!(
                        "✅ API key successfully saved to macOS Keychain for {}",
                        args.remote_url
                    ),
                    Err(e) => warn!(
                        "❌ Failed to save API key to keychain for {}: {}",
                        args.remote_url, e
                    ),
                }
            }
        }

        // Try to get key from keychain if not provided but use_keychain is true
        if args.api_key.is_none() && args.use_keychain {
            match keychain::get_api_key(&args.remote_url) {
                Ok(key) => {
                    info!(
                        "Using API key from macOS Keychain for {} (length: {})",
                        args.remote_url,
                        key.len()
                    );
                    args.api_key = Some(key);
                }
                Err(e) => {
                    if args.use_keychain {
                        debug!(
                            "Could not retrieve API key from keychain for {}: {}",
                            args.remote_url, e
                        );
                    }
                }
            }
        }
    }

    info!("Starting Ollama proxy server...");
    info!("Local address: {}", args.local_addr);
    info!("Remote URL: {}", args.remote_url);
    info!(
        "API key authentication: {}",
        if let Some(key) = &args.api_key {
            format!("enabled (length: {})", key.len())
        } else {
            "disabled".to_string()
        }
    );
    if keychain::is_keychain_enabled() {
        info!("macOS Keychain support: enabled (per remote URL)");
    } else {
        info!("macOS Keychain support: disabled");
    }

    // Validate remote URL format
    if !args.remote_url.starts_with("http://") && !args.remote_url.starts_with("https://") {
        anyhow::bail!("Remote URL must start with http:// or https://");
    }

    // Create HTTPS client with timeouts suitable for streaming
    let https = HttpsConnector::new();
    let client = Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(300))
        .pool_max_idle_per_host(32) // Increase connection pool size
        .http2_only(false) // Support both HTTP/1.1 and HTTP/2
        .http2_initial_stream_window_size(1024 * 1024) // 1MB
        .http2_initial_connection_window_size(1024 * 1024) // 1MB
        .build::<_, Body>(https);

    // Create shared state
    let state = Arc::new(AppState {
        client,
        args: args.clone(),
    });

    // Bind to the local address
    let addr: SocketAddr = args
        .local_addr
        .parse()
        .context("Failed to parse local address")?;

    // Create the service
    let make_service = make_service_fn(move |_| {
        let state = state.clone();
        async move { Ok::<_, hyper::Error>(service_fn(move |req| proxy_handler(req, state.clone()))) }
    });

    // Start the server
    let server = Server::bind(&addr).serve(make_service);
    info!("Ollama proxy server listening on http://{}", addr);

    // Set up graceful shutdown signal handler
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    // Handle Ctrl+C to gracefully shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Received shutdown signal, gracefully shutting down...");
        let _ = tx.send(());
    };

    // Run the server with graceful shutdown
    tokio::select! {
        result = server => {
            result.context("Server error")?;
        },
        _ = shutdown_signal => {
            info!("Server shutting down...");
        },
        _ = rx => {
            info!("Shutdown signal received, server shutting down...");
        }
    }

    info!("Server shutdown complete");
    Ok(())
}
