use crate::app::{FXKeyEvent, FXWidget, PERFORMANCE_HISTORY};
use async_trait::async_trait;
use crossterm::event::KeyCode;
use fx_callback::{Callback, Subscription};
use popcorn_fx_torrent::torrent::format_bytes;
use popcorn_fx_torrent::torrent::tracker::{
    Tracker, TrackerClient, TrackerClientEvent, TrackerState,
};
use ratatui::layout::Constraint::{Fill, Length, Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Color, StatefulWidget, Style};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Cell, HighlightSpacing, Paragraph, Row, Sparkline, Table, TableState, Widget,
};
use ratatui::Frame;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub(crate) const TRACKER_INFO_WIDGET_NAME: &str = "Tracker";

#[derive(Debug)]
pub struct TrackersInfoWidget {
    tracker_manager: TrackerClient,
    data: TrackerManagerData,
    details_widget: TrackerDetailsWidget,
    event_receiver: Subscription<TrackerClientEvent>,
}

impl TrackersInfoWidget {
    pub fn new(tracker_manager: TrackerClient, trackers: Vec<Tracker>) -> Self {
        let event_receiver = tracker_manager.subscribe();

        Self {
            tracker_manager,
            data: TrackerManagerData::default(),
            details_widget: TrackerDetailsWidget::new(trackers),
            event_receiver,
        }
    }

