use crate::app::{AppCommandSender, FXKeyEvent, FXWidget};
use async_trait::async_trait;
use crossterm::event::KeyCode;
use fx_callback::{Callback, Subscription};
use fx_handle::Handle;
use popcorn_fx_torrent::torrent::dht::{DhtEvent, DhtTracker, Node, NodeState};
use popcorn_fx_torrent::torrent::format_bytes;
use ratatui::layout::Constraint::{Fill, Length, Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Color, StatefulWidget, Style};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Sparkline, Table, TableState, Widget,
};
use ratatui::Frame;
use std::sync::Mutex;

pub(crate) const DHT_INFO_WIDGET_NAME: &str = "DHT";
const PERFORMANCE_HISTORY: usize = 150;
const CHECKMARK_CHAR: &str = "\u{2713}";

#[derive(Debug)]
pub struct DhtInfoWidget {
    handle: Handle,
    data: DhtData,
    dht: DhtTracker,
    node_info_widget: DhtNodeInfoWidget,
    event_receiver: Subscription<DhtEvent>,
    app_sender: AppCommandSender,
}

impl DhtInfoWidget {
    pub fn new(dht: DhtTracker, nodes: Vec<Node>, app_sender: AppCommandSender) -> Self {
        let event_receiver = dht.subscribe();
        let node_info_widget = DhtNodeInfoWidget::new(nodes);

        Self {
            handle: Default::default(),
            data: Default::default(),
            dht,
            node_info_widget,
            event_receiver,
            app_sender,
        }
    }

    fn handle_event(&mut self, event: &DhtEvent) {
        match event {
            DhtEvent::NodeAdded(node) => {
                self.node_info_widget.add_node(node.clone());
            }
            DhtEvent::Stats(metrics) => {
                self.data.total_nodes = metrics.nodes.get();
                self.data.total_router_nodes = metrics.router_nodes.get();
                self.data.pending_queries = metrics.pending_queries.get();
                self.data.errors = metrics.errors.total();
                self.data.discovered_peers = metrics.discovered_peers.total();

                self.data.bytes_down.push(metrics.bytes_in.get());
                if self.data.bytes_down.len() >= PERFORMANCE_HISTORY {
                    let _ = self.data.bytes_down.remove(0);
                }

                self.data.bytes_up.push(metrics.bytes_out.get());
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
        self.node_info_widget.on_key_event(key);
    }

    fn on_paste_event(&mut self, _: String) {
        // no-op
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let main = Layout::vertical([Length(10), Fill(1)]);
        let [header_area, details_area] = main.areas(area);
        let header = Layout::horizontal([Percentage(50), Percentage(50)]);
        let [data_area, performance_area] = header.areas(header_area);
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
                self.data.pending_queries.to_string().into(),
            ]),
            Line::from(vec![
                Span::from("Discovered peers: ").bold(),
                self.data.discovered_peers.to_string().into(),
            ]),
            Line::from(vec![
                Span::from("Errors: ").bold(),
                self.data.errors.to_string().into(),
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

        // render the node information
        self.node_info_widget.render(frame, details_area);
    }
}

#[derive(Debug, Default)]
struct DhtData {
    total_nodes: u64,
    total_router_nodes: u64,
    pending_queries: u64,
    errors: u64,
    discovered_peers: u64,
    bytes_down: Vec<u64>,
    bytes_up: Vec<u64>,
}

#[derive(Debug)]
struct DhtNodeInfoWidget {
    nodes: Vec<Node>,
    state: Mutex<TableState>,
}

impl DhtNodeInfoWidget {
    fn new(nodes: Vec<Node>) -> Self {
        Self {
            nodes,
            state: Default::default(),
        }
    }

    fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
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
                        .max(self.nodes.len().saturating_sub(1));

                    state.select(Some(offset));
                }
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let header = vec!["Address", "State", "Secure"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::new().bg(Color::Yellow));
        let rows = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let color = if index % 2 == 0 {
                    Color::Rgb(80, 80, 50)
                } else {
                    Color::Rgb(80, 80, 80)
                };
                let secure = if node.is_secure() { CHECKMARK_CHAR } else { "" };

                Row::new(vec![
                    node.addr.to_string(),
                    node_state_as_str(&node.state).to_string(),
                    secure.to_string(),
                ])
                .style(Style::new().bg(color))
            })
            .collect::<Vec<Row>>();

        let table = Table::new(rows, [Fill(1), Min(14), Min(6)])
            .header(header)
            .block(Block::bordered().title("Nodes"))
            .row_highlight_style(Style::new().bg(Color::LightYellow))
            .highlight_spacing(HighlightSpacing::Always);

        if let Ok(mut state) = self.state.lock() {
            StatefulWidget::render(table, area, frame.buffer_mut(), &mut state);
        }
    }
}

fn node_state_as_str(state: &NodeState) -> &str {
    match state {
        NodeState::Good => "Good",
        NodeState::Questionable => "Questionable",
        NodeState::Bad => "Bad",
    }
}
