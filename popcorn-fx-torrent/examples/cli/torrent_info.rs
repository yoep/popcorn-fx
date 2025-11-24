use crate::app::{FXKeyEvent, FXWidget, PERFORMANCE_HISTORY};
use crate::widget::{print_optional_string, print_string_len};
use async_trait::async_trait;
use crossterm::event::KeyCode;
use fx_callback::{Callback, Subscription};
use log::{info, warn};
use popcorn_fx_torrent::torrent::peer::{Peer, PeerClientInfo, PeerEvent, PeerHandle, PeerState};
use popcorn_fx_torrent::torrent::{
    format_bytes, File, FileIndex, FilePriority, InfoHash, PieceIndex, Torrent, TorrentEvent,
    TorrentPeer, TorrentState,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Fill, Length, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Alignment, Color, Style};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::block::{Position, Title};
use ratatui::widgets::{
    Block, Borders, Cell, Gauge, HighlightSpacing, List, ListItem, ListState, Paragraph, Row,
    Sparkline, StatefulWidget, Table, TableState, Widget,
};
use ratatui::Frame;
use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

const REMOVE_CLOSED_PEER_AFTER: Duration = Duration::from_secs(3);

#[derive(Debug)]
pub struct TorrentInfoWidget {
    name: String,
    torrent: Torrent,
    files_widget: TorrentFilesWidget,
    priorities_widget: TorrentFilePriorityWidget,
    peers_widget: TorrentPeersWidget,
    event_receiver: Subscription<TorrentEvent>,
    command_sender: UnboundedSender<TorrentInfoCommand>,
    command_receiver: UnboundedReceiver<TorrentInfoCommand>,
    data: TorrentData,
    state: Mutex<TorrentInfoState>,
}

impl TorrentInfoWidget {
    pub async fn new(name: &str, torrent: Torrent) -> Self {
        let event_receiver = torrent.subscribe();
        let (command_sender, command_receiver) = unbounded_channel();
        let data = if let Some(metadata) = torrent.metadata().await.ok() {
            TorrentData {
                info_hash: Some(metadata.info_hash.clone()),
                path: torrent.path().await,
                state: None,
                total_pieces: 0,
                completed_pieces: 0,
                wanted_size: metadata
                    .info
                    .as_ref()
                    .map(|e| e.len() as u64)
                    .unwrap_or_default(),
                wanted_completed_size: 0,
                total_files: 0,
                peers: 0,
                progress: 0.0,
                wasted: 0,
                down: vec![],
                up: vec![],
            }
        } else {
            Default::default()
        };

        Self {
            name: name.to_string(),
            torrent,
            files_widget: TorrentFilesWidget::new(command_sender.clone()),
            priorities_widget: TorrentFilePriorityWidget::new(command_sender.clone()),
            peers_widget: TorrentPeersWidget::new(),
            event_receiver,
            command_sender,
            command_receiver,
            data,
            state: Default::default(),
        }
    }

    fn state(&self) -> TorrentInfoState {
        self.state
            .lock()
            .map(|e| *e)
            .unwrap_or(TorrentInfoState::Files)
    }

