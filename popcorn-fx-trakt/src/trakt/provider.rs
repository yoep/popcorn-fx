use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::net::{SocketAddr, TcpListener};
use std::result;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{Local, Utc};
use log::{debug, error, info, trace, warn};
use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, TokenResponse,
    TokenUrl,
};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use reqwest::Client;
use reqwest::header::HeaderMap;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex, oneshot};
use url::Url;
use warp::Filter;
use warp::http::Response;

use popcorn_fx_core::core::{
    block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks,
};
use popcorn_fx_core::core::config::{
    ApplicationConfig, Tracker, TrackingClientProperties, TrackingProperties,
};
use popcorn_fx_core::core::media::MediaIdentifier;
use popcorn_fx_core::core::media::tracking::{
    AuthorizationError, OpenAuthorization, TrackingError, TrackingEvent, TrackingProvider,
};

use crate::trakt::{AddToWatchList, Movie, MovieId, WatchedMovie};

const TRACKING_NAME: &str = "trakt";
const AUTHORIZED_PORTS: [u16; 5] = [30200u16, 30201u16, 30202u16, 30203u16, 30204u16];

/// Represents the result type used in Trakt operations.
pub type Result<T> = result::Result<T, TraktError>;

/// Represents errors that can occur during Trakt operations.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum TraktError {
    /// Indicates a failure during instance creation.
    #[error("failed to create new instance: {0}")]
    Creation(String),
    /// Indicates that none of the authorized ports are available.
    #[error("none of the authorized ports are available")]
    NoAvailablePorts,
    /// Indicates that the authorization process failed.
    #[error("failed to authorize the user, {0}")]
    AuthorizationError(String),
    /// Indicates that the Trakt provider has not been authorized.
    #[error("Trakt provider has not been authorized")]
    Unauthorized,
    /// Indicates an error during token exchange.
    #[error("failed to exchange token: {0}")]
    TokenError(String),
}

pub struct TraktProvider {
    config: Arc<ApplicationConfig>,
    oauth_client: BasicClient,
    client: Client,
    open_authorization_callback: Mutex<OpenAuthorization>,
    runtime: Arc<Runtime>,
    callbacks: CoreCallbacks<TrackingEvent>,
}

impl TraktProvider {
    pub fn new(config: Arc<ApplicationConfig>, runtime: Arc<Runtime>) -> Result<Self> {
        let tracking: TrackingProperties;
        let client: &TrackingClientProperties;
        {
            let properties = config.properties_ref();
            tracking = properties
                .tracker(TRACKING_NAME)
                .cloned()
                .map_err(|e| TraktError::Creation(e.to_string()))?;
            client = tracking.client();
        }

        let oauth_client = BasicClient::new(
            ClientId::new(client.client_id.clone()),
            Some(ClientSecret::new(client.client_secret.clone())),
            AuthUrl::new(client.user_authorization_uri.clone())
                .map_err(|e| TraktError::Creation(e.to_string()))?,
            Some(
                TokenUrl::new(client.access_token_uri.clone())
                    .map_err(|e| TraktError::Creation(e.to_string()))?,
            ),
        );

        Ok(Self {
            config,
            oauth_client,
            client: Self::create_new_client(client),
            open_authorization_callback: Mutex::new(Box::new(|uri: String| {
                match open::that(uri.as_str()) {
                    Ok(_) => true,
                    Err(e) => {
                        error!("Failed to open authorization uri, {}", e);
                        false
                    }
                }
            })),
            runtime,
            callbacks: Default::default(),
        })
    }

