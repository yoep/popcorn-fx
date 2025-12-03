use crate::app::{FXKeyEvent, FXWidget, PERFORMANCE_HISTORY};
use crate::widget::InputWidget;
use async_trait::async_trait;
use crossterm::event::KeyCode;
use fx_callback::{Callback, Subscription};
use popcorn_fx_torrent::torrent::dht::{DhtEvent, DhtTracker, Node, NodeState};
use popcorn_fx_torrent::torrent::format_bytes;
use ratatui::layout::Constraint::{Fill, Length, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Color, StatefulWidget, Style, Text};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Sparkline, Table, TableState, Widget,
};
use ratatui::Frame;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub(crate) const DHT_INFO_WIDGET_NAME: &str = "DHT";
const CHECKMARK_CHAR: &str = "\u{2713}";

#[derive(Debug)]
pub struct DhtInfoWidget {
    data: DhtData,
    dht: DhtTracker,
    state: DhtInfoState,
    node_info_widget: DhtNodeInfoWidget,
    add_node_widget: DhtAddNodeWidget,
    event_receiver: Subscription<DhtEvent>,
    command_receiver: UnboundedReceiver<DhtInfoCommand>,
}

impl DhtInfoWidget {
    pub fn new(dht: DhtTracker, nodes: Vec<Node>) -> Self {
        let (command_sender, command_receiver) = unbounded_channel();
        let event_receiver = dht.subscribe();

        Self {
            data: Default::default(),
            dht,
            state: DhtInfoState::Nodes,
            node_info_widget: DhtNodeInfoWidget::new(nodes),
            add_node_widget: DhtAddNodeWidget::new(command_sender),
            event_receiver,
            command_receiver,
        }
    }

    async fn handle_event(&mut self, event: &DhtEvent) {
        match event {
            DhtEvent::IDChanged => {}
            DhtEvent::ExternalIpChanged(ip) => {
                self.data.ip = Some(*ip);
            }
            DhtEvent::NodeAdded(node) => {
                self.node_info_widget.add_node(node.clone());
            }
            DhtEvent::Stats(metrics) => {
                self.data.total_nodes = metrics.nodes.get();
                self.data.pending_queries = metrics.pending_queries.get();
                self.data.errors = metrics.errors.total();
                self.data.discovered_peers = metrics.discovered_peers.total();

                self.data.bytes_in.push(metrics.bytes_in.get());
                if self.data.bytes_in.len() >= PERFORMANCE_HISTORY {
                    let _ = self.data.bytes_in.remove(0);
                }

                self.data.bytes_out.push(metrics.bytes_out.get());
                if self.data.bytes_out.len() >= PERFORMANCE_HISTORY {
                    let _ = self.data.bytes_out.remove(0);
                }
            }
        }
    }

    async fn handle_command_event(&mut self, command: DhtInfoCommand) {
        match command {
            DhtInfoCommand::ShowNodes => {
                self.state = DhtInfoState::Nodes;
            }
            DhtInfoCommand::AddNode(addr) => {
                let dht = self.dht.clone();
                tokio::spawn(async move {
                    let _ = dht.add_node(addr).await;
                });
            }
        }
    }
}

#[async_trait]
impl FXWidget for DhtInfoWidget {
    fn name(&self) -> &str {
        DHT_INFO_WIDGET_NAME
    }

    async fn tick(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(&*event).await;
        }
        while let Ok(command) = self.command_receiver.try_recv() {
            self.handle_command_event(command).await;
        }