    async fn handle_event(&mut self, event: &TorrentEvent) {
        let data = &mut self.data;

        match event {
            TorrentEvent::StateChanged(state) => {
                data.state = Some(*state);
            }
            TorrentEvent::MetadataChanged(metadata) => {
                data.info_hash = Some(metadata.info_hash.clone());
                data.path = self.torrent.path().await;

                if let Some(name) = metadata.name().map(|e| e.to_string()) {
                    self.name = name;
                }
                if let Some(info) = metadata.info.as_ref() {
                    data.wanted_size = info.len() as u64;
                }
            }
            TorrentEvent::PeerConnected(peer) => {
                data.peers = self.torrent.active_peer_connections().await;

                if let Some(peer) = self.torrent.peer(&peer.handle).await {
                    self.peers_widget.add_peer(peer).await;
                } else {
                    warn!("Torrent {} failed to find peer {}", self.torrent, peer);
                }
            }
            TorrentEvent::PeerDisconnected(peer) => {
                data.peers = self.torrent.active_peer_connections().await;
                self.peers_widget.remove_peer(&peer.handle);
            }
            TorrentEvent::PiecesChanged(total_pieces) => {
                data.total_pieces = *total_pieces as u64;
            }
            TorrentEvent::PieceCompleted(piece) => {
                data.completed_pieces = self.torrent.total_completed_pieces().await as u64;
                self.files_widget.on_piece_completed(piece);
            }
            TorrentEvent::PiecePrioritiesChanged => {
                let files = self.torrent.files().await;
                self.files_widget.on_priorities_changed(
                    files
                        .into_iter()
                        .map(|e| (e.index, e.priority))
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
            }
            TorrentEvent::FilesChanged => {
                data.total_files = self.torrent.total_files().await.unwrap_or(0);
                self.files_widget
                    .on_files_changed(self.torrent.files().await);
            }
            TorrentEvent::Stats(stats) => {
                data.progress = stats.progress();
                data.wanted_completed_size = stats.wanted_completed_size.get();
                data.wasted = stats.wasted.total();
                data.down.push(stats.download.rate() as u64);
                data.up.push(stats.upload.rate() as u64);

                if data.down.len() > PERFORMANCE_HISTORY {
                    data.down.remove(0);
                }
                if data.up.len() > PERFORMANCE_HISTORY {
                    data.up.remove(0);
                }
                info!("Torrent {} stats {}", self.torrent, stats);
            }
            _ => {}
        }
    }

    async fn handle_command(&mut self, command: TorrentInfoCommand) {
        match command {
            TorrentInfoCommand::ShowFiles => {
                if let Ok(mut state) = self.state.lock() {
                    *state = TorrentInfoState::Files
                }
            }
            TorrentInfoCommand::ShowPriority(index, priority) => {
                self.priorities_widget.set_file(index);
                self.priorities_widget.select(priority);

                if let Ok(mut state) = self.state.lock() {
                    *state = TorrentInfoState::Priority
                }
            }
            TorrentInfoCommand::UpdatePriority(index, priority) => {
                self.torrent.prioritize_files(vec![(index, priority)]).await;
            }
            TorrentInfoCommand::TogglePaused => {
                if self.torrent.is_paused().await {
                    self.torrent.resume().await;
                } else {
                    self.torrent.pause().await;
                }
            }
        }
    }
}

#[async_trait]
impl FXWidget for TorrentInfoWidget {
    fn name(&self) -> &str {
        &self.name
    }

    async fn tick(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(&event).await;
        }
        while let Ok(command) = self.command_receiver.try_recv() {
            self.handle_command(command).await;
        }

        self.peers_widget.tick().await;
    }

    fn on_key_event(&mut self, mut key: FXKeyEvent) {
        if key.code() == KeyCode::Char('p') {
            key.consume();
            let _ = self.command_sender.send(TorrentInfoCommand::TogglePaused);
            return;
        }

        match self.state() {
            TorrentInfoState::Files => self.files_widget.on_key_event(key),
            TorrentInfoState::Priority => self.priorities_widget.on_key_event(key),
        }
    }

    fn on_paste_event(&mut self, _: String) {
        // no-op
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        Widget::render(self, area, frame.buffer_mut());
    }
}

