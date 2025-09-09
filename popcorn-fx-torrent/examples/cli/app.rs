use crate::app_logger::AppLogger;
use crate::dht_info::{DhtInfoWidget, DHT_INFO_WIDGET_NAME};
use crate::menu::MenuWidget;
use crate::torrent_info::TorrentInfoWidget;
use async_trait::async_trait;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use futures::{future, FutureExt};
use fx_callback::{Callback, Subscription};
use fx_handle::Handle;
use log::{error, warn};
use popcorn_fx_torrent::torrent::operation::{
    TorrentConnectPeersOperation, TorrentCreateFilesOperation, TorrentCreatePiecesOperation,
    TorrentDhtNodesOperation, TorrentDhtPeersOperation, TorrentFileValidationOperation,
    TorrentMetadataOperation, TorrentTrackersOperation,
};
use popcorn_fx_torrent::torrent::{
    FxSessionCache, FxTorrentSession, Session, SessionEvent, SessionState, TorrentFlags,
    TorrentOperationFactory,
};
use ratatui::layout::Constraint::{Length, Min};
use ratatui::layout::{Alignment, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Tabs, Widget};
use ratatui::{DefaultTerminal, Frame};
use std::fmt::Debug;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

pub const APP_CLIENT_NAME: &str = "FX torrent";
pub const APP_DEFAULT_STORAGE: &str = "torrents";
const APP_QUIT_KEY: char = 'q';
const TAB_NAME_LEN: usize = 16;
const SESSION_CACHE_LIMIT: usize = 10;
const RENDER_INTERVAL: Duration = Duration::from_millis(200);

/// The app command sender type.
pub type AppCommandSender = UnboundedSender<AppCommand>;

#[async_trait]
pub trait FXWidget: Debug {
    /// Get the unique handle of the widget.
    fn handle(&self) -> Handle;

    /// Get the name of the widget.
    fn name(&self) -> &str;

    /// Execute a widget tick that allows to process events.
    async fn tick(&mut self);

    /// Handle the specified key event within this widget.
    fn on_key_event(&mut self, key: FXKeyEvent);

    /// Handle a paste event within this widget.
    fn on_paste_event(&mut self, text: String);

    /// Render this widget for the given frame and area.
    fn render(&self, frame: &mut Frame, area: Rect);
}

#[derive(Debug, Clone)]
pub struct FXKeyEvent {
    inner: Arc<InnerFxKeyEvent>,
}

impl FXKeyEvent {
    pub fn code(&self) -> KeyCode {
        self.inner.event.code
    }

    /// Check if the event is consumed.
    /// If not, propagation is allowed, else stop.
    pub fn is_consumed(&self) -> bool {
        if let Ok(consumed) = self.inner.consumed.lock() {
            *consumed
        } else {
            false
        }
    }

    /// Marks this event as consumed. This stops its further propagation.
    pub fn consume(&mut self) {
        if let Ok(mut consumed) = self.inner.consumed.lock() {
            *consumed = true;
        }
    }
}

impl From<KeyEvent> for FXKeyEvent {
    fn from(value: KeyEvent) -> Self {
        Self {
            inner: Arc::new(InnerFxKeyEvent {
                event: value,
                consumed: Default::default(),
            }),
        }
    }
}

#[derive(Debug)]
struct InnerFxKeyEvent {
    event: KeyEvent,
    consumed: Mutex<bool>,
}

#[derive(Debug)]
pub struct App {
    tabs: Vec<Box<dyn FXWidget>>,
    selected_tab_handle: Handle,
    session_state: SessionState,
    session: FxTorrentSession,
    session_event_receiver: Subscription<SessionEvent>,
    settings: AppSettings,
    app_command_receiver: UnboundedReceiver<AppCommand>,
    cancellation_token: CancellationToken,
}

