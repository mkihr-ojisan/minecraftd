use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, bail};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use mcctl_protocol::{Aggregation, GetMetricsRequest, client::Client};
use minecraftd_manifest::ServerManifest;
use ratatui::{DefaultTerminal, prelude::*, widgets::*};
use tokio::runtime::Handle;
use tui_scrollview::{ScrollView, ScrollViewState};

use crate::cli::StatsArgs;

pub async fn stats(args: StatsArgs) -> anyhow::Result<()> {
    let client = Client::connect().await?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        ratatui::run(|terminal| App::new(client, server_dir).run(terminal))?;
        Ok(())
    })
    .await??;
    Ok(())
}

struct App {
    client: Client,
    server_dir: String,

    exit: bool,
    next_update: Instant,
    chart_scale: ChartScale,

    tps_data: Vec<(f64, f64)>,
    mspt_data: Vec<(f64, f64)>,
    allocated_memory_bytes_data: Vec<(f64, f64)>,
    used_memory_bytes_data: Vec<(f64, f64)>,
    cpu_usage_percent_data: Vec<(f64, f64)>,
    player_count_data: Vec<(f64, f64)>,
    entity_count_data: Vec<(f64, f64)>,
    loaded_chunk_count_data: Vec<(f64, f64)>,
    proxy_received_bytes_per_second_data: Vec<(f64, f64)>,
    proxy_sent_bytes_per_second_data: Vec<(f64, f64)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ChartScale {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

impl ChartScale {
    fn resolution(&self) -> i64 {
        match self {
            ChartScale::Minute => 1,
            ChartScale::Hour => 15,
            ChartScale::Day => 60 * 15,
            ChartScale::Week => 3600,
            ChartScale::Month => 3600 * 4,
        }
    }

    fn update_interval(&self) -> Duration {
        Duration::from_secs(self.resolution() as u64)
    }

    fn start_timestamp(&self, now: i64) -> i64 {
        self.end_timestamp(now)
            - match self {
                ChartScale::Minute => 60,
                ChartScale::Hour => 3600,
                ChartScale::Day => 3600 * 24,
                ChartScale::Week => 3600 * 24 * 7,
                ChartScale::Month => 3600 * 24 * 30,
            }
    }

    fn end_timestamp(&self, now: i64) -> i64 {
        match self.downsample_interval() {
            Some(interval) => (now + interval - 1) / interval * interval,
            None => now,
        }
    }

    fn aggregation(&self) -> Aggregation {
        match self {
            ChartScale::Minute => Aggregation::None,
            _ => Aggregation::Avg,
        }
    }

    fn downsample_interval(&self) -> Option<i64> {
        match self {
            ChartScale::Minute => None,
            _ => Some(self.resolution()),
        }
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        match self {
            ChartScale::Minute => [-60.0, -1.0],
            ChartScale::Hour => [-3600.0, -1.0],
            ChartScale::Day => [-3600.0 * 24.0, -1.0],
            ChartScale::Week => [-3600.0 * 24.0 * 7.0, -1.0],
            ChartScale::Month => [-3600.0 * 24.0 * 30.0, -1.0],
        }
    }

    fn x_axis_labels(&self) -> &'static [&'static str] {
        match self {
            ChartScale::Minute => &["-60s", "-30s", "Now"],
            ChartScale::Hour => &["-60m", "-30m", "Now"],
            ChartScale::Day => &["-24h", "-12h", "Now"],
            ChartScale::Week => &["-7d", "Now"],
            ChartScale::Month => &["-30d", "-15d", "Now"],
        }
    }

    fn next_scale(&self) -> Option<Self> {
        match self {
            ChartScale::Minute => Some(ChartScale::Hour),
            ChartScale::Hour => Some(ChartScale::Day),
            ChartScale::Day => Some(ChartScale::Week),
            ChartScale::Week => Some(ChartScale::Month),
            ChartScale::Month => None,
        }
    }

    fn prev_scale(&self) -> Option<Self> {
        match self {
            ChartScale::Minute => None,
            ChartScale::Hour => Some(ChartScale::Minute),
            ChartScale::Day => Some(ChartScale::Hour),
            ChartScale::Week => Some(ChartScale::Day),
            ChartScale::Month => Some(ChartScale::Week),
        }
    }
}

#[derive(Default)]
struct AppState {
    scroll_view_state: ScrollViewState,
}

impl App {
    fn new(client: Client, server_dir: String) -> Self {
        Self {
            client,
            server_dir,
            exit: false,
            next_update: Instant::now(),
            chart_scale: ChartScale::Minute,

            tps_data: Vec::new(),
            mspt_data: Vec::new(),
            allocated_memory_bytes_data: Vec::new(),
            used_memory_bytes_data: Vec::new(),
            cpu_usage_percent_data: Vec::new(),
            player_count_data: Vec::new(),
            entity_count_data: Vec::new(),
            loaded_chunk_count_data: Vec::new(),
            proxy_received_bytes_per_second_data: Vec::new(),
            proxy_sent_bytes_per_second_data: Vec::new(),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        let mut state = AppState::default();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame, &mut state))?;
            self.handle_events(&mut state)?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, state: &mut AppState) {
        frame.render_stateful_widget(self as &App, frame.area(), state);
    }