impl Widget for &TorrentInfoWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let main = Layout::vertical([Length(12), Length(4), Fill(1)]);
        let [header_area, progress_area, details_area] = main.areas(area);
        let header = Layout::horizontal([Percentage(50), Percentage(50)]);
        let [metadata_area, performance_area] = header.areas(header_area);
        let performance = Layout::vertical([Percentage(50), Percentage(50)]);
        let [down_performance, up_performance] = performance.areas(performance_area);
        let details = Layout::horizontal([Percentage(60), Percentage(40)]);
        let [files_area, peers_area] = details.areas(details_area);

        let data = &self.data;

        // render the metadata
        Paragraph::new(vec![
            Line::from(vec![Span::from("Name: ").bold(), self.name.as_str().into()]),
            Line::from(vec![
                Span::from("State: ").bold(),
                print_optional_string(data.state.as_ref()).into(),
            ]),
            Line::from(vec![
                Span::from("Path: ").bold(),
                print_optional_string(self.data.path.as_ref().and_then(|e| e.to_str())).into(),
            ]),
            Line::from(vec![
                Span::from("Info hash: ").bold(),
                print_optional_string(self.data.info_hash.as_ref()).into(),
            ]),
            Line::from(vec![
                Span::from("Size: ").bold(),
                format!(
                    "{}/{}",
                    format_bytes(self.data.wanted_completed_size as usize),
                    format_bytes(self.data.wanted_size as usize)
                )
                .into(),
            ]),
            Line::from(vec![
                Span::from("Pieces: ").bold(),
                format!("{}/{}", self.data.completed_pieces, self.data.total_pieces).into(),
            ]),
            Line::from(vec![
                Span::from("Wasted: ").bold(),
                format_bytes(self.data.wasted as usize).into(),
            ]),
            Line::from(vec![
                Span::from("Files: ").bold(),
                self.data.total_files.to_string().into(),
            ]),
            Line::from(vec![
                Span::from("Connected peers: ").bold(),
                self.data.peers.to_string().into(),
            ]),
        ])
        .block(
            Block::bordered()
                .title(" Metadata ")
                .title(Title::from(" Press p to pause/resume ").position(Position::Bottom)),
        )
        .render(metadata_area, buf);

        // render the performance
        Sparkline::default()
            .block(Block::bordered().title(format!(
                "Down: {}/s",
                format_bytes(self.data.down.last().map(|e| *e as usize).unwrap_or(0))
            )))
            .data(&self.data.down)
            .style(Style::default().fg(Color::Yellow))
            .render(down_performance, buf);

        Sparkline::default()
            .block(Block::bordered().title(format!(
                "Up: {}/s",
                format_bytes(self.data.up.last().map(|e| *e as usize).unwrap_or(0))
            )))
            .data(&self.data.up)
            .style(Style::default().fg(Color::Yellow))
            .render(up_performance, buf);

        // render the progress
        Gauge::default()
            .block(
                Block::bordered()
                    .title("Progress")
                    .title_alignment(Alignment::Center),
            )
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio(self.data.progress as f64)
            .label(format!("{:.1}%", self.data.progress * 100f32))
            .render(progress_area, buf);

        // render the file details area
        match self.state() {
            TorrentInfoState::Files => self.files_widget.render(files_area, buf),
            TorrentInfoState::Priority => self.priorities_widget.render(files_area, buf),
        }
        // render the peers
        self.peers_widget.render(peers_area, buf);
    }
}

#[derive(Debug)]
struct TorrentData {
    path: Option<PathBuf>,
    state: Option<TorrentState>,
    info_hash: Option<InfoHash>,
    total_pieces: u64,
    completed_pieces: u64,
    wanted_size: u64,
    wanted_completed_size: u64,
    total_files: usize,
    peers: usize,
    progress: f32,
    wasted: u64,
    down: Vec<u64>,
    up: Vec<u64>,
}

impl Default for TorrentData {
    fn default() -> Self {
        Self {
            path: None,
            state: None,
            info_hash: None,
            wanted_size: 0,
            total_pieces: 0,
            completed_pieces: 0,
            wanted_completed_size: 0,
            total_files: 0,
            peers: 0,
            progress: 0.0,
            wasted: 0,
            down: vec![],
            up: vec![],
        }
    }
}

#[derive(Debug)]
struct TorrentFilesWidget {
    files: Vec<TorrentFileData>,
    state: Mutex<TableState>,
    command_sender: UnboundedSender<TorrentInfoCommand>,
}

impl TorrentFilesWidget {
    pub fn new(command_sender: UnboundedSender<TorrentInfoCommand>) -> Self {
        Self {
            files: vec![],
            state: Mutex::new(TableState::new().with_selected(0)),
            command_sender,
        }
    }

    fn selected_index(&self) -> usize {
        self.state
            .lock()
            .ok()
            .and_then(|e| e.selected())
            .unwrap_or(0)
    }