    async fn handle_events(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            match &*event {
                TrackerClientEvent::TrackerAdded(handle) => {
                    if let Some(tracker) = self.tracker_manager.get(handle).await {
                        self.details_widget.add_tracker(tracker);
                    }
                }
                TrackerClientEvent::Stats(metrics) => {
                    self.data.total_trackers = self.tracker_manager.trackers_len().await;
                    self.data.tracked_torrents = self.tracker_manager.torrents_len().await;

                    self.data.bytes_in.push(metrics.bytes_in.get());
                    if self.data.bytes_in.len() >= PERFORMANCE_HISTORY {
                        let _ = self.data.bytes_in.remove(0);
                    }

                    self.data.bytes_out.push(metrics.bytes_out.get());
                    if self.data.bytes_out.len() >= PERFORMANCE_HISTORY {
                        let _ = self.data.bytes_out.remove(0);
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl FXWidget for TrackersInfoWidget {
    fn name(&self) -> &str {
        TRACKER_INFO_WIDGET_NAME
    }

    async fn tick(&mut self) {
        self.handle_events().await;
        self.details_widget.tick().await;
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        self.details_widget.on_key_event(key);
    }

    fn on_paste_event(&mut self, _: String) {
        // no-op
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let main = Layout::vertical([Length(10), Fill(1)]);
        let [header_area, details_area] = main.areas(area);
        let header = Layout::horizontal([Percentage(50), Percentage(50)]);
        let [metadata_area, performance_area] = header.areas(header_area);
        let performance = Layout::vertical([Percentage(50), Percentage(50)]);
        let [performance_in, performance_out] = performance.areas(performance_area);

        // render the metadata
        Paragraph::new(vec![
            Line::from(vec![
                Span::from("Trackers: ").bold(),
                self.data.total_trackers.to_string().into(),
            ]),
            Line::from(vec![
                Span::from("Tracked torrents: ").bold(),
                self.data.tracked_torrents.to_string().into(),
            ]),
        ])
        .block(Block::bordered().title("Trackers"))
        .render(metadata_area, frame.buffer_mut());

        // render the performance
        Sparkline::default()
            .block(Block::bordered().title(format!(
                "Down: {}/s",
                format_bytes(self.data.bytes_in.last().map(|e| *e as usize).unwrap_or(0))
            )))
            .data(&self.data.bytes_in)
            .style(Style::default().fg(Color::Yellow))
            .render(performance_in, frame.buffer_mut());
        Sparkline::default()
            .block(Block::bordered().title(format!(
                "Up: {}/s",
                format_bytes(self.data.bytes_out.last().map(|e| *e as usize).unwrap_or(0))
            )))
            .data(&self.data.bytes_out)
            .style(Style::default().fg(Color::Yellow))
            .render(performance_out, frame.buffer_mut());

        // render the details
        self.details_widget.render(frame, details_area);
    }
}

#[derive(Debug)]
struct TrackerDetailsWidget {
    trackers: Vec<(Tracker, TrackerData)>,
    state: Mutex<TableState>,
    last_updated: Instant,
}

impl TrackerDetailsWidget {
    fn new(trackers: Vec<Tracker>) -> Self {
        Self {
            trackers: trackers
                .into_iter()
                .map(|e| {
                    let url = e.url().to_string();
                    (e, TrackerData::new(url))
                })
                .collect(),
            state: Mutex::new(Default::default()),
            last_updated: Instant::now(),
        }
    }

    fn add_tracker(&mut self, tracker: Tracker) {
        let url = tracker.url().to_string();
        self.trackers.push((tracker, TrackerData::new(url)));
    }

    async fn tick(&mut self) {
        if self.last_updated.elapsed() < Duration::from_secs(1) {
            return;
        }

        for (tracker, data) in self.trackers.iter_mut() {
            let metrics = tracker.metrics();

            data.state = tracker.state().await;
            data.confirmed = metrics.confirmed.total();
            data.errors = metrics.errors.total();
            data.bytes_in = metrics.bytes_in.total();
            data.bytes_out = metrics.bytes_out.total();
        }

        self.last_updated = Instant::now();
    }

    fn on_key_event(&mut self, mut key: FXKeyEvent) {
        match key.code() {
            KeyCode::Up => {
                key.consume();
                if let Ok(mut state) = self.state.lock() {
                    let offset = state.selected().unwrap_or(0).saturating_sub(1);
                    state.select(Some(offset));
                }
            }
            KeyCode::Down => {
                key.consume();
                if let Ok(mut state) = self.state.lock() {
                    let offset = state
                        .selected()
                        .unwrap_or(0)
                        .saturating_add(1)
                        .max(self.trackers.len().saturating_sub(1));

                    state.select(Some(offset));
                }
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let header = vec!["Url", "State", "Sent", "Received", "Confirmed", "Error"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::new().bg(Color::DarkGray).fg(Color::White));
        let rows = self
            .trackers
            .iter()
            .enumerate()
            .map(|(index, (_, data))| {
                let color = if index % 2 == 0 {
                    Color::Rgb(80, 80, 50)
                } else {
                    Color::Rgb(80, 80, 80)
                };
                let state = if data.state == TrackerState::Active {
                    "Active"
                } else {
                    "Disabled"
                };

                Row::new(vec![
                    data.url.clone(),
                    state.to_string(),
                    format_bytes(data.bytes_out as usize).to_string(),
                    format_bytes(data.bytes_in as usize).to_string(),
                    data.confirmed.to_string(),
                    data.errors.to_string(),
                ])
                .style(Style::new().bg(color))
            })
            .collect::<Vec<Row>>();

        let table = Table::new(
            rows,
            [
                Fill(1),
                Length(10),
                Length(10),
                Length(10),
                Length(12),
                Length(12),
            ],
        )
        .header(header)
        .block(Block::bordered().title("Trackers"))
        .row_highlight_style(Style::new().bg(Color::LightYellow).fg(Color::DarkGray))
        .highlight_spacing(HighlightSpacing::Always);

        if let Ok(mut state) = self.state.lock() {
            StatefulWidget::render(table, area, frame.buffer_mut(), &mut state);
        }
    }
}

#[derive(Debug, Default)]
struct TrackerManagerData {
    total_trackers: usize,
    tracked_torrents: usize,
    bytes_in: Vec<u64>,
    bytes_out: Vec<u64>,
}

#[derive(Debug)]
struct TrackerData {
    url: String,
    state: TrackerState,
    confirmed: u64,
    errors: u64,
    bytes_in: u64,
    bytes_out: u64,
}

impl TrackerData {
    fn new(url: String) -> Self {
        Self {
            url,
            state: TrackerState::Active,
            confirmed: 0,
            errors: 0,
            bytes_in: 0,
            bytes_out: 0,
        }
    }
}