    fn handle_events(&mut self, state: &mut AppState) -> anyhow::Result<()> {
        let event_available =
            crossterm::event::poll(self.next_update.saturating_duration_since(Instant::now()))?;

        if event_available {
            #[allow(clippy::single_match)]
            match crossterm::event::read()? {
                Event::Key(key_event) => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        match key_event.code {
                            KeyCode::Char('c') => self.quit(),
                            _ => {}
                        }
                    } else {
                        match key_event.code {
                            KeyCode::Char('q') => self.quit(),
                            KeyCode::Char('+') => self.zoom_in()?,
                            KeyCode::Char('-') => self.zoom_out()?,
                            KeyCode::Up => self.scroll_up(state),
                            KeyCode::Down => self.scroll_down(state),
                            KeyCode::PageUp => self.page_up(state),
                            KeyCode::PageDown => self.page_down(state),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        } else {
            self.update()?;
        }

        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        Handle::current().block_on(async {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let start_timestamp = self.chart_scale.start_timestamp(now);
            let end_timestamp = self.chart_scale.end_timestamp(now);
            let aggregation = self.chart_scale.aggregation() as i32;
            let downsample_interval = self.chart_scale.downsample_interval();

            macro_rules! get_data {
                ($metric:expr) => {
                    self.client
                        .get_metrics(GetMetricsRequest {
                            server_dir: self.server_dir.clone(),
                            metric: $metric.to_string(),
                            start_timestamp,
                            end_timestamp,
                            aggregation,
                            downsample_interval,
                            limit: None,
                            offset: None,
                        })
                        .await?
                        .data_points
                        .into_iter()
                        .map(|point| ((point.timestamp - now) as f64, point.value))
                        .collect()
                };
            }

            self.tps_data = get_data!("tps");
            self.mspt_data = get_data!("mspt");
            self.allocated_memory_bytes_data = get_data!("allocated_memory_bytes");
            self.used_memory_bytes_data = get_data!("used_memory_bytes");
            self.cpu_usage_percent_data = get_data!("cpu_usage_percent");
            self.player_count_data = get_data!("player_count");
            self.entity_count_data = get_data!("entity_count");
            self.loaded_chunk_count_data = get_data!("loaded_chunk_count");
            self.proxy_received_bytes_per_second_data =
                get_data!("proxy_received_bytes_per_second");
            self.proxy_sent_bytes_per_second_data = get_data!("proxy_sent_bytes_per_second");

            anyhow::Result::<()>::Ok(())
        })?;

        while Instant::now() >= self.next_update {
            self.next_update += self.chart_scale.update_interval();
        }

        Ok(())
    }

    fn quit(&mut self) {
        self.exit = true;
    }

    fn scroll_up(&mut self, state: &mut AppState) {
        state.scroll_view_state.scroll_up();
    }

    fn scroll_down(&mut self, state: &mut AppState) {
        state.scroll_view_state.scroll_down();
    }

    fn page_up(&mut self, state: &mut AppState) {
        state.scroll_view_state.scroll_page_up();
    }

    fn page_down(&mut self, state: &mut AppState) {
        state.scroll_view_state.scroll_page_down();
    }

    fn zoom_in(&mut self) -> anyhow::Result<()> {
        if let Some(prev_scale) = self.chart_scale.prev_scale() {
            self.chart_scale = prev_scale;
            self.update_now()?;
        }
        Ok(())
    }

    fn zoom_out(&mut self) -> anyhow::Result<()> {
        if let Some(next_scale) = self.chart_scale.next_scale() {
            self.chart_scale = next_scale;
            self.update_now()?;
        }
        Ok(())
    }

    fn update_now(&mut self) -> anyhow::Result<()> {
        self.next_update = Instant::now();
        self.update()
    }
}

impl StatefulWidget for &App {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        const NUM_CHARTS: usize = 8;
        const CHART_HEIGHT: u16 = 11;

        let layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
        let [title_bar_area, chart_area] = layout.areas(area);

        let title_bar = Paragraph::new(Line::from(vec![
            format!(" Minecraft Server Stats - '{}'", self.server_dir).bold(),
            " (Press 'q' to quit, '+'/'-' to zoom, Up/Down/PageUp/PageDown to scroll)".into(),
        ]))
        .bg(Color::Blue);
        title_bar.render(title_bar_area, buf);

        let height = CHART_HEIGHT * NUM_CHARTS as u16;
        let mut scroll_view = ScrollView::new(Size {
            width: if area.height > height {
                area.width
            } else {
                area.width - 1
            },
            height,
        });

        let layout = Layout::vertical(std::iter::repeat_n(
            Constraint::Length(CHART_HEIGHT),
            NUM_CHARTS,
        ));
        let areas = layout.areas::<NUM_CHARTS>(scroll_view.area());

        scroll_view.render_widget(
            SingleChart {
                title: " TPS ",
                data: &self.tps_data,
                default_y_bounds: (15.0, 25.0),
                is_integer: false,
                unit: ("tick/s", "ticks/s"),
                chart_scale: self.chart_scale,
            },
            areas[0],
        );

        scroll_view.render_widget(
            SingleChart {
                title: " MSPT ",
                data: &self.mspt_data,
                default_y_bounds: (0.0, 60.0),
                is_integer: false,
                unit: ("ms/tick", "ms/tick"),
                chart_scale: self.chart_scale,
            },
            areas[1],
        );

        scroll_view.render_widget(
            DoubleChart {
                title: " Memory Usage (MiB) ",
                data1: &self.allocated_memory_bytes_data,
                data2: &self.used_memory_bytes_data,
                default_y_bounds: (0.0, 1073741824.0),
                label1: "Allocated",
                label2: "Used",
                unit: "MiB",
                scale: 1048576.0,
                chart_scale: self.chart_scale,
            },
            areas[2],
        );

        scroll_view.render_widget(
            SingleChart {
                title: " CPU Usage (%) ",
                data: &self.cpu_usage_percent_data,
                default_y_bounds: (0.0, 100.0),
                is_integer: false,
                unit: ("%", "%"),
                chart_scale: self.chart_scale,
            },
            areas[3],
        );

        scroll_view.render_widget(
            SingleChart {
                title: " Player Count ",
                data: &self.player_count_data,
                default_y_bounds: (0.0, 10.0),
                is_integer: true,
                unit: ("player", "players"),
                chart_scale: self.chart_scale,
            },
            areas[4],
        );

        scroll_view.render_widget(
            SingleChart {
                title: " Entity Count ",
                data: &self.entity_count_data,
                default_y_bounds: (0.0, 1000.0),
                is_integer: true,
                unit: ("entity", "entities"),
                chart_scale: self.chart_scale,
            },
            areas[5],
        );

        scroll_view.render_widget(
            SingleChart {
                title: " Loaded Chunks ",
                data: &self.loaded_chunk_count_data,
                default_y_bounds: (0.0, 1000.0),
                is_integer: true,
                unit: ("chunk", "chunks"),
                chart_scale: self.chart_scale,
            },
            areas[6],
        );

        scroll_view.render_widget(
            DoubleChart {
                title: " Network Usage (via Proxy) (KiB/s) ",
                data1: &self.proxy_received_bytes_per_second_data,
                data2: &self.proxy_sent_bytes_per_second_data,
                default_y_bounds: (0.0, 1024.0),
                label1: "Received",
                label2: "Sent",
                unit: "KiB/s",
                scale: 1024.0,
                chart_scale: self.chart_scale,
            },
            areas[7],
        );

        scroll_view.render(chart_area, buf, &mut state.scroll_view_state);
    }
}

struct SingleChart<'a> {
    title: &'a str,
    data: &'a [(f64, f64)],
    default_y_bounds: (f64, f64),
    is_integer: bool,
    unit: (&'a str, &'a str),
    chart_scale: ChartScale,
}
impl Widget for SingleChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title(self.title)
            .title_style(Style::default().magenta().bold());

        let y_min = self.default_y_bounds.0.min(
            self.data
                .iter()
                .map(|(_, v)| *v)
                .fold(f64::INFINITY, f64::min),
        );
        let y_max = self.default_y_bounds.1.max(
            self.data
                .iter()
                .map(|(_, v)| *v)
                .fold(f64::NEG_INFINITY, f64::max),
        );

        let chart = Chart::new(vec![
            Dataset::default()
                .name("Data")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Cyan))
                .graph_type(GraphType::Line)
                .data(self.data),
        ])
        .block(block)
        .x_axis(
            Axis::default()
                .bounds(self.chart_scale.x_axis_bounds())
                .labels(
                    self.chart_scale
                        .x_axis_labels()
                        .iter()
                        .map(|s| Span::from(*s)),
                ),
        )
        .y_axis(
            Axis::default().bounds([y_min, y_max]).labels(
                if self.is_integer {
                    [format!("{:.0}", y_min), format!("{:.0}", y_max)]
                } else {
                    [format!("{:.1}", y_min), format!("{:.1}", y_max)]
                }
                .into_iter()
                .map(Span::from),
            ),
        );

        chart.render(area, buf);

        if self.chart_scale == ChartScale::Minute
            && let Some((_, latest_value)) = self.data.last()
        {
            let value_str = if self.is_integer {
                format!(
                    "{:.0} {}",
                    latest_value,
                    if *latest_value == 1.0 {
                        self.unit.0
                    } else {
                        self.unit.1
                    }
                )
            } else {
                format!("{:.2} {}", latest_value, self.unit.1)
            };
            let value_span = Span::styled(value_str, Style::default().yellow().bold());
            let value_area = Rect {
                x: area.x + area.width.saturating_sub(value_span.width() as u16 + 2),
                y: area.y + 1,
                width: value_span.width() as u16,
                height: 1,
            };
            value_span.render(value_area, buf);
        }

        if self.data.is_empty() {
            let no_data_span = Span::styled("No data", Style::default().bold());
            let no_data_area = Rect {
                x: area.x + area.width / 2 - no_data_span.width() as u16 / 2,
                y: area.y + area.height / 2,
                width: no_data_span.width() as u16,
                height: 1,
            };
            no_data_span.render(no_data_area, buf);
        }
    }
}

