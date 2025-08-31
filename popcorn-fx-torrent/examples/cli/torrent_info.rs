use crate::app::{App, FXKeyEvent, FXWidget};
use async_trait::async_trait;
use crossterm::event::KeyCode;
use fx_callback::{Callback, Subscription};
use fx_handle::Handle;
use popcorn_fx_torrent::torrent::{
    format_bytes, File, FilePriority, InfoHash, PieceIndex, Torrent, TorrentEvent, TorrentState,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Fill, Length, Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Alignment, Color, Line, Style};
use ratatui::widgets::{
    Block, Cell, Gauge, HighlightSpacing, Paragraph, Row, Sparkline, StatefulWidget, Table,
    TableState, Widget,
};
use ratatui::Frame;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug)]
pub struct TorrentInfoWidget {
    name: String,
    torrent: Torrent,
    files_widget: TorrentFilesWidget,
    event_receiver: Subscription<TorrentEvent>,
    data: TorrentData,
}

impl TorrentInfoWidget {
    pub fn new(name: &str, torrent: Torrent) -> Self {
        let event_receiver = torrent.subscribe();

        Self {
            name: name.to_string(),
            torrent,
            files_widget: TorrentFilesWidget::new(),
            event_receiver,
            data: Default::default(),
        }
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
                    data.size = info.len();
                }
            }
            TorrentEvent::PeerConnected(_) => {
                data.peers = self.torrent.active_peer_connections().await;
            }
            TorrentEvent::PeerDisconnected(_) => {
                data.peers = self.torrent.active_peer_connections().await;
            }
            TorrentEvent::TrackersChanged => {}
            TorrentEvent::PiecesChanged(total_pieces) => {
                data.total_pieces = *total_pieces;
            }
            TorrentEvent::PiecePrioritiesChanged => {}
            TorrentEvent::PieceCompleted(piece) => {
                data.completed_pieces = self.torrent.total_completed_pieces().await;
                self.files_widget.on_piece_completed(piece);
            }
            TorrentEvent::FilesChanged => {
                data.total_files = self.torrent.total_files().await.unwrap_or(0);
                self.files_widget
                    .on_files_changed(self.torrent.files().await);
            }
            TorrentEvent::OptionsChanged => {}
            TorrentEvent::Stats(stats) => {
                data.progress = stats.progress();
                data.completed_size = stats.total_completed_size;
                data.wasted = stats.total_wasted;
                data.down.push(stats.download_rate);
                data.up.push(stats.upload_rate);

                if data.down.len() > 100 {
                    data.down.remove(0);
                }
                if data.up.len() > 100 {
                    data.up.remove(0);
                }
            }
        }
    }
}

#[async_trait]
impl FXWidget for TorrentInfoWidget {
    fn handle(&self) -> Handle {
        self.torrent.handle()
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn tick(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(&event).await;
        }
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        self.files_widget.on_key_event(key);
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
        let main = Layout::vertical([Min(10), Length(4), Fill(1)]);
        let [header_area, progress_area, files_area] = main.areas(area);
        let header = Layout::horizontal([Percentage(50), Percentage(50)]);
        let [metadata_area, performance_area] = header.areas(header_area);
        let performance = Layout::vertical([Percentage(50), Percentage(50)]);
        let [down_performance, up_performance] = performance.areas(performance_area);

        let data = &self.data;

        // render the metadata
        Paragraph::new(vec![
            Line::from(format!("Name: {}", self.name)),
            Line::from(format!(
                "State: {}",
                App::print_optional_string(data.state.as_ref())
            )),
            Line::from(format!(
                "Path: {}",
                App::print_optional_string(self.data.path.as_ref().and_then(|e| e.to_str()))
            )),
            Line::from(format!(
                "Info hash: {}",
                App::print_optional_string(self.data.info_hash.as_ref())
            )),
            Line::from(format!(
                "Size: {}/{}",
                format_bytes(self.data.completed_size),
                format_bytes(self.data.size)
            )),
            Line::from(format!(
                "Pieces: {}/{}",
                self.data.completed_pieces, self.data.total_pieces
            )),
            Line::from(format!("Wasted: {}", format_bytes(self.data.wasted))),
            Line::from(format!("Files: {}", self.data.total_files)),
            Line::from(format!("Connected peers: {}", self.data.peers)),
        ])
        .block(Block::bordered().title(" Metadata "))
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

        // render the files
        self.files_widget.render(files_area, buf);
    }
}

#[derive(Debug)]
struct TorrentData {
    path: Option<PathBuf>,
    state: Option<TorrentState>,
    info_hash: Option<InfoHash>,
    size: usize,
    total_pieces: usize,
    completed_pieces: usize,
    completed_size: usize,
    total_files: usize,
    peers: usize,
    progress: f32,
    wasted: usize,
    down: Vec<u64>,
    up: Vec<u64>,
}

impl Default for TorrentData {
    fn default() -> Self {
        Self {
            path: None,
            state: None,
            info_hash: None,
            size: 0,
            total_pieces: 0,
            completed_pieces: 0,
            completed_size: 0,
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
}

impl TorrentFilesWidget {
    pub fn new() -> Self {
        Self {
            files: vec![],
            state: Mutex::new(TableState::new().with_selected(0)),
        }
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
                    let mut offset = state.selected().unwrap_or(0).saturating_add(1);
                    if offset > self.files.len() {
                        offset = self.files.len() - 1;
                    }

                    state.select(Some(offset));
                }
            }
            _ => {}
        }
    }

    fn on_piece_completed(&mut self, piece: &PieceIndex) {
        for file in &mut self.files {
            if file.pieces.contains(piece) {
                file.completed_pieces += 1;
            }
        }
    }

    fn on_files_changed(&mut self, files: Vec<File>) {
        for file in files {
            self.files.push(TorrentFileData {
                name: file.filename(),
                size: file.len(),
                priority: file.priority,
                pieces: file.pieces.clone(),
                completed_pieces: 0,
                total_pieces: file.pieces.len(),
            });
        }
    }
}

impl Widget for &TorrentFilesWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Ok(mut state) = self.state.lock() {
            let header = vec!["Name", "Priority", "Size", "Progress", "Pieces"]
                .into_iter()
                .map(Cell::from)
                .collect::<Row>()
                .style(Style::new().bg(Color::Yellow));
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
                        priority_text(file.priority).to_string(),
                        format_bytes(file.size),
                        format!("{}%", 0),
                        format!("{}/{}", 0, file.total_pieces),
                    ])
                    .style(Style::new().bg(color))
                })
                .collect::<Vec<Row>>();

            let table = Table::new(rows, [Fill(1), Min(12), Min(16), Min(20), Min(16)])
                .header(header)
                .block(Block::bordered().title("Files"))
                .row_highlight_style(Style::new().bg(Color::LightYellow))
                .highlight_spacing(HighlightSpacing::Always);

            StatefulWidget::render(table, area, buf, &mut state);
        }
    }
}

#[derive(Debug)]
struct TorrentFileData {
    name: String,
    size: usize,
    priority: FilePriority,
    pieces: Range<PieceIndex>,
    completed_pieces: usize,
    total_pieces: usize,
}

fn priority_text(priority: FilePriority) -> &'static str {
    match priority {
        FilePriority::None => "None",
        FilePriority::Normal => "Normal",
        FilePriority::High => "High",
        FilePriority::Readahead => "Readahead",
        FilePriority::Next => "Next",
        FilePriority::Now => "Now",
    }
}