    fn selected(&self) -> &TorrentFileData {
        &self.files[self.selected_index()]
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        match key.code() {
            KeyCode::Up => {
                if let Ok(mut state) = self.state.lock() {
                    let offset = state.selected().unwrap_or(0).saturating_sub(1);
                    state.select(Some(offset));
                }
            }
            KeyCode::Down => {
                if let Ok(mut state) = self.state.lock() {
                    let offset = state
                        .selected()
                        .unwrap_or(0)
                        .saturating_add(1)
                        .min(self.files.len().saturating_sub(1));

                    state.select(Some(offset));
                }
            }
            KeyCode::Enter => {
                let torrent_file = self.selected();
                let _ = self.command_sender.send(TorrentInfoCommand::ShowPriority(
                    torrent_file.index,
                    torrent_file.priority,
                ));
            }
            _ => {}
        }
    }

    fn on_piece_completed(&mut self, piece: &PieceIndex) {
        for file in &mut self.files {
            if file.pieces.contains(piece) {
                file.completed_pieces += 1;
                file.completed_percentage =
                    ((file.completed_pieces as f32) / (file.total_pieces as f32)) * 100f32;
            }
        }
    }

    fn on_priorities_changed(&mut self, priorities: &[(FileIndex, FilePriority)]) {
        for priority in priorities {
            if let Some(mut file) = self.files.iter_mut().find(|e| e.index == priority.0) {
                file.priority = priority.1;
            }
        }
    }

    fn on_files_changed(&mut self, files: Vec<File>) {
        self.files = files
            .into_iter()
            .map(|file| TorrentFileData {
                index: file.index,
                name: file.filename(),
                size: file.len(),
                priority: file.priority,
                pieces: file.pieces.clone(),
                completed_percentage: 0.0,
                completed_pieces: 0,
                total_pieces: file.pieces.len(),
            })
            .collect()
    }
}

impl Widget for &TorrentFilesWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header = vec!["Name", "Priority", "Size", "Progress", "Pieces"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::new().bg(Color::DarkGray).fg(Color::White));
        let rows = self
            .files
            .iter()
            .enumerate()
            .map(|(index, file)| {
                let color = if index % 2 == 0 {
                    Color::Rgb(80, 80, 50)
                } else {
                    Color::Rgb(80, 80, 80)
                };

                Row::new(vec![
                    file.name.clone(),
                    priority_text(&file.priority).to_string(),
                    format_bytes(file.size),
                    format!("{:0.2}%", file.completed_percentage),
                    format!("{}/{}", file.completed_pieces, file.total_pieces),
                ])
                .style(Style::new().bg(color))
            })
            .collect::<Vec<Row>>();

        let table = Table::new(
            rows,
            [Fill(1), Length(12), Length(16), Length(20), Length(16)],
        )
        .header(header)
        .block(Block::bordered().title("Files"))
        .row_highlight_style(Style::new().bg(Color::Yellow).fg(Color::DarkGray))
        .highlight_spacing(HighlightSpacing::Always);

        if let Ok(mut state) = self.state.lock() {
            StatefulWidget::render(table, area, buf, &mut state);
        }
    }
}

#[derive(Debug)]
struct TorrentFilePriorityWidget {
    file: FileIndex,
    priorities: Vec<FilePriority>,
    state: Mutex<ListState>,
    command_sender: UnboundedSender<TorrentInfoCommand>,
}

impl TorrentFilePriorityWidget {
    fn new(command_sender: UnboundedSender<TorrentInfoCommand>) -> Self {
        Self {
            file: FileIndex::default(),
            priorities: FilePriority::iter().collect(),
            state: Default::default(),
            command_sender,
        }
    }

    fn set_file(&mut self, file: FileIndex) {
        self.file = file;
    }

    fn select(&mut self, priority: FilePriority) {
        if let Ok(mut state) = self.state.lock() {
            state.select(Some(
                self.priorities
                    .iter()
                    .position(|e| *e == priority)
                    .unwrap_or(0),
            ));
        }
    }

