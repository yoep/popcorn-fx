use crate::app::{AppCommandSender, FXKeyEvent, FXWidget};
use async_trait::async_trait;
use fx_callback::{Callback, Subscription};
use fx_handle::Handle;
use popcorn_fx_torrent::torrent::dht::{DhtEvent, DhtTracker};
use popcorn_fx_torrent::torrent::format_bytes;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline, Widget};
use ratatui::Frame;

pub(crate) const DHT_INFO_WIDGET_NAME: &str = "DHT";
const PERFORMANCE_HISTORY: usize = 150;

#[derive(Debug)]
pub struct DhtInfoWidget {
    handle: Handle,
    data: DhtData,
    dht: DhtTracker,
    event_receiver: Subscription<DhtEvent>,
    app_sender: AppCommandSender,
}

impl DhtInfoWidget {
    pub fn new(dht: DhtTracker, app_sender: AppCommandSender) -> Self {
        let event_receiver = dht.subscribe();

        Self {
            handle: Default::default(),
            data: Default::default(),
            dht,
            event_receiver,
            app_sender,
        }
    }

    fn handle_event(&mut self, event: &DhtEvent) {
        match event {
            DhtEvent::NodeAdded(_) => {}
            DhtEvent::Stats(metrics) => {
                self.data.total_nodes = metrics.total_nodes.get();
                self.data.total_router_nodes = metrics.total_router_nodes.get();
                self.data.total_pending_queries = metrics.total_pending_queries.get();

                self.data.bytes_down.push(metrics.total_bytes_in.get());
                if self.data.bytes_down.len() >= PERFORMANCE_HISTORY {
                    let _ = self.data.bytes_down.remove(0);
                }

                self.data.bytes_up.push(metrics.total_bytes_out.get());
                if self.data.bytes_up.len() >= PERFORMANCE_HISTORY {
                    let _ = self.data.bytes_up.remove(0);
                }
            }
        }
    }
}

#[async_trait]
impl FXWidget for DhtInfoWidget {
    fn handle(&self) -> Handle {
        self.handle
    }

    fn name(&self) -> &str {
        DHT_INFO_WIDGET_NAME
    }

    async fn tick(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(&*event);
        }
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        match key.code() {
            _ => {}
        }
    }

    fn on_paste_event(&mut self, text: String) {
        // no-op
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let header = Layout::horizontal([Percentage(50), Percentage(50)]);
        let [data_area, performance_area] = header.areas(area);
        let performance_layout = Layout::vertical([Percentage(50), Percentage(50)]);
        let [down_performance, up_performance] = performance_layout.areas(performance_area);

        // render the DHT network data
        Paragraph::new(vec![
            Line::from(vec![
                Span::from("Port: ").bold(),
                self.dht.port().to_string().into(),
            ]),
            Line::from(vec![
                Span::from("Nodes: ").bold(),
                self.data.total_nodes.to_string().into(),
            ]),
            Line::from(vec![
                Span::from("Router nodes: ").bold(),
                self.data.total_router_nodes.to_string().into(),
            ]),
            Line::from(vec![
                Span::from("Pending queries: ").bold(),
                self.data.total_pending_queries.to_string().into(),
            ]),
        ])
        .block(Block::new().title("DHT network").borders(Borders::ALL))
        .render(data_area, frame.buffer_mut());

        // render the performance
        Sparkline::default()
            .block(Block::bordered().title(format!(
                "Down: {}/s",
                format_bytes(self.data.bytes_down.last().map(|e| *e as usize).unwrap_or(0))
            )))
            .data(&self.data.bytes_down)
            .style(Style::default().fg(Color::Yellow))
            .render(down_performance, frame.buffer_mut());
        Sparkline::default()
            .block(Block::bordered().title(format!(
                "Up: {}/s",
                format_bytes(self.data.bytes_up.last().map(|e| *e as usize).unwrap_or(0))
            )))
            .data(&self.data.bytes_up)
            .style(Style::default().fg(Color::Yellow))
            .render(up_performance, frame.buffer_mut());
    }
}

#[derive(Debug, Default)]
struct DhtData {
    total_nodes: u64,
    total_router_nodes: u64,
    total_pending_queries: u64,
    bytes_down: Vec<u64>,
    bytes_up: Vec<u64>,
}
