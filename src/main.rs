use std::fs;
use std::io::{self, stdout};
use std::path;
use std::process::Command;

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};

type Days = i32;

enum GitStatus {
    UpToDate,
    LocalBehind(Days),
    RemoteBehind(Days),
}

enum ProjectStatus {
    Unchecked,
    Waiting,
    Complete(GitStatus),
    Failed,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn get_directories() -> Result<std::vec::Vec<std::path::PathBuf>, std::io::Error> {
    let cur_dir = std::env::current_dir()?;
    let hub_directories = std::vec![cur_dir.parent().unwrap_or(path::Path::new("/"))];
    let mut entries = std::vec![];
    for d in hub_directories {
        entries = fs::read_dir(d)?
            .filter_map(|d| {
                if let Ok(dir) = d {
                    if dir.path().is_dir() {
                        Some(dir.path().to_path_buf())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
    }
    Ok(entries)
}

async fn get_git_status(dirs: std::vec::Vec<path::PathBuf>) {
    let statuses: Vec<GitStatus> = dirs
        .iter()
        .map(|d| {
            let output = Command::new("git").args(["pull"]).current_dir(&d).output();

            match output {
                Ok(output) => {
                    println!("{:?}", output)
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
            GitStatus::UpToDate
        })
        .collect();
}

fn ui(frame: &mut Frame) {
    let dirs_clone = get_directories().unwrap();
    let dirs: std::vec::Vec<&str> = dirs_clone
        .iter()
        .map(|p| p.to_str().unwrap_or_default())
        .collect();
    frame.render_widget(
        List::new(dirs).block(Block::bordered().title("Project Directories")),
        frame.size(),
    );
}