impl App {
    pub fn new(logger: AppLogger) -> io::Result<Self> {
        let settings = AppSettings::default();
        let session = Self::create_session(&settings)?;
        let session_event_receiver = session.subscribe();
        let (app_sender, app_receiver) = unbounded_channel();
        let menu = MenuWidget::new(app_sender, logger);
        let menu_handle = menu.handle();

        Ok(Self {
            tabs: vec![Box::new(menu)],
            selected_tab_handle: menu_handle,
            session_state: SessionState::Initializing,
            session,
            session_event_receiver,
            settings,
            app_command_receiver: app_receiver,
            cancellation_token: Default::default(),
        })
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        let mut reader = EventStream::new();

        loop {
            terminal.draw(|frame| self.render(frame))?;

            select! {
                _ = self.cancellation_token.cancelled() => return Ok(()),
                _ = time::sleep(RENDER_INTERVAL) => {},
                event = reader.next().fuse() => self.handle_event(event).await,
                Some(command) = self.app_command_receiver.recv() => self.handle_command(command).await,
                Some(event) = self.session_event_receiver.recv() => self.handle_session_event(&*event).await,
            }

            // tick all widgets, which allows them to process events
            future::join_all(self.tabs.iter_mut().map(|e| e.tick()).collect::<Vec<_>>()).await;
        }
    }

    async fn handle_session_event(&mut self, event: &SessionEvent) {
        match event {
            SessionEvent::TorrentAdded(_) => {}
            SessionEvent::TorrentRemoved(_) => {}
            SessionEvent::StateChanged(state) => {
                self.session_state = *state;
            }
        }
    }

    async fn handle_command(&mut self, command: AppCommand) {
        match command {
            AppCommand::AddTorrentUri(uri) => self.add_torrent_uri(uri.as_str()).await,
            AppCommand::DhtEnabled(enabled) => self.update_dht(enabled),
            AppCommand::TrackerEnabled(enabled) => self.update_trackers(enabled),
            AppCommand::Storage(location) => self.update_storage(location).await,
            AppCommand::DhtInfo(app_sender) => self.show_dht_info(app_sender).await,
            AppCommand::Quit => self.cancellation_token.cancel(),
        }
    }

    fn selected_tab(&mut self) -> (usize, &mut Box<dyn FXWidget>) {
        let selected_tab = self
            .tabs
            .iter()
            .position(|e| &e.handle() == &self.selected_tab_handle)
            .unwrap_or(0);

        (
            selected_tab,
            self.tabs
                .iter_mut()
                .nth(selected_tab)
                .expect("expected the tab to exist"),
        )
    }

    fn select_tab(&mut self, tab_index: usize) {
        self.selected_tab_handle = self
            .tabs
            .iter()
            .nth(tab_index)
            .or_else(|| self.tabs.iter().nth(0))
            .map(|e| e.handle())
            .expect("expected the tab to exist");
    }

    async fn handle_event(&mut self, event: Option<io::Result<Event>>) {
        match event {
            Some(Ok(event)) => match event {
                Event::Key(key) => {
                    let (_, selected_tab) = self.selected_tab();
                    let event = FXKeyEvent::from(key);

                    // invoke the event within the active tab
                    selected_tab.on_key_event(event.clone());

                    // check if the event was consumed
                    if !event.is_consumed() {
                        match event.code() {
                            KeyCode::Char(APP_QUIT_KEY) => self.cancellation_token.cancel(),
                            KeyCode::Left => {
                                let position = self.selected_tab().0.saturating_sub(1);
                                self.select_tab(position);
                            }
                            KeyCode::Right => {
                                let mut position = self.selected_tab().0.saturating_add(1);
                                if position > self.tabs.len() {
                                    position = self.tabs.len() - 1;
                                }

                                self.select_tab(position);
                            }
                            _ => {}
                        }
                    }
                }
                Event::Paste(text) => {
                    let (_, selected_tab) = self.selected_tab();
                    selected_tab.on_paste_event(text);
                }
                _ => {}
            },
            _ => {}
        }
    }

    async fn add_torrent_uri(&mut self, uri: &str) {
        match self
            .session
            .add_torrent_from_uri(uri, TorrentFlags::default() | TorrentFlags::UploadMode)
            .await
        {
            Ok(torrent) => match torrent.metadata().await {
                Ok(metadata) => {
                    let name = metadata
                        .info
                        .as_ref()
                        .map(|info| info.name())
                        .unwrap_or_else(|| {
                            metadata
                                .name()
                                .map(|e| e.to_string())
                                .unwrap_or("<unknown>".to_string())
                        });
                    self.tabs
                        .push(Box::new(TorrentInfoWidget::new(&name, torrent)));
                }
                Err(e) => {
                    warn!("Torrent uri {} has been dropped too early, {}", uri, e);
                }
            },
            Err(e) => {
                error!("Failed to add torrent {}: {}", uri, e);
            }
        }
    }

