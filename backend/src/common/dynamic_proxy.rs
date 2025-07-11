use anyhow::{Context, Result};
use rand::seq::IndexedRandom;
use reqwest::{Client, Error as ReqwestError, Proxy};
use std::time::Duration;

// List of User-Agent strings to be chosen randomly.
const USER_AGENT: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.1 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.1 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36 Vivaldi/7.5.3735.44",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.4 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_7_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.4 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (Linux; Android 16; LM-Q720) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.7204.46 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 16; SM-A102U) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.7204.46 Mobile Safari/537.36",
];

/// Returns a randomly selected User-Agent string from the `USER_AGENTS` list.
pub fn get_random_user_agent() -> &'static str {
    // Fallback to the first agent if choose fails (should not happen with a non-empty list).
    USER_AGENT
        .choose(&mut rand::rng())
        .unwrap_or(&USER_AGENT[0])
}

/// Configuration for using proxies.
/// Currently, this is designed for future use as load_dynamic_proxy returns an empty list.
pub struct ProxyConfig {
    pub proxies: Vec<String>,
}

impl ProxyConfig {
    /// Creates a new `ProxyConfig`.
    pub fn new(proxies: Vec<String>) -> Self {
        Self { proxies }
    }

    /// Returns a randomly selected proxy string from the list.
    /// Returns `None` if the proxy list is empty.
    pub fn get_random_proxy_str(&self) -> Option<&String> {
        if self.proxies.is_empty() {
            None
        } else {
            self.proxies.choose(&mut rand::rng())
        }
    }

    /// Attempts to create a `reqwest::Proxy` object from a randomly selected proxy string.
    /// Returns `None` if no proxy string is available.
    /// Returns `Some(Result<Proxy, ReqwestError>)` to indicate success or failure in creating the Proxy object.
    pub fn get_proxy_object(&self) -> Option<Result<Proxy, ReqwestError>> {
        self.get_random_proxy_str().map(|proxy_str| {
            println!("[PROXY] Trying to use proxy from the list"); // Log which proxy is being attempted
            Proxy::all(proxy_str) // This can return an error if the proxy string format is invalid
        })
    }
}

/// Loads a list of proxy strings.
/// Currently, returns an empty vector indicating that dynamic proxies are not yet implemented.
fn load_dynamic_proxy() -> Vec<String> {
    let proxies = vec![]; // Initialize as empty

    // [INFO] Placeholder for future implementation of dynamic proxy loading.
    // Example:
    // let proxies = vec![
    //     "http://user1:pass1@proxy.example.com:8080".to_string(),
    //     "socks5://user2:pass2@anotherproxy.example.com:1080".to_string(),
    // ];

    if !proxies.is_empty() {
        println!(
            "[PROXY] {} Dynamic proxy loaded successfully",
            proxies.len()
        );
    } else {
        println!(
            "[PROXY] No dynamic proxy configuration found. Using system proxy setting or direct connection."
        );
    }
    proxies
}

/// Builds a reqwest::Client with specified configurations, including a random User-Agent
/// and optional proxy settings.
fn build_configured_http_client(
    proxy_config: Option<&ProxyConfig>,
) -> Result<Client, ReqwestError> {
    let user_agent = get_random_user_agent();
    let mut client_builder = Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(30)) // General request timeout
        .connect_timeout(Duration::from_secs(20)); // Connection establishment timeout

    // [INFO] This is used for future dynamic proxy configuration.
    let mut using_proxy = false;
    if let Some(config) = proxy_config {
        if let Some(proxy_result) = config.get_proxy_object() {
            match proxy_result {
                Ok(proxy) => {
                    client_builder = client_builder.proxy(proxy);
                    using_proxy = true;
                }
                Err(e) => {
                    // Log the error but continue without this specific proxy.
                    // The client will be built without a proxy or might use system proxy.
                    eprintln!(
                        "[HTTP Client Internal] Gagal mengkonfigurasi proxy: {}. Melanjutkan tanpa proxy.",
                        e
                    );
                }
            }
        } else {
            // No proxy string was available from ProxyConfig.
            println!("[HTTP CLIENT] No proxy string provided by ProxyConfig.")
        }
    }

    if using_proxy {
        println!(
            "[HTTP Client] Configured with dynamic proxy. User-Agent: {}",
            user_agent
        )
    } else {
        println!(
            "[HTTP Client] Configured without dynamic proxy. User-Agent: {}",
            user_agent
        )
    }
    client_builder.build() // This can also return ReqwestError
}

/// Initializes and returns a reqwest::Client.
/// This is the main public function for obtaining a configured HTTP client
pub fn init_client() -> Result<Client> {
    println!("[HTTP Client] Initializing HTTP client...");
    let proxy_list = load_dynamic_proxy(); // Currently returns empty

    // Only create ProxyConfig if proxy_list is not empty.
    let proxy_config = if !proxy_list.is_empty() {
        Some(ProxyConfig::new(proxy_list))
    } else {
        None
    };

    build_configured_http_client(proxy_config.as_ref()) // Pass Option<&ProxyConfig>
        .context("Failed to initialize HTTP client") // Context from anyhow for better error reporting
}