    fn selected(&self) -> FilePriority {
        let offset = self
            .state
            .lock()
            .ok()
            .and_then(|e| e.selected())
            .unwrap_or_default();
        self.priorities[offset]
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        match key.code() {
            KeyCode::Esc | KeyCode::Backspace => {
                let _ = self.command_sender.send(TorrentInfoCommand::ShowFiles);
            }
            KeyCode::Enter => {
                let _ = self.command_sender.send(TorrentInfoCommand::UpdatePriority(
                    self.file,
                    self.selected(),
                ));
                let _ = self.command_sender.send(TorrentInfoCommand::ShowFiles);
            }
            KeyCode::Up => {
                if let Ok(mut state) = self.state.lock() {
                    let offset = state.selected().unwrap_or(0).saturating_sub(1);
                    state.select(Some(offset));
                }
            }
            KeyCode::Down => {
                if let Ok(mut state) = self.state.lock() {
                    let selected = state.selected().unwrap_or(0).saturating_add(1);
                    if selected <= self.priorities.len() - 1 {
                        state.select(Some(selected));
                    }
                }
            }
            _ => {}
        }
    }
}

impl Widget for &TorrentFilePriorityWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let items = self
            .priorities
            .iter()
            .map(|e| priority_text(e))
            .collect::<Vec<_>>();
        let menu_list = List::new(items)
            .block(Block::new().title("File priority").borders(Borders::ALL))
            .highlight_style(Style::new().bg(Color::DarkGray));

        let mut state = self.state.lock().expect("Mutex poisoned");
        StatefulWidget::render(menu_list, area, buf, &mut state);
    }
}

#[derive(Debug)]
struct TorrentFileData {
    index: FileIndex,
    name: String,
    size: usize,
    priority: FilePriority,
    pieces: Range<PieceIndex>,
    completed_percentage: f32,
    completed_pieces: usize,
    total_pieces: usize,
}

fn priority_text(priority: &FilePriority) -> &'static str {
    match *priority {
        FilePriority::None => "None",
        FilePriority::Normal => "Normal",
        FilePriority::High => "High",
        FilePriority::Readahead => "Readahead",
        FilePriority::Next => "Next",
        FilePriority::Now => "Now",
    }
}

#[derive(Debug)]
struct TorrentPeersWidget {
    peers: HashMap<PeerHandle, TorrentPeerData>,
}

impl TorrentPeersWidget {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    async fn add_peer(&mut self, peer: TorrentPeer) {
        let events_receiver = peer.subscribe();
        let state = peer.state().await;
        let is_seed = peer.is_seed().await;
        let metrics = peer.metrics();

        self.peers.insert(
            peer.handle(),
            TorrentPeerData {
                client: peer.client(),
                available_pieces: metrics.available_pieces.get(),
                client_interested: metrics.client_interested.get(),
                remote_interested: metrics.remote_interested.get(),
                client_choked: metrics.client_choked.get(),
                remote_choked: metrics.remote_choked.get(),
                bytes_in: metrics.bytes_in.rate(),
                bytes_in_total: metrics.bytes_in.total(),
                bytes_out: metrics.bytes_out.rate(),
                bytes_out_total: metrics.bytes_out.total(),
                peer,
                state,
                is_seed,
                events_receiver,
                closed_since: None,
            },
        );
    }

    fn remove_peer(&mut self, handle: &PeerHandle) {
        if let Some(peer) = self.peers.get_mut(handle) {
            peer.closed_since = Some(Instant::now());
        }
    }