struct DoubleChart<'a> {
    title: &'a str,
    data1: &'a [(f64, f64)],
    data2: &'a [(f64, f64)],
    default_y_bounds: (f64, f64),
    label1: &'a str,
    label2: &'a str,
    unit: &'a str,
    scale: f64,
    chart_scale: ChartScale,
}

impl Widget for DoubleChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title(self.title)
            .title_style(Style::default().magenta().bold());

        let y_min = self.default_y_bounds.0.min(
            self.data1
                .iter()
                .chain(self.data2.iter())
                .map(|(_, v)| *v)
                .fold(f64::INFINITY, f64::min),
        );
        let y_max = self.default_y_bounds.1.max(
            self.data1
                .iter()
                .chain(self.data2.iter())
                .map(|(_, v)| *v)
                .fold(f64::NEG_INFINITY, f64::max),
        );

        let chart = Chart::new(vec![
            Dataset::default()
                .name(self.label1)
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Cyan))
                .graph_type(GraphType::Line)
                .data(self.data1),
            Dataset::default()
                .name(self.label2)
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(self.data2),
        ])
        .block(block)
        .x_axis(
            Axis::default()
                .bounds(self.chart_scale.x_axis_bounds())
                .labels(
                    self.chart_scale
                        .x_axis_labels()
                        .iter()
                        .map(|s| Span::from(*s)),
                ),
        )
        .y_axis(
            Axis::default().bounds([y_min, y_max]).labels(
                [
                    format!("{:.1}", y_min / self.scale),
                    format!("{:.1}", y_max / self.scale),
                ]
                .into_iter()
                .map(Span::from),
            ),
        );

        chart.render(area, buf);

        if self.chart_scale == ChartScale::Minute
            && let Some((_, latest_allocated)) = self.data1.last()
            && let Some((_, latest_used)) = self.data2.last()
        {
            let value_str = format!(
                "{}: {:.1} {}, {}: {:.1} {}",
                self.label1,
                latest_allocated / self.scale,
                self.unit,
                self.label2,
                latest_used / self.scale,
                self.unit
            );
            let value_span = Span::styled(value_str, Style::default().yellow().bold());
            let value_area = Rect {
                x: area.x + area.width.saturating_sub(value_span.width() as u16 + 2),
                y: area.y + 1,
                width: value_span.width() as u16,
                height: 1,
            };
            value_span.render(value_area, buf);
        }

        if self.data1.is_empty() && self.data2.is_empty() {
            let no_data_span = Span::styled("No data", Style::default().bold());
            let no_data_area = Rect {
                x: area.x + area.width / 2 - no_data_span.width() as u16 / 2,
                y: area.y + area.height / 2,
                width: no_data_span.width() as u16,
                height: 1,
            };
            no_data_span.render(no_data_area, buf);
        }
    }
}