    fn update_dht(&mut self, enabled: bool) {
        self.settings.dht_enabled = enabled;
        self.remove_dht_info();
        match Self::create_session(&self.settings) {
            Ok(session) => {
                self.session = session;
            }
            Err(e) => error!("Failed to create session: {}", e),
        }
    }

    fn update_trackers(&mut self, enabled: bool) {
        self.settings.trackers_enabled = enabled;
        self.remove_dht_info();
        match Self::create_session(&self.settings) {
            Ok(session) => {
                self.session = session;
            }
            Err(e) => error!("Failed to create session: {}", e),
        }
    }

    async fn update_storage(&self, location: PathBuf) {
        self.session.set_base_path(location).await;
    }

    async fn show_dht_info(&mut self, app_sender: AppCommandSender) {
        if !self.tabs.iter().any(|e| e.name() == DHT_INFO_WIDGET_NAME) {
            if let Some(dht) = self.session.dht().await {
                let nodes = dht.nodes().await;
                self.tabs
                    .insert(1, Box::new(DhtInfoWidget::new(dht, nodes, app_sender)));
            } else {
                return;
            }
        }

        self.select_tab(1);
    }

    fn remove_dht_info(&mut self) {
        self.tabs.retain(|e| e.name() != DHT_INFO_WIDGET_NAME)
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let main = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, content_area, footer_area] = main.areas(area);
        let header = Layout::horizontal([Min(0), Min(10)]);
        let [tabs_area, title_area] = header.areas(header_area);
        let session_state = self.session_state.to_string();
        let titles: Vec<String> = self
            .tabs
            .iter()
            .map(|e| {
                let name = e.name();
                let len = name.len().min(TAB_NAME_LEN);

                format!("  {}  ", name.chars().take(len).collect::<String>())
            })
            .collect();
        let (selected_tab_index, selected_tab) = self.selected_tab();

        // render the header
        Tabs::new(titles)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(tabs_area, frame.buffer_mut());
        Paragraph::new(format!("FX Torrent - {}", session_state))
            .alignment(Alignment::Right)
            .render(title_area, frame.buffer_mut());

        // render the contents
        selected_tab.render(frame, content_area);

        // render the footer
        Line::raw(format!(
            "◄ ► to change tab | Press {} to quit",
            APP_QUIT_KEY
        ))
        .centered()
        .render(footer_area, frame.buffer_mut());
    }

    fn create_session(settings: &AppSettings) -> io::Result<FxTorrentSession> {
        let mut operations: Vec<TorrentOperationFactory> = vec![
            || Box::new(TorrentConnectPeersOperation::new()),
            || Box::new(TorrentMetadataOperation::new()),
            || Box::new(TorrentCreatePiecesOperation::new()),
            || Box::new(TorrentCreateFilesOperation::new()),
            || Box::new(TorrentFileValidationOperation::new()),
        ];

        if settings.dht_enabled {
            operations.insert(0, || Box::new(TorrentDhtNodesOperation::new()));
            operations.insert(1, || Box::new(TorrentDhtPeersOperation::new()));
        }

        if settings.trackers_enabled {
            operations.insert(0, || Box::new(TorrentTrackersOperation::new()));
        }

        FxTorrentSession::builder()
            .client_name(APP_CLIENT_NAME)
            .base_path(&settings.storage)
            .session_cache(FxSessionCache::new(SESSION_CACHE_LIMIT))
            .operations(operations)
            .dht(settings.dht_enabled)
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

#[derive(Debug)]
pub enum AppCommand {
    /// Try to add the given torrent uri to the app
    AddTorrentUri(String),
    /// Set if DHT is enabled
    DhtEnabled(bool),
    /// Set if trackers are enabled
    TrackerEnabled(bool),
    /// Set the new storage location of the session
    Storage(PathBuf),
    /// Show the DHT info widget
    DhtInfo(AppCommandSender),
    /// Quit the app
    Quit,
}

#[derive(Debug, Clone)]
struct AppSettings {
    storage: PathBuf,
    dht_enabled: bool,
    trackers_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            storage: PathBuf::from(APP_DEFAULT_STORAGE),
            dht_enabled: true,
            trackers_enabled: true,
        }
    }
}