    async fn handle_peer_events(&mut self) {
        for (_, peer_data) in &mut self.peers {
            while let Ok(event) = peer_data.events_receiver.try_recv() {
                match &*event {
                    PeerEvent::StateChanged(state) => {
                        peer_data.state = *state;
                    }
                    PeerEvent::RemoteAvailablePieces(_) => {
                        peer_data.is_seed = peer_data.peer.is_seed().await;
                    }
                    PeerEvent::Stats(metrics) => {
                        peer_data.available_pieces = metrics.available_pieces.get();
                        peer_data.client_interested = metrics.client_interested.get();
                        peer_data.remote_interested = metrics.remote_interested.get();
                        peer_data.client_choked = metrics.client_choked.get();
                        peer_data.remote_choked = metrics.remote_choked.get();
                        peer_data.bytes_in = metrics.bytes_in.rate();
                        peer_data.bytes_in_total = metrics.bytes_in.total();
                        peer_data.bytes_out = metrics.bytes_out.rate();
                        peer_data.bytes_out_total = metrics.bytes_out.total();
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_closed_peers(&mut self) {
        self.peers.retain(|_, peer_data| {
            peer_data
                .closed_since
                .as_ref()
                .unwrap_or(&Instant::now())
                .elapsed()
                <= REMOVE_CLOSED_PEER_AFTER
        });
    }

    async fn tick(&mut self) {
        self.handle_peer_events().await;
        self.handle_closed_peers();
    }
}

impl Widget for &TorrentPeersWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = self
            .peers
            .iter()
            .enumerate()
            .map(|(index, (_, peer))| {
                let color = if index % 2 == 0 {
                    Color::Rgb(80, 80, 50)
                } else {
                    Color::Rgb(80, 80, 80)
                };
                let seed_text = if peer.is_seed { " :: seed :: " } else { " :: " };
                let client_interest = if peer.client_interested { "I" } else { "" };
                let remote_interest = if peer.remote_interested { "i" } else { "" };
                let client_choked = if peer.client_choked { "C" } else { "" };
                let remote_choked = if peer.remote_choked { "c" } else { "" };

                ListItem::new(vec![
                    Line::from(vec![
                        print_string_len(peer.client.addr.to_string(), 21).into(),
                        " :: ".into(),
                        peer.client.connection_protocol.to_string().into(),
                        seed_text.into(),
                        peer_state_as_str(&peer.state).into(),
                    ])
                    .style(Style::new().bold()),
                    Line::from(vec![
                        format!(
                            "down: {}/s ({})",
                            format_bytes(peer.bytes_in as usize),
                            format_bytes(peer.bytes_in_total as usize)
                        )
                        .into(),
                        " - ".into(),
                        format!(
                            "up: {}/s ({})",
                            format_bytes(peer.bytes_out as usize),
                            format_bytes(peer.bytes_out_total as usize)
                        )
                        .into(),
                        " - ".into(),
                        format!("pieces: {}", peer.available_pieces).into(),
                        " - ".into(),
                        format!(
                            "{}{}{}{}",
                            client_interest, remote_interest, client_choked, remote_choked
                        )
                        .into(),
                    ]),
                ])
                .style(Style::new().bg(color))
            })
            .collect::<Vec<ListItem>>();

        let peers_list = List::new(items).block(Block::bordered().title("Peers"));

        Widget::render(peers_list, area, buf);
    }
}

#[derive(Debug)]
struct TorrentPeerData {
    peer: TorrentPeer,
    client: PeerClientInfo,
    state: PeerState,
    is_seed: bool,
    available_pieces: u64,
    client_interested: bool,
    remote_interested: bool,
    client_choked: bool,
    remote_choked: bool,
    bytes_in: u32,
    bytes_in_total: u64,
    bytes_out: u32,
    bytes_out_total: u64,
    events_receiver: Subscription<PeerEvent>,
    closed_since: Option<Instant>,
}

fn peer_state_as_str(state: &PeerState) -> &'static str {
    match state {
        PeerState::Handshake => "Handshake",
        PeerState::RetrievingMetadata => "Retrieving metadata",
        PeerState::Paused => "Paused",
        PeerState::Idle => "Idle",
        PeerState::Downloading => "Downloading",
        PeerState::Uploading => "Uploading",
        PeerState::Error => "Error",
        PeerState::Closed => "Closed",
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum TorrentInfoState {
    Files,
    Priority,
}

impl Default for TorrentInfoState {
    fn default() -> Self {
        Self::Files
    }
}

#[derive(Debug)]
enum TorrentInfoCommand {
    ShowFiles,
    ShowPriority(FileIndex, FilePriority),
    UpdatePriority(FileIndex, FilePriority),
    TogglePaused,
}
