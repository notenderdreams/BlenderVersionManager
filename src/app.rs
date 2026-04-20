use crate::blender::{self, BlenderManager, BlenderVersion, InstalledVersion};
use crate::network;
use crate::ui;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::Backend;
use ratatui::Terminal;
use ratatui::widgets::ListState;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Clone)]
pub enum ViewMode {
    Available,
    Installed,
    ConfirmDelete(String),
}

pub enum Action {
    FetchVersions,
    SetAvailable(Vec<BlenderVersion>),
    SetStatus(String),
    Install(BlenderVersion),
    UpdateProgress(f64),
    RefreshInstalled,
    Remove(String),
    Launch(String),
}

pub struct App {
    pub manager: BlenderManager,
    pub available: Vec<BlenderVersion>,
    pub installed: Vec<InstalledVersion>,
    pub available_state: ListState,
    pub installed_state: ListState,
    pub view_mode: ViewMode,
    pub status: String,
    pub downloading: Option<f64>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let manager = BlenderManager::new()?;
        let installed = manager.list_installed()?;
        
        let mut available_state = ListState::default();
        available_state.select(Some(0));
        
        let mut installed_state = ListState::default();
        installed_state.select(Some(0));

        Ok(Self {
            manager,
            available: Vec::new(),
            installed,
            available_state,
            installed_state,
            view_mode: ViewMode::Available,
            status: "Welcome to BVM! Press 'f' to fetch versions.".to_string(),
            downloading: None,
            should_quit: false,
        })
    }

    pub fn next(&mut self) {
        match self.view_mode {
            ViewMode::Available => {
                if self.available.is_empty() { return; }
                let i = match self.available_state.selected() {
                    Some(i) => if i >= self.available.len() - 1 { 0 } else { i + 1 },
                    None => 0,
                };
                self.available_state.select(Some(i));
            }
            ViewMode::Installed => {
                if self.installed.is_empty() { return; }
                let i = match self.installed_state.selected() {
                    Some(i) => if i >= self.installed.len() - 1 { 0 } else { i + 1 },
                    None => 0,
                };
                self.installed_state.select(Some(i));
            }
            _ => {}
        }
    }

    pub fn previous(&mut self) {
        match self.view_mode {
            ViewMode::Available => {
                if self.available.is_empty() { return; }
                let i = match self.available_state.selected() {
                    Some(i) => if i == 0 { self.available.len() - 1 } else { i - 1 },
                    None => 0,
                };
                self.available_state.select(Some(i));
            }
            ViewMode::Installed => {
                if self.installed.is_empty() { return; }
                let i = match self.installed_state.selected() {
                    Some(i) => if i == 0 { self.installed.len() - 1 } else { i - 1 },
                    None => 0,
                };
                self.installed_state.select(Some(i));
            }
            _ => {}
        }
    }

    pub fn switch_tab(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Available => ViewMode::Installed,
            ViewMode::Installed => ViewMode::Available,
            ViewMode::ConfirmDelete(_) => ViewMode::Installed,
        };
    }
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: Arc<Mutex<App>>,
    tx: mpsc::Sender<Action>,
    mut rx: mpsc::Receiver<Action>,
) -> Result<()> {
    loop {
        {
            let mut app = app.lock().unwrap();
            terminal.draw(|f| ui::ui(f, &mut app))?;
            if app.should_quit {
                return Ok(());
            }
        }

        while let Ok(action) = rx.try_recv() {
            match action {
                Action::FetchVersions => {
                    let tx_c = tx.clone();
                    tokio::spawn(async move {
                        let _ = tx_c.send(Action::SetStatus("Fetching available versions...".to_string())).await;
                        let versions = network::fetch_blender_versions().await.unwrap_or_default();
                        let _ = tx_c.send(Action::SetAvailable(versions)).await;
                        let _ = tx_c.send(Action::SetStatus("Versions fetched.".to_string())).await;
                    });
                }
                Action::SetAvailable(versions) => {
                    app.lock().unwrap().available = versions;
                }
                Action::SetStatus(msg) => {
                    app.lock().unwrap().status = msg;
                }
                Action::UpdateProgress(p) => {
                    app.lock().unwrap().downloading = Some(p);
                }
                Action::RefreshInstalled => {
                    let mut a = app.lock().unwrap();
                    a.installed = a.manager.list_installed().unwrap_or_default();
                    a.downloading = None;
                }
                Action::Install(v) => {
                    let tx_c = tx.clone();
                    let mgr_path = app.lock().unwrap().manager.base_path.clone();
                    tokio::spawn(async move {
                        let _ = network::install_version(v, mgr_path, tx_c).await;
                    });
                }
                Action::Remove(v) => {
                    let mut a = app.lock().unwrap();
                    let _ = a.manager.remove_version(&v);
                    a.installed = a.manager.list_installed().unwrap_or_default();
                }
                Action::Launch(version) => {
                    let a = app.lock().unwrap();
                    let path = a.manager.get_versions_dir().join(&version);
                    let env = a.manager.get_launch_env();
                    blender::launch_blender(path, env)?;
                }
            }
        }

        if event::poll(Duration::from_millis(16))? { 
            if let Event::Key(key) = event::read()? {
                if key.kind != event::KeyEventKind::Press {
                    continue;
                }
                let action = {
                    let mut app = app.lock().unwrap();
                    match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                            None
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.next();
                            None
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.previous();
                            None
                        }
                        KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Char('l') => {
                            app.switch_tab();
                            None
                        }
                        KeyCode::Char('1') => {
                            app.view_mode = ViewMode::Available;
                            None
                        }
                        KeyCode::Char('2') => {
                            app.view_mode = ViewMode::Installed;
                            None
                        }
                        KeyCode::Char('f') => Some(Action::FetchVersions),
                        KeyCode::Enter => {
                            match app.view_mode {
                                ViewMode::Available => {
                                    if let Some(i) = app.available_state.selected() {
                                        app.available.get(i).cloned().map(Action::Install)
                                    } else {
                                        None
                                    }
                                }
                                ViewMode::Installed => {
                                    if let Some(i) = app.installed_state.selected() {
                                        app.installed.get(i).cloned().map(|v| Action::Launch(v.version))
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            }
                        }
                        KeyCode::Char('d') => {
                            if matches!(app.view_mode, ViewMode::Installed) {
                                if let Some(i) = app.installed_state.selected() {
                                    if let Some(v) = app.installed.get(i) {
                                        app.view_mode = ViewMode::ConfirmDelete(v.version.clone());
                                    }
                                }
                            }
                            None
                        }
                        KeyCode::Char('y') => {
                            if let ViewMode::ConfirmDelete(v) = &app.view_mode {
                                let action = Some(Action::Remove(v.clone()));
                                app.view_mode = ViewMode::Installed;
                                action
                            } else {
                                None
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Esc => {
                            if let ViewMode::ConfirmDelete(_) = &app.view_mode {
                                app.view_mode = ViewMode::Installed;
                            }
                            None
                        }
                        _ => None,
                    }
                };

                if let Some(action) = action {
                    tx.send(action).await?;
                }
            } else if let Event::Mouse(mouse) = event::read()? {
                let mut app = app.lock().unwrap();
                match mouse.kind {
                    event::MouseEventKind::ScrollDown => app.next(),
                    event::MouseEventKind::ScrollUp => app.previous(),
                    _ => {}
                }
            }
        }
    }
}