    fn start_auth_server(
        &self,
        sender: Sender<AuthCallbackResult>,
        shutdown_signal: oneshot::Receiver<()>,
    ) -> Result<SocketAddr> {
        trace!("Starting new Trakt authorization callback server");
        let routes = warp::get()
            .and(warp::path!("callback"))
            .and(warp::query::<HashMap<String, String>>())
            .map(move |p: HashMap<String, String>| {
                if let Some(auth_code) = p.get("code") {
                    if let Some(state) = p.get("state") {
                        sender
                            .send(AuthCallbackResult {
                                authorization_code: auth_code.to_string(),
                                state: state.to_string(),
                            })
                            .unwrap();
                    }
                }

                Response::builder()
                    .body("You can close this window now")
                    .unwrap()
            })
            .with(warp::cors().allow_any_origin());

        let server = warp::serve(routes);

        let addr = Self::available_address()?;
        debug!("Starting auth server on {}", addr);
        match server.try_bind_with_graceful_shutdown(addr, async {
            shutdown_signal.await.ok();
            debug!("Shutting down Trakt auth server");
        }) {
            Ok((addr, server)) => {
                self.runtime.spawn(server);
                Ok(addr)
            }
            Err(e) => Err(TraktError::AuthorizationError(e.to_string())),
        }
    }

    async fn bearer_token(&self) -> Result<String> {
        match self
            .config
            .user_settings_ref()
            .tracking()
            .tracker(TRACKING_NAME)
            .clone()
        {
            None => Err(TraktError::Unauthorized),
            Some(settings) => {
                let mut access_token = settings.access_token;

                if let Some(expired_at) = settings.expires_in.filter(|e| {
                    let now = Local::now().with_timezone(&Utc);
                    &now > e
                }) {
                    if let Some(refresh_token) = settings.refresh_token {
                        debug!("Token has expired at {}, refreshing token info", expired_at);
                        let token = self.exchange_refresh_token(refresh_token).await?;
                        access_token = token.access_token().secret().clone();
                        self.update_token_info(token);
                    } else {
                        warn!("Token has expired at {}, unable to refresh token, no refresh token present", expired_at);
                        return Err(TraktError::Unauthorized);
                    }
                }

                Ok(access_token)
            }
        }
    }

    async fn exchange_refresh_token<S: Into<String>>(
        &self,
        refresh_token: S,
    ) -> Result<BasicTokenResponse> {
        let refresh_token = refresh_token.into();
        trace!("Exchanging refresh token {}", refresh_token);
        self.oauth_client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(|e| TraktError::TokenError(e.to_string()))
    }

    fn update_token_info(&self, token: BasicTokenResponse) {
        let tracker = Tracker {
            access_token: token.access_token().secret().clone(),
            expires_in: token.expires_in().map(|e| {
                let now = Local::now().with_timezone(&Utc);
                now + e
            }),
            refresh_token: token.refresh_token().map(|e| e.secret().clone()),
            scopes: token
                .scopes()
                .map(|vec| vec.into_iter().map(|e| e.to_string()).collect()),
        };

        self.config.update_tracker(TRACKING_NAME, tracker);
    }

    fn available_address() -> Result<SocketAddr> {
        for port in AUTHORIZED_PORTS.iter() {
            trace!("Checking port availability of {}", port);
            if let Ok(listener) = TcpListener::bind(("localhost", port.clone())) {
                return Ok(listener.local_addr().unwrap());
            }
        }

        Err(TraktError::NoAvailablePorts)
    }

    fn create_new_client(properties: &TrackingClientProperties) -> Client {
        let mut headers = HeaderMap::new();

        headers.insert("trakt-api-version", "2".parse().unwrap());
        headers.insert("trakt-api-key", properties.client_id.parse().unwrap());

        Client::builder().default_headers(headers).build().unwrap()
    }

    fn properties(&self) -> TrackingProperties {
        let properties = self.config.properties();

        properties
            .tracker(TRACKING_NAME)
            .cloned()
            .expect("expected the tracker properties to have been present")
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
        self.config
            .user_settings_ref()
            .tracking()
            .tracker(TRACKING_NAME)
            .is_some()
    }

