use binrw::io::BufReader;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use sdw::model::SonarDataRecord;
use sdw::parser::jsf;
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App<T> {
    items: StatefulList<SonarDataRecord<T>>,
}

impl<T> App<T> {
    fn build(v: Vec<SonarDataRecord<T>>) -> App<T> {
        App {
            items: StatefulList::with_items(v),
        }
    }
}

fn main() -> std::io::Result<()> {
    // Load data
    let f = std::fs::File::open("assets/HE501_Hydro3_025.001.jsf")?;
    let reader = BufReader::new(f);
    let jsf = jsf::File::new(reader);
    let v: Vec<SonarDataRecord<f32>> = jsf.map(|msg| SonarDataRecord::from(msg.unwrap())).collect();

    let app = App::build(v);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<T, B: Backend>(terminal: &mut Terminal<B>, mut app: App<T>) -> std::io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Left => app.items.unselect(),
                KeyCode::Down => app.items.next(),
                KeyCode::Up => app.items.previous(),
                _ => {}
            }
        }
    }
}

fn ui<T, B: Backend>(f: &mut Frame<B>, app: &mut App<T>) {
    let format = time::format_description::parse(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]",
    )
    .unwrap();

    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .enumerate()
        .map(|(i, rec)| {
            let datatype = match rec {
                SonarDataRecord::Ping(_) => "Ping".to_string(),
                SonarDataRecord::Course(_) => "Course".to_string(),
                SonarDataRecord::Orientation(_) => "Orientation".to_string(),
                SonarDataRecord::Position(_) => "Position".to_string(),
                SonarDataRecord::Unknown => "Unknown".to_string(),
            };

            let timestamp = match rec {
                SonarDataRecord::Ping(rec) => rec.timestamp.format(&format).unwrap(),
                SonarDataRecord::Course(rec) => rec.timestamp.format(&format).unwrap(),
                SonarDataRecord::Orientation(rec) => rec.timestamp.format(&format).unwrap(),
                SonarDataRecord::Position(rec) => rec.timestamp.format(&format).unwrap(),
                SonarDataRecord::Unknown => "".to_string(),
            };

            let channel = match rec {
                SonarDataRecord::Ping(rec) => format!("{:?}", rec.channel),
                SonarDataRecord::Course(_) => "".to_string(),
                SonarDataRecord::Orientation(_) => "".to_string(),
                SonarDataRecord::Position(_) => "".to_string(),
                SonarDataRecord::Unknown => "".to_string(),
            };

            let s = format!(
                "{0:5} {1:25} {2:15} {3:10}",
                i, timestamp, datatype, channel
            );

            ListItem::new(s)
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, f.size(), &mut app.items.state);
}
