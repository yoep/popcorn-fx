use crossterm::event::{Event, EventStream};
use futures::future::pending;
use futures::FutureExt;
use futures::StreamExt;
use fx_callback::{Callback, Subscription};
use popcorn_fx_torrent::torrent::{
    format_bytes, FxTorrentSession, InfoHash, Session, SessionEvent, SessionState, Torrent,
    TorrentEvent, TorrentFlags, TorrentState,
};
use ratatui::layout::Constraint::{Length, Min};
use ratatui::layout::Layout;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{env, io};
use tokio::{select, time};

#[tokio::main]
async fn main() -> io::Result<()> {
    let torrent_uri = env::args().nth(1).ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "expected a torrent uri to have been provided",
    ))?;
    let mut app = App::new()?;

    let result = select! {
        _ = tokio::signal::ctrl_c() => Ok(()),
        result = app.run(torrent_uri.as_str()) => result,
    };

    ratatui::restore();
    result
}

#[derive(Debug)]
struct App {
    /// The data of the application
    data: AppData,
    /// The underlying torrent session used by the app
    session: FxTorrentSession,
    /// The underlying torrent being downloaded by the app
    torrent: Option<Torrent>,
    /// The event subscription of the torrent session
    session_event_receiver: Subscription<SessionEvent>,
    /// The event subscription of the torrent
    torrent_event_receiver: Option<Subscription<TorrentEvent>>,
    /// The `Ratatui` terminal
    terminal: DefaultTerminal,
}

impl App {
    fn new() -> io::Result<Self> {
        let data = AppData::default();
        let session = FxTorrentSession::builder()
            .client_name("FX torrent")
            .base_path("torrents")
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let session_event_receiver = session.subscribe();

        Ok(Self {
            data,
            session,
            torrent: None,
            session_event_receiver,
            torrent_event_receiver: None,
            terminal: ratatui::init(),
        })
    }

    async fn run(&mut self, torrent_uri: &str) -> io::Result<()> {
        let mut reader = EventStream::new();

        loop {
            self.draw().await?;

            select! {
                _ = time::sleep(Duration::from_millis(100)) => {},
                event = reader.next().fuse() => handle_event(event).await,
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
            TorrentEvent::PeerConnected(_) => {}
            TorrentEvent::PeerDisconnected(_) => {}
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
            TorrentEvent::Stats(_) => {}
        }
    }

    async fn draw(&mut self) -> io::Result<()> {
        self.terminal
            .draw(|frame| Self::do_draw(frame, &self.data))?;
        Ok(())
    }

    fn subscribe_to_torrent(&mut self, torrent: &Torrent) {
        self.torrent_event_receiver = Some(torrent.subscribe());
    }

    fn do_draw(frame: &mut Frame<'_>, data: &AppData) {
        let main = Layout::vertical([Length(1), Min(0)]);
        let [title_area, metadata_area] = main.areas(frame.area());

        frame.render_widget(
            Block::bordered().title(format!(" FX Torrent - {:?} ", data.session_state)),
            title_area,
        );

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(format!(
                    "Name: {}",
                    Self::print_optional_string(data.torrent_name.as_ref())
                )),
                Line::from(format!(
                    "State: {}",
                    Self::print_optional_string(data.torrent_state.as_ref())
                )),
                Line::from(format!(
                    "Path: {}",
                    Self::print_optional_string(
                        data.torrent_path.as_ref().and_then(|e| e.to_str())
                    )
                )),
                Line::from(format!(
                    "Size: {}",
                    Self::print_optional_string(
                        data.torrent_size.as_ref().map(|e| *e).map(format_bytes)
                    )
                )),
                Line::from(format!(
                    "Info hash: {}",
                    Self::print_optional_string(data.torrent_info_hash.as_ref())
                )),
                Line::from(format!(
                    "Pieces: {}/{}",
                    data.torrent_completed_pieces, data.torrent_total_pieces
                )),
                Line::from(format!("Files: {}", data.torrent_total_files)),
                Line::from(format!("Connected peers: {}", data.torrent_peers)),
            ])
            .block(Block::bordered().title(" Metadata ")),
            metadata_area,
        );
    }

    fn print_optional_string<S: ToString>(value: Option<S>) -> String {
        value
            .as_ref()
            .map(|e| e.to_string())
            .unwrap_or(String::default())
    }
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
        }
    }
}

async fn handle_event(event: Option<io::Result<Event>>) {
    match event {
        Some(Ok(event)) => if let Event::Key(key) = event {},
        _ => {}
    }
}