    async fn authorize(&self) -> result::Result<(), AuthorizationError> {
        trace!("Starting authorization flow for TraktTV");
        let open_callback = self.open_authorization_callback.lock().await;
        let (tx_shutdown, rx_shutdown) = oneshot::channel();
        let (tx, rx) = channel();

        let addr = self.start_auth_server(tx, rx_shutdown).map_err(|e| {
            error!("Failed to start authorization server, {}", e);
            AuthorizationError::AuthorizationCode
        })?;
        let oauth_client = self.oauth_client.clone().set_redirect_uri(
            RedirectUrl::new(format!("http://localhost:{}/callback", addr.port()))
                .expect("expected a valid redirect url"),
        );
        let (auth_url, csrf_token) = oauth_client.authorize_url(CsrfToken::new_random).url();

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

                    return match self
                        .oauth_client
                        .exchange_code(AuthorizationCode::new(callback.authorization_code))
                        .request_async(async_http_client)
                        .await
                    {
                        Ok(e) => {
                            trace!("Received token response {:?}", e);
                            self.update_token_info(e);
                            self.callbacks
                                .invoke(TrackingEvent::AuthorizationStateChanged(true));
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

    async fn disconnect(&self) {
        trace!("Disconnecting Trakt media tracking");
        self.config.remove_tracker(TRACKING_NAME);
        self.callbacks
            .invoke(TrackingEvent::AuthorizationStateChanged(false));
    }

    async fn add_watched_movies(
        &self,
        movie_ids: Vec<String>,
    ) -> result::Result<(), TrackingError> {
        trace!("Adding {:?} movies to Trakt", movie_ids);
        let properties = self.properties();
        let bearer_token = self.bearer_token().await.map_err(|e| {
            error!("Failed to retrieve Trakt bearer token, {}", e);
            TrackingError::Unauthorized
        })?;
        let mut uri = Url::parse(properties.uri()).unwrap();
        uri.set_path("/sync/watchlist");

        let response = self
            .client
            .post(uri)
            .bearer_auth(bearer_token)
            .json(&AddToWatchList {
                movies: movie_ids
                    .into_iter()
                    .map(|e| Movie {
                        title: "".to_string(),
                        year: None,
                        ids: MovieId {
                            trakt: None,
                            slug: None,
                            imdb: e,
                            tmdb: None,
                        },
                    })
                    .collect(),
                shows: vec![],
            })
            .send()
            .await
            .map_err(|e| {
                error!("Failed to updated watched movies, {}", e);
                TrackingError::Request
            })?;

        if response.status().is_success() {
            info!("Watched movies have been updated with Trakt");
            Ok(())
        } else {
            error!("Received status code {}", response.status());
            Err(TrackingError::Request)
        }
    }

    async fn watched_movies(&self) -> result::Result<Vec<Box<dyn MediaIdentifier>>, TrackingError> {
        trace!("Retrieving Trakt watched movies");
        let properties = self.properties();
        let bearer_token = self.bearer_token().await.map_err(|e| {
            error!("Failed to retrieve Trakt bearer token, {}", e);
            TrackingError::Unauthorized
        })?;
        let mut uri = Url::parse(properties.uri()).unwrap();
        uri.set_path("/sync/watched/movies");

        let response = self
            .client
            .get(uri)
            .bearer_auth(bearer_token)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to retrieve watched movies, {}", e);
                TrackingError::Request
            })?
            .json::<Vec<WatchedMovie>>()
            .await
            .map_err(|e| {
                error!("Failed to parse movies, {}", e);
                TrackingError::Parsing
            })?;

        trace!("Mapping tracking movie response {:?}", response);
        Ok(response
            .into_iter()
            .map(|e| Box::new(e) as Box<dyn MediaIdentifier>)
            .collect())
    }
}

impl Debug for TraktProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraktProvider")
            .field("config", &self.config)
            .field("oauth_client", &self.oauth_client)
            .field("client", &self.client)
            .field("runtime", &self.runtime)
            .field("callbacks", &self.callbacks)
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
    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;
    use reqwest::Client;
    use reqwest::header::CONTENT_TYPE;
    use tempfile::tempdir;
    use url::Url;

    use popcorn_fx_core::core::config::{PopcornProperties, PopcornSettings, TrackingSettings};
    use popcorn_fx_core::core::media::MediaType;
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    const HEADER_APPLICATION_JSON: &str = "application/json";