        self.node_info_widget.tick().await;
    }

    fn on_key_event(&mut self, mut event: FXKeyEvent) {
        let state = &self.state;
        if state != &DhtInfoState::AddNode {
            match event.key_code() {
                KeyCode::Char('a') => {
                    event.consume();
                    self.state = DhtInfoState::AddNode;
                    return;
                }
                _ => {}
            }
        }

        match self.state {
            DhtInfoState::Nodes => self.node_info_widget.on_key_event(event),
            DhtInfoState::AddNode => self.add_node_widget.on_key_event(event),
        }
    }

    fn on_paste_event(&mut self, text: String) {
        if &DhtInfoState::AddNode == &self.state {
            self.add_node_widget.on_paste_event(text);
        }
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
                Span::from("IP: ").bold(),
                self.data
                    .ip
                    .map(|e| e.to_string())
                    .unwrap_or("--.--.--.--".to_string())
                    .into(),
            ]),
            Line::from(vec![
                Span::from("Port: ").bold(),
                self.dht.port().to_string().into(),
            ]),
            Line::from(vec![
                Span::from("Nodes: ").bold(),
                self.data.total_nodes.to_string().into(),
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
        .block(
            Block::new()
                .title("DHT network")
                .title_bottom(" Press A to add DHT node ")
                .borders(Borders::ALL),
        )
        .render(data_area, frame.buffer_mut());

        // render the performance
        Sparkline::default()
            .block(Block::bordered().title(format!(
                "Down: {}/s",
                format_bytes(self.data.bytes_in.last().map(|e| *e as usize).unwrap_or(0))
            )))
            .data(&self.data.bytes_in)
            .style(Style::default().fg(Color::Yellow))
            .render(down_performance, frame.buffer_mut());
        Sparkline::default()
            .block(Block::bordered().title(format!(
                "Up: {}/s",
                format_bytes(self.data.bytes_out.last().map(|e| *e as usize).unwrap_or(0))
            )))
            .data(&self.data.bytes_out)
            .style(Style::default().fg(Color::Yellow))
            .render(up_performance, frame.buffer_mut());

        // render the node information
        match &self.state {
            DhtInfoState::Nodes => self.node_info_widget.render(frame, details_area),
            DhtInfoState::AddNode => self.add_node_widget.render(frame, details_area),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum DhtInfoState {
    Nodes,
    AddNode,
}

#[derive(Debug)]
enum DhtInfoCommand {
    ShowNodes,
    AddNode(SocketAddr),
}

#[derive(Debug, Default)]
struct DhtData {
    ip: Option<IpAddr>,
    total_nodes: u64,
    pending_queries: u64,
    errors: u64,
    discovered_peers: u64,
    bytes_in: Vec<u64>,
    bytes_out: Vec<u64>,
}

#[derive(Debug)]
struct DhtNodeInfoWidget {
    nodes: Vec<NodeData>,
    state: Mutex<TableState>,
    last_updated: Instant,
}

impl DhtNodeInfoWidget {
    fn new(nodes: Vec<Node>) -> Self {
        Self {
            nodes: nodes.into_iter().map(NodeData::new).collect(),
            state: Default::default(),
            last_updated: Instant::now(),
        }
    }

    fn add_node(&mut self, node: Node) {
        self.nodes.push(NodeData::new(node));
    }

    async fn tick(&mut self) {
        let elapsed = self.last_updated.elapsed();
        if elapsed < Duration::from_secs(3) {
            return;
        }

        for data in &mut self.nodes {
            let state = data.node.state().await;
            let last_seen = data.node.last_seen().await;
            (*data).state = state;
            (*data).last_seen = last_seen;
        }

        self.last_updated = Instant::now();
    }

    fn on_key_event(&mut self, mut event: FXKeyEvent) {
        match event.key_code() {
            KeyCode::Up => {
                event.consume();
                if let Ok(mut state) = self.state.lock() {
                    let offset = state.selected().unwrap_or(0).saturating_sub(1);
                    state.select(Some(offset));
                }
            }
            KeyCode::Down => {
                event.consume();
                if let Ok(mut state) = self.state.lock() {
                    let offset = state
                        .selected()
                        .unwrap_or(0)
                        .saturating_add(1)
                        .min(self.nodes.len().saturating_sub(1));

                    state.select(Some(offset));
                }
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::horizontal([Percentage(70), Percentage(30)]);
        let [nodes_area, _] = layout.areas(area);

        self.render_nodes(frame, nodes_area);
    }

    fn render_nodes(&self, frame: &mut Frame, area: Rect) {
        let header = vec![
            "Address",
            "State",
            "Last seen",
            "Secure",
            "Queried",
            "Errors",
        ]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(Self::header_style());
        let rows = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, data)| {
                let color = Self::row_color(index);
                let node = &data.node;
                let state = &data.state;
                let last_seen = data.last_seen.elapsed();
                let secure = if node.is_secure() { CHECKMARK_CHAR } else { "" };

                Row::new(vec![
                    node.addr().to_string(),
                    node_state_as_str(state).to_string(),
                    format!("{}s", last_seen.as_secs()),
                    secure.to_string(),
                    node.metrics().confirmed_queries.total().to_string(),
                    node.metrics().errors.total().to_string(),
                ])
                .style(Style::new().bg(color))
            })
            .collect::<Vec<Row>>();

        let table = Table::new(
            rows,
            [
                Fill(1),
                Length(14),
                Length(10),
                Length(8),
                Length(10),
                Length(10),
            ],
        )
        .header(header)
        .block(Block::bordered().title("Nodes"))
        .row_highlight_style(Style::new().bg(Color::LightYellow).fg(Color::DarkGray))
        .highlight_spacing(HighlightSpacing::Always);

        if let Ok(mut state) = self.state.lock() {
            StatefulWidget::render(table, area, frame.buffer_mut(), &mut state);
        }
    }

    fn header_style() -> Style {
        Style::new().bg(Color::DarkGray).fg(Color::White)
    }

    fn row_color(index: usize) -> Color {
        let color = if index % 2 == 0 {
            Color::Rgb(80, 80, 50)
        } else {
            Color::Rgb(80, 80, 80)
        };
        color
    }
}

#[derive(Debug)]
struct NodeData {
    node: Node,
    state: NodeState,
    last_seen: Instant,
}

impl NodeData {
    fn new(node: Node) -> Self {
        Self {
            node,
            state: NodeState::Good,
            last_seen: Instant::now(),
        }
    }
}

#[derive(Debug)]
struct DhtAddNodeWidget {
    input: InputWidget,
    error: Option<String>,
    command_sender: UnboundedSender<DhtInfoCommand>,
}

impl DhtAddNodeWidget {
    fn new(command_sender: UnboundedSender<DhtInfoCommand>) -> Self {
        Self {
            input: InputWidget::new_with_opts("", true),
            error: None,
            command_sender,
        }
    }

    fn reset(&mut self) {
        self.input.reset();
        self.error = None;
    }

    fn try_parse_addr(&mut self) -> Option<SocketAddr> {
        let addr_value = self.input.as_str();

        match SocketAddr::from_str(addr_value) {
            Ok(addr) => Some(addr),
            Err(e) => {
                self.error = Some(format!("Node address is invalid, {}", e));
                None
            }
        }
    }

    fn on_key_event(&mut self, mut event: FXKeyEvent) {
        match event.key_code() {
            KeyCode::Esc => {
                event.consume();
                self.reset();
                let _ = self.command_sender.send(DhtInfoCommand::ShowNodes);
            }
            KeyCode::Backspace => {
                event.consume();
                self.input.backspace();
            }
            KeyCode::Enter => {
                event.consume();
                if let Some(addr) = self.try_parse_addr() {
                    self.reset();
                    let _ = self.command_sender.send(DhtInfoCommand::AddNode(addr));
                    let _ = self.command_sender.send(DhtInfoCommand::ShowNodes);
                }
            }
            KeyCode::Char(ch) => {
                event.consume();
                self.input.insert(ch);
            }
            KeyCode::Left => {
                event.consume();
                self.input.cursor_left();
            }
            KeyCode::Right => {
                event.consume();
                self.input.cursor_right();
            }
            _ => {}
        }
    }

    fn on_paste_event(&mut self, text: String) {
        self.input.append(text.as_str());
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Fill(1), Length(1), Length(1)]);
        let [input_area, help_area, invalid_area] = layout.areas(area);

        // render the input widget
        let block = Block::new()
            .title("Enter DHT node address")
            .borders(Borders::ALL);
        self.input.render(frame, block.inner(input_area));
        frame.render_widget(block, input_area);

        // render the help info
        Text::from("Press Esc to return, Enter to add node")
            .style(Style::new().italic())
            .render(help_area, frame.buffer_mut());

        // render the error message
        if let Some(error) = &self.error {
            Text::from(error.as_str())
                .style(Style::new().fg(Color::Red))
                .render(invalid_area, frame.buffer_mut());
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
