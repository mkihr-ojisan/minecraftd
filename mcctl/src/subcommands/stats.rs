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
    chart_type: ChartType,

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
enum ChartType {
    Realtime,
    Minutely,
    Hourly,
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
            chart_type: ChartType::Realtime,

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
                            KeyCode::Char('r') => self.update_now()?,
                            KeyCode::Char('t') => self.switch_chart_type()?,
                            KeyCode::Up => self.scroll_up(state),
                            KeyCode::Down => self.scroll_down(state),
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

            let start_timestamp = match self.chart_type {
                ChartType::Realtime => now - 60,
                ChartType::Minutely => now - 60 * 60,
                ChartType::Hourly => now - 60 * 60 * 24,
            };
            let aggregation = match self.chart_type {
                ChartType::Realtime => Aggregation::None,
                ChartType::Minutely => Aggregation::Avg,
                ChartType::Hourly => Aggregation::Avg,
            };
            let downsample_interval = match self.chart_type {
                ChartType::Realtime => None,
                ChartType::Minutely => Some(60),
                ChartType::Hourly => Some(3600),
            };

            macro_rules! get_data {
                ($metric:expr) => {
                    self.client
                        .get_metrics(GetMetricsRequest {
                            server_dir: self.server_dir.clone(),
                            metric: $metric.to_string(),
                            start_timestamp,
                            end_timestamp: now + 1,
                            aggregation: aggregation as i32,
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

        let update_interval = match self.chart_type {
            ChartType::Realtime => Duration::from_secs(1),
            ChartType::Minutely => Duration::from_secs(60),
            ChartType::Hourly => Duration::from_secs(3600),
        };
        while Instant::now() >= self.next_update {
            self.next_update += update_interval;
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

    fn switch_chart_type(&mut self) -> anyhow::Result<()> {
        self.chart_type = match self.chart_type {
            ChartType::Realtime => ChartType::Minutely,
            ChartType::Minutely => ChartType::Hourly,
            ChartType::Hourly => ChartType::Realtime,
        };
        self.update_now()
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
                chart_type: self.chart_type,
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
                chart_type: self.chart_type,
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
                chart_type: self.chart_type,
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
                chart_type: self.chart_type,
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
                chart_type: self.chart_type,
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
                chart_type: self.chart_type,
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
                chart_type: self.chart_type,
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
                chart_type: self.chart_type,
            },
            areas[7],
        );

        scroll_view.render(area, buf, &mut state.scroll_view_state);
    }
}

struct SingleChart<'a> {
    title: &'a str,
    data: &'a [(f64, f64)],
    default_y_bounds: (f64, f64),
    is_integer: bool,
    unit: (&'a str, &'a str),
    chart_type: ChartType,
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
                .bounds(match self.chart_type {
                    ChartType::Realtime => [-60.0, 0.0],
                    ChartType::Minutely => [-3600.0, 0.0],
                    ChartType::Hourly => [-86400.0, 0.0],
                })
                .labels(match self.chart_type {
                    ChartType::Realtime => ["-60s", "-30s", "Now"],
                    ChartType::Minutely => ["-60m", "-30m", "Now"],
                    ChartType::Hourly => ["-24h", "-12h", "Now"],
                }),
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

        if self.chart_type == ChartType::Realtime
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
    chart_type: ChartType,
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
                .bounds(match self.chart_type {
                    ChartType::Realtime => [-60.0, 0.0],
                    ChartType::Minutely => [-3600.0, 0.0],
                    ChartType::Hourly => [-86400.0, 0.0],
                })
                .labels(match self.chart_type {
                    ChartType::Realtime => ["-60s", "-30s", "Now"],
                    ChartType::Minutely => ["-60m", "-30m", "Now"],
                    ChartType::Hourly => ["-24h", "-12h", "Now"],
                }),
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

        if self.chart_type == ChartType::Realtime
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