    #[test]
    fn test_new() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());

        let result = TraktProvider::new(settings, runtime);

        if let Err(e) = result {
            assert!(false, "failed to create new Trakt instance, {}", e)
        }
    }

    #[test]
    fn test_is_authorized() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let settings = Arc::new(
            ApplicationConfig::builder()
                .storage(temp_path)
                .settings(PopcornSettings {
                    subtitle_settings: Default::default(),
                    ui_settings: Default::default(),
                    server_settings: Default::default(),
                    torrent_settings: Default::default(),
                    playback_settings: Default::default(),
                    tracking_settings: Default::default(),
                })
                .build(),
        );
        settings.update_tracker(
            TRACKING_NAME,
            Tracker {
                access_token: "".to_string(),
                expires_in: None,
                refresh_token: None,
                scopes: None,
            },
        );
        let trakt = TraktProvider::new(settings, runtime).unwrap();

        let result = trakt.is_authorized();

        assert!(result, "expected the tracker to have been authorized");
    }

    #[test]
    fn test_is_authorized_tracker_settings_not_present() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let settings = Arc::new(
            ApplicationConfig::builder()
                .storage(temp_path)
                .settings(PopcornSettings {
                    subtitle_settings: Default::default(),
                    ui_settings: Default::default(),
                    server_settings: Default::default(),
                    torrent_settings: Default::default(),
                    playback_settings: Default::default(),
                    tracking_settings: Default::default(),
                })
                .build(),
        );
        let trakt = TraktProvider::new(settings, runtime).unwrap();

        let result = trakt.is_authorized();

        assert!(!result, "expected the tracker to not have been authorized");
    }

    #[test]
    fn test_authorize() {
        init_logger();
        let expected_code = "MyAuthCodeResult";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/oauth/token")
                .header_exists("authorization");
            then.status(200)
                .header(CONTENT_TYPE.as_str(), HEADER_APPLICATION_JSON)
                .body(
                    r#"{
  "access_token": "dbaf9757982a9e738f05d249b7b5b4a266b3a139049317c4909f2f263572c781",
  "token_type": "bearer",
  "expires_in": 7200,
  "refresh_token": "76ba4c5c75c96f6087f58a4de10be6c00b29ea1ddc3b2022ee2016d1363e3a7c",
  "scope": "public",
  "created_at": 1487889741
}"#,
                );
        });
        let settings = Arc::new(
            ApplicationConfig::builder()
                .storage(temp_path)
                .properties(PopcornProperties {
                    loggers: Default::default(),
                    update_channel: "".to_string(),
                    providers: Default::default(),
                    enhancers: Default::default(),
                    subtitle: Default::default(),
                    tracking: vec![(
                        "trakt".to_string(),
                        TrackingProperties {
                            uri: server.base_url(),
                            client: TrackingClientProperties {
                                client_id: "SomeClientId".to_string(),
                                client_secret: "SomeClientSecret".to_string(),
                                user_authorization_uri: server.url("/oauth/authorize"),
                                access_token_uri: server.url("/oauth/token"),
                            },
                        },
                    )]
                    .into_iter()
                    .collect(),
                })
                .build(),
        );
        let (tx, rx) = channel();
        let trakt = TraktProvider::new(settings, runtime).unwrap();

        trakt.register_open_authorization(Box::new(move |uri| {
            tx.send(uri).unwrap();
            true
        }));

        trakt.runtime.spawn(async move {
            let client = Client::new();
            let authorization_uri = rx.recv_timeout(Duration::from_secs(1)).unwrap();
            let auth_uri = Url::parse(authorization_uri.as_str())
                .expect("expected the authorization open to have been invoked");

            let (_, state) = auth_uri
                .query_pairs()
                .find(|(key, _)| key == "state")
                .expect("expected the state key to have been found");
            let (_, redirect_uri) = auth_uri
                .query_pairs()
                .find(|(key, _)| key == "redirect_uri")
                .expect("expected the redirect uri to have been present");

            let uri = Url::parse(redirect_uri.as_ref())
                .expect("expected a valid redirect uri")
                .query_pairs_mut()
                .append_pair("code", expected_code)
                .append_pair("state", state.as_ref())
                .finish()
                .to_string();

            if let Err(e) = client.get(uri).send().await {
                assert!(false, "expected the callback to have succeeded, {}", e)
            }
        });

        let result = block_in_place(trakt.authorize());

        if let Err(e) = result {
            assert!(false, "expected the authorization to have succeeded, {}", e);
        }

        let result = trakt
            .config
            .user_settings()
            .tracking()
            .tracker(TRACKING_NAME)
            .unwrap();

        assert_ne!(String::new(), result.access_token);
        mock.assert_hits(1);
    }

    #[test]
    fn test_disconnect() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        settings.update_tracker(
            TRACKING_NAME,
            Tracker {
                access_token: "SomeRandomToken".to_string(),
                expires_in: None,
                refresh_token: None,
                scopes: None,
            },
        );
        let trakt = TraktProvider::new(settings, runtime).unwrap();

        let settings = trakt.config.user_settings().tracking_settings;
        assert!(
            settings.tracker(TRACKING_NAME).is_some(),
            "expected the tracker info to have been present"
        );
        block_in_place(trakt.disconnect());

        let settings = trakt.config.user_settings().tracking_settings;
        let result = settings.tracker(TRACKING_NAME);
        assert!(
            result.is_none(),
            "expected the tracker info to have been removed"
        );
    }

    #[test]
    fn test_watched_movies() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/sync/watched/movies")
                .header_exists("Authorization");
            then.status(200)
                .header("Content-Type", HEADER_APPLICATION_JSON)
                .body(
                    r#"[{
    "plays": 4,
    "last_watched_at": "2014-10-11T17:00:54.000Z",
    "last_updated_at": "2014-10-11T17:00:54.000Z",
    "movie": {
      "title": "Batman Begins",
      "year": 2005,
      "ids": {
        "trakt": 6,
        "slug": "batman-begins-2005",
        "imdb": "tt0372784",
        "tmdb": 272
      }
    }
}]"#,
                );
        });
        let settings = Arc::new(
            ApplicationConfig::builder()
                .storage(temp_path)
                .properties(PopcornProperties {
                    loggers: Default::default(),
                    update_channel: Default::default(),
                    providers: Default::default(),
                    enhancers: Default::default(),
                    subtitle: Default::default(),
                    tracking: vec![(
                        "trakt".to_string(),
                        TrackingProperties {
                            uri: server.base_url(),
                            client: TrackingClientProperties {
                                client_id: "Foo".to_string(),
                                client_secret: "Bar".to_string(),
                                user_authorization_uri: server.url("/oauth/authorize"),
                                access_token_uri: server.url("/oauth/token"),
                            },
                        },
                    )]
                    .into_iter()
                    .collect(),
                })
                .settings(PopcornSettings {
                    subtitle_settings: Default::default(),
                    ui_settings: Default::default(),
                    server_settings: Default::default(),
                    torrent_settings: Default::default(),
                    playback_settings: Default::default(),
                    tracking_settings: TrackingSettings::builder()
                        .tracker(
                            TRACKING_NAME,
                            Tracker {
                                access_token: "MyAccessToken".to_string(),
                                expires_in: None,
                                refresh_token: None,
                                scopes: None,
                            },
                        )
                        .build(),
                })
                .build(),
        );
        let trakt = TraktProvider::new(settings, runtime).unwrap();

        let result = block_in_place(trakt.watched_movies());

        if let Ok(result) = result {
            let result = result.get(0).unwrap();

            mock.assert_hits(1);
            assert_eq!("tt0372784", result.imdb_id());
            assert_eq!(MediaType::Movie, result.media_type());
            assert_eq!("Batman Begins", result.title());
        } else {
            assert!(false, "expected Result::Ok, but got {:?} instead", result);
        }
    }
}
