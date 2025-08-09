use crossterm::event::{Event, EventStream, KeyCode};
use futures::future::pending;
use futures::FutureExt;
use futures::StreamExt;
use fx_callback::{Callback, Subscription};
use popcorn_fx_torrent::torrent::{
    format_bytes, FxSessionCache, FxTorrentSession, InfoHash, Session, SessionEvent, SessionState,
    Torrent, TorrentEvent, TorrentFlags, TorrentState,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::Line;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Gauge, Paragraph, Sparkline, Widget};
use ratatui::DefaultTerminal;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

const SESSION_CACHE_LIMIT: usize = 10;

#[derive(Debug)]
pub struct App {
    /// The data of the application
    data: AppData,
    /// The logs of the application
    logs: Vec<String>,
    /// The underlying torrent session used by the app
    session: FxTorrentSession,
    /// The underlying torrent being downloaded by the app
    torrent: Option<Torrent>,
    /// The event subscription of the torrent session
    session_event_receiver: Subscription<SessionEvent>,
    /// The event subscription of the torrent
    torrent_event_receiver: Option<Subscription<TorrentEvent>>,
    cancellation_token: CancellationToken,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let data = AppData::default();
        let session = FxTorrentSession::builder()
            .client_name("FX torrent")
            .base_path("torrents")
            .session_cache(FxSessionCache::new(SESSION_CACHE_LIMIT))
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let session_event_receiver = session.subscribe();

