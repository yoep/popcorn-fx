use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::net::TcpListener;
use std::result;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

use async_trait::async_trait;
use log::{debug, error, trace, warn};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, TokenUrl};
use oauth2::basic::BasicClient;
use oauth2::http::HeaderMap;
use oauth2::reqwest::async_http_client;
use reqwest::Client;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex, oneshot};
use warp::Filter;
use warp::http::Response;

use popcorn_fx_core::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks};
use popcorn_fx_core::core::config::{ApplicationConfig, TrackingClientProperties, TrackingProperties};
use popcorn_fx_core::core::tracking::{AuthorizationError, OpenAuthorization, TrackingEvent, TrackingProvider};

const AUTHORIZED_PORTS: [u16; 3] = [
    30200u16,
    30201u16,
    30202u16,
];

pub type Result<T> = result::Result<T, TraktError>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum TraktError {
    #[error("failed to create new instance, {0}")]
    Creation(String),
    #[error("none of the authorized ports are available")]
    NoAvailablePorts,
}

pub struct TraktProvider {
    oauth_client: BasicClient,
    client: Client,
    port: u16,
    open_authorization_callback: Mutex<OpenAuthorization>,
    runtime: Runtime,
    callbacks: CoreCallbacks<TrackingEvent>,
}

impl TraktProvider {
    pub fn new(application_config: Arc<ApplicationConfig>) -> Result<Self> {
        let tracking: TrackingProperties;
        let client: &TrackingClientProperties;
        {
            let properties = application_config.properties_ref();
            tracking = properties.tracker("trakt")
                .cloned()
                .map_err(|e| {
                    TraktError::Creation(e.to_string())
                })?;
            client = tracking.client();
        }

        let port = Self::available_port()?;
        let oauth_client = BasicClient::new(
            ClientId::new(client.client_id.clone()),
            Some(ClientSecret::new(client.client_secret.clone())),
            AuthUrl::new(client.user_authorization_uri.clone()).map_err(|e| {
                TraktError::Creation(e.to_string())
            })?,
            Some(TokenUrl::new(client.access_token_uri.clone()).map_err(|e| {
                TraktError::Creation(e.to_string())
            })?),
        )
            .set_redirect_uri(RedirectUrl::new(format!("http://localhost:{}/callback", port)).expect("expected a valid redirect url"));

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(1)
            .thread_name("trakt-auth-server")
            .build()
            .expect("expected a new runtime");

        Ok(Self {
            oauth_client,
            client: Self::create_new_client(client),
            port,
            open_authorization_callback: Mutex::new(Box::new(|uri: String| match open::that(uri.as_str()) {
                Ok(_) => true,
                Err(e) => {
                    error!("Failed to open authorization uri, {}", e);
                    false
                }
            })),
            runtime,
            callbacks: Default::default(),
        })
    }

    fn start_auth_server(&self, sender: Sender<AuthCallbackResult>, shutdown_signal: oneshot::Receiver<()>) {
        trace!("Starting new Trakt auth server on port {}", self.port);
        let routes = warp::get()
            .and(warp::path!("callback"))
            .and(warp::query::<HashMap<String, String>>())
            .map(move |p: HashMap<String, String>| {
                if let Some(auth_code) = p.get("code") {
                    if let Some(state) = p.get("state") {
                        sender.send(AuthCallbackResult {
                            authorization_code: auth_code.to_string(),
                            state: state.to_string(),
                        }).unwrap();
                    }
                }

                Response::builder()
                    .body("You can close this window now")
                    .unwrap()
            })
            .with(warp::cors().allow_any_origin());

        let server = warp::serve(routes);

        debug!("Starting auth server on port {}", self.port);
        let (_, server) = server.bind_with_graceful_shutdown(([127, 0, 0, 1], self.port), async {
            shutdown_signal.await.ok();
            debug!("Shutting down Trakt auth server");
        });
        
        self.runtime.spawn(server);
    }

    fn available_port() -> Result<u16> {
        for port in AUTHORIZED_PORTS.iter() {
            trace!("Checking port availability of {}", port);
            if TcpListener::bind(("localhost", port.clone())).is_ok() {
                return Ok(port.clone());
            }
        }

        Err(TraktError::NoAvailablePorts)
    }
    
    fn create_new_client(properties: &TrackingClientProperties) -> Client {
        let mut headers = HeaderMap::new();
        
        headers.insert("trakt-api-version", "2".parse().unwrap());
        headers.insert("trakt-api-key", properties.client_id.parse().unwrap());
        
        Client::builder()
            .default_headers(headers)
            .build().unwrap()
    }
}

impl Callbacks<TrackingEvent> for TraktProvider {
    fn add(&self, callback: CoreCallback<TrackingEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
    }
}