        Ok(Self {
            data,
            logs: vec![],
            session,
            torrent: None,
            session_event_receiver,
            torrent_event_receiver: None,
            cancellation_token: Default::default(),
        })
    }

    pub async fn run(
        &mut self,
        mut terminal: DefaultTerminal,
        mut command_receiver: UnboundedReceiver<AppCommand>,
        torrent_uri: &str,
    ) -> io::Result<()> {
        let mut reader = EventStream::new();

        loop {
            terminal.draw(|frame| frame.render_widget(&*self, frame.area()))?;

            select! {
                _ = self.cancellation_token.cancelled() => return Ok(()),
                _ = time::sleep(Duration::from_millis(100)) => {},
                event = reader.next().fuse() => self.handle_event(event).await,
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
                Some(event) = self.session_event_receiver.recv() => self.handle_session_event(&*event, torrent_uri).await,
                Some(event) = async {
                    match self.torrent_event_receiver.as_mut() {
                        Some(receiver) => receiver.recv().await,
                        None => pending::<Option<Arc<TorrentEvent>>>().await,
                    }
                } => self.handle_torrent_event(&*event).await,
            }
        }
    }

    async fn handle_session_event(&mut self, event: &SessionEvent, torrent_uri: &str) {
        match event {
            SessionEvent::TorrentAdded(handle) => {
                if let Some(torrent) = self.session.find_torrent_by_handle(handle).await {
                    if let Ok(metadata) = torrent.metadata().await {
                        self.data.torrent_name = metadata.name().map(|e| e.to_string());
                        self.data.torrent_info_hash = Some(metadata.info_hash);
                    }

                    self.subscribe_to_torrent(&torrent);
                }
            }
            SessionEvent::TorrentRemoved(_) => {}
            SessionEvent::StateChanged(state) => {
                self.data.session_state = *state;

                if *state == SessionState::Running {
                    if let Ok(torrent) = self
                        .session
                        .add_torrent_from_uri(torrent_uri, TorrentFlags::none())
                        .await
                    {
                        self.torrent = Some(torrent);
                    }
                }
            }
        }
    }

    async fn handle_torrent_event(&mut self, event: &TorrentEvent) {
        match event {
            TorrentEvent::StateChanged(state) => {
                self.data.torrent_state = Some(*state);
            }
            TorrentEvent::MetadataChanged(metadata) => {
                self.data.torrent_name = metadata.name().map(|e| e.to_string());
                self.data.torrent_info_hash = Some(metadata.info_hash.clone());

                if let Some(torrent) = self.torrent.as_ref() {
                    self.data.torrent_path = torrent.path().await;
                }
                if let Some(info) = metadata.info.as_ref() {
                    self.data.torrent_size = Some(info.len());
                }
            }
            TorrentEvent::PeerConnected(_) => {
                if let Some(torrent) = self.torrent.as_ref() {
                    self.data.torrent_peers = torrent.active_peer_connections().await;
                }
            }
            TorrentEvent::PeerDisconnected(_) => {
                if let Some(torrent) = self.torrent.as_ref() {
                    self.data.torrent_peers = torrent.active_peer_connections().await;
                }
            }
            TorrentEvent::TrackersChanged => {}
            TorrentEvent::PiecesChanged(total_pieces) => {
                self.data.torrent_total_pieces = *total_pieces;
            }
            TorrentEvent::PiecePrioritiesChanged => {}
            TorrentEvent::PieceCompleted(_) => {
                if let Some(torrent) = self.torrent.as_ref() {
                    self.data.torrent_completed_pieces = torrent.total_completed_pieces().await;
                }
            }
            TorrentEvent::FilesChanged => {
                if let Some(torrent) = self.torrent.as_ref() {
                    self.data.torrent_total_files = torrent.total_files().await.unwrap_or(0);
                }
            }
            TorrentEvent::OptionsChanged => {}
            TorrentEvent::Stats(stats) => {
                self.data.torrent_progress = stats.progress();
                self.data.torrent_down.push(stats.download_rate);
                self.data.torrent_up.push(stats.upload_rate);

                if self.data.torrent_down.len() > 100 {
                    self.data.torrent_down.remove(0);
                }
                if self.data.torrent_up.len() > 100 {
                    self.data.torrent_up.remove(0);
                }
            }
        }
    }

    async fn handle_command(&mut self, command: AppCommand) {
        match command {
            AppCommand::Log(log) => {
                self.logs.push(log);
                if self.logs.len() > 10 {
                    self.logs.remove(0);
                }
            }
        }
    }

    fn subscribe_to_torrent(&mut self, torrent: &Torrent) {
        self.torrent_event_receiver = Some(torrent.subscribe());
    }

    async fn handle_event(&self, event: Option<io::Result<Event>>) {
        match event {
            Some(Ok(event)) => {
                if let Event::Key(key) = event {
                    if key.code == KeyCode::Char('q') {
                        self.cancellation_token.cancel();
                    }
                }
            }
            _ => {}
        }
    }

    fn print_optional_string<S: ToString>(value: Option<S>) -> String {
        value
            .as_ref()
            .map(|e| e.to_string())
            .unwrap_or(String::default())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let main = Layout::vertical([Length(1), Min(0), Length(4), Min(0)]);
        let [title_area, header_area, progress_area, log_area] = main.areas(area);
        let header = Layout::horizontal([Percentage(50), Percentage(50)]);
        let [metadata_area, performance_area] = header.areas(header_area);
        let performance = Layout::vertical([Percentage(50), Percentage(50)]);
        let [down_performance, up_performance] = performance.areas(performance_area);

        // render the title
        Block::bordered()
            .title(format!(" FX Torrent - {:?} ", self.data.session_state))
            .render(title_area, buf);

        // render the metadata
        Paragraph::new(vec![
            Line::from(format!(
                "Name: {}",
                App::print_optional_string(self.data.torrent_name.as_ref())
            )),
            Line::from(format!(
                "State: {}",
                App::print_optional_string(self.data.torrent_state.as_ref())
            )),
            Line::from(format!(
                "Path: {}",
                App::print_optional_string(
                    self.data.torrent_path.as_ref().and_then(|e| e.to_str())
                )
            )),
            Line::from(format!(
                "Size: {}",
                App::print_optional_string(
                    self.data
                        .torrent_size
                        .as_ref()
                        .map(|e| *e)
                        .map(format_bytes)
                )
            )),
            Line::from(format!(
                "Info hash: {}",
                App::print_optional_string(self.data.torrent_info_hash.as_ref())
            )),
            Line::from(format!(
                "Pieces: {}/{}",
                self.data.torrent_completed_pieces, self.data.torrent_total_pieces
            )),
            Line::from(format!("Files: {}", self.data.torrent_total_files)),
            Line::from(format!("Connected peers: {}", self.data.torrent_peers)),
        ])
        .block(Block::bordered().title(" Metadata "))
        .render(metadata_area, buf);

        // render the performance
        Sparkline::default()
            .block(Block::bordered().title(format!(
                    "Down: {}/s",
                    format_bytes(
                        self.data
                            .torrent_down
                            .last()
                            .map(|e| *e as usize)
                            .unwrap_or(0)
                    )
                )))
            .data(&self.data.torrent_down)
            .style(Style::default().fg(Color::Yellow))
            .render(down_performance, buf);

        Sparkline::default()
            .block(Block::bordered().title(format!(
                "Up: {}/s",
                format_bytes(
                    self.data
                        .torrent_up
                        .last()
                        .map(|e| *e as usize)
                        .unwrap_or(0)
                )
            )))
            .data(&self.data.torrent_up)
            .style(Style::default().fg(Color::Yellow))
            .render(up_performance, buf);

        // render the progress
        Gauge::default()
            .block(Block::bordered().title("Progress"))
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio(self.data.torrent_progress as f64)
            .label(format!("{:.1}%", self.data.torrent_progress * 100f32))
            .render(progress_area, buf);

        // render the logs
        Paragraph::new(
            self.logs
                .iter()
                .map(|l| Line::from(l.clone()))
                .collect::<Vec<_>>(),
        )
        .block(Block::bordered().title(" Logs "))
        .render(log_area, buf);
    }
}

#[derive(Debug)]
pub enum AppCommand {
    Log(String),
}

#[derive(Debug)]
struct AppData {
    session_state: SessionState,
    torrent_name: Option<String>,
    torrent_path: Option<PathBuf>,
    torrent_state: Option<TorrentState>,
    torrent_size: Option<usize>,
    torrent_info_hash: Option<InfoHash>,
    torrent_total_pieces: usize,
    torrent_completed_pieces: usize,
    torrent_total_files: usize,
    torrent_peers: usize,
    torrent_progress: f32,
    torrent_down: Vec<u64>,
    torrent_up: Vec<u64>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            session_state: SessionState::Initializing,
            torrent_name: None,
            torrent_path: None,
            torrent_state: None,
            torrent_size: None,
            torrent_info_hash: None,
            torrent_total_pieces: 0,
            torrent_completed_pieces: 0,
            torrent_total_files: 0,
            torrent_peers: 0,
            torrent_progress: 0.0,
            torrent_down: vec![],
            torrent_up: vec![],
        }
    }
}