#[async_trait]
impl TrackingProvider for TraktProvider {
    fn register_open_authorization(&self, open_callback: OpenAuthorization) {
        trace!("Updating authorization open callback");
        let mut mutex = block_in_place(self.open_authorization_callback.lock());
        *mutex = open_callback;
        debug!("Callback for opening authorization uri's has been updated");
    }

    fn is_authorized(&self) -> bool {
        false
    }

    async fn authorize(&self) -> result::Result<(), AuthorizationError> {
        trace!("Starting authorization flow for TraktTV");
        let open_callback = self.open_authorization_callback.lock().await;
        let (tx_shutdown, rx_shutdown) = oneshot::channel();
        let (tx, rx) = channel();

        let (auth_url, csrf_token) = self.oauth_client
            .authorize_url(CsrfToken::new_random)
            .url();
        self.start_auth_server(tx, rx_shutdown);

        return if open_callback(auth_url.to_string()) {
            return match rx.recv_timeout(Duration::from_secs(60 * 5)) {
                Ok(callback) => {
                    trace!("Received callback result {:?}", callback);
                    tx_shutdown.send(()).unwrap();

                    // verify csrf token
                    if csrf_token.secret() != &callback.state {
                        warn!("Authorization CSRF token mismatch, Trakt won't be authorized");
                        return Err(AuthorizationError::CsrfFailure);
                    }

                    return match self.oauth_client
                        .exchange_code(AuthorizationCode::new(callback.authorization_code))
                        .request_async(async_http_client)
                        .await {
                        Ok(e) => {
                            trace!("Received token response {:?}", e);
                            self.callbacks.invoke(TrackingEvent::AuthorizationStateChanged(true));
                            Ok(())
                        }
                        Err(e) => {
                            error!("Token exchange failed, {}", e);
                            Err(AuthorizationError::Token)
                        }
                    };
                }
                Err(e) => {
                    error!("Failed to retrieve authorization code, {}", e);
                    tx_shutdown.send(()).unwrap();
                    Err(AuthorizationError::AuthorizationCode)
                }
            };
        } else {
            Err(AuthorizationError::AuthorizationUriOpen)
        };
    }
}

impl Debug for TraktProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraktProvider")
            .field("client", &self.oauth_client)
            .field("port", &self.port)
            .field("runtime", &self.runtime)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AuthCallbackResult {
    pub authorization_code: String,
    pub state: String,
}

#[cfg(test)]
mod tests {
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use reqwest::Client;
    use tempfile::tempdir;
    use url::Url;

    use popcorn_fx_core::core::config::PopcornProperties;
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_new() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build());

        let result = TraktProvider::new(settings);

        if let Err(e) = result {
            assert!(false, "failed to create new Trakt instance, {}", e)
        }
    }

    #[test]
    fn test_authorize() {
        init_logger();
        let expected_code = "MyAuthCodeResult";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/oauth/token")
                .header_exists("authorization");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(r#"
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "tGzv3JOkF0XG5Qx2TlKWIA",
  "scope": "openid"
}
                "#);
        });
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties {
                loggers: Default::default(),
                update_channel: "".to_string(),
                providers: Default::default(),
                enhancers: Default::default(),
                subtitle: Default::default(),
                tracking: vec![
                    ("trakt".to_string(), TrackingProperties {
                        uri: server.base_url(),
                        client: TrackingClientProperties {
                            client_id: "SomeClientId".to_string(),
                            client_secret: "SomeClientSecret".to_string(),
                            user_authorization_uri: server.url("/oauth/authorize"),
                            access_token_uri: server.url("/oauth/token"),
                        },
                    })
                ].into_iter().collect(),
            })
            .build());
        let (tx, rx) = channel();
        let trakt = TraktProvider::new(settings).unwrap();

        trakt.register_open_authorization(Box::new(move |uri| {
            tx.send(uri).unwrap();
            true
        }));

        let callback_uri = trakt.oauth_client.redirect_url().cloned().unwrap();
        trakt.runtime.spawn(async move {
            let client = Client::new();
            let auth_uri = Url::parse(rx.recv_timeout(Duration::from_secs(1)).unwrap().as_str()).expect("expected the authorization open to have been invoked");

            let (_, state) = auth_uri.query_pairs().find(|(key, _)| key == "state").expect("expected the state key to have been found");

            let uri = callback_uri.url().clone()
                .query_pairs_mut()
                .append_pair("code", expected_code)
                .append_pair("state", state.as_ref())
                .finish()
                .to_string();

            if let Err(e) = client.get(uri)
                .send()
                .await {
                assert!(false, "expected the callback to have succeeded, {}", e)
            }
        });

        let result = block_in_place(trakt.authorize());
        
        if let Err(e) = result {
            assert!(false, "expected the authorization to have succeeded, {}", e);
        }
        
        mock.assert_hits(1);
    }
}

