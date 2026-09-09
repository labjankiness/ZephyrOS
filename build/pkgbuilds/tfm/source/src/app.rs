use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum Panel { Left, Right }

#[derive(Clone)]
pub struct PanelState {
    pub path: PathBuf,
    pub entries: Vec<PathBuf>,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl PanelState {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut state = PanelState { path, entries: vec![], selected: 0, scroll_offset: 0 };
        state.refresh()?;
        Ok(state)
    }

    pub fn refresh(&mut self) -> Result<()> {
        let mut entries: Vec<PathBuf> = fs::read_dir(&self.path)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        entries.sort_by(|a, b| {
            let a_dir = a.is_dir();
            let b_dir = b.is_dir();
            if a_dir != b_dir { return b_dir.cmp(&a_dir); }
            a.file_name().cmp(&b.file_name())
        });
        self.entries = entries;
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        Ok(())
    }

    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.entries.get(self.selected)
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    pub fn move_down(&mut self, visible_height: usize) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
            if self.selected >= self.scroll_offset + visible_height {
                self.scroll_offset = self.selected - visible_height + 1;
            }
        }
    }
}

pub enum InputMode {
    Normal,
    Rename(String),
    NewDir(String),
}

pub struct App {
    pub left: PanelState,
    pub right: PanelState,
    pub active: Panel,
    pub input_mode: InputMode,
    pub status: String,
}

impl App {
    pub fn new() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let home = dirs_next()?;
        Ok(App {
            left: PanelState::new(cwd)?,
            right: PanelState::new(home)?,
            active: Panel::Left,
            input_mode: InputMode::Normal,
            status: String::from("q: quit  tab: switch panel  enter: open  backspace: up  d: delete  r: rename  n: new dir"),
        })
    }

    pub fn active_panel(&self) -> &PanelState {
        match self.active { Panel::Left => &self.left, Panel::Right => &self.right }
    }

    pub fn active_panel_mut(&mut self) -> &mut PanelState {
        match self.active { Panel::Left => &mut self.left, Panel::Right => &mut self.right }
    }

    pub fn switch_panel(&mut self) {
        self.active = match self.active { Panel::Left => Panel::Right, Panel::Right => Panel::Left };
    }

    pub fn move_up(&mut self) {
        self.active_panel_mut().move_up();
    }

    pub fn move_down(&mut self) {
        self.active_panel_mut().move_down(20);
    }

    pub fn enter(&mut self) -> Result<()> {
        let path = self.active_panel().selected_path().cloned();
        if let Some(p) = path {
            if p.is_dir() {
                self.active_panel_mut().path = p;
                self.active_panel_mut().selected = 0;
                self.active_panel_mut().scroll_offset = 0;
                self.active_panel_mut().refresh()?;
            }
        }
        Ok(())
    }

    pub fn go_parent(&mut self) -> Result<()> {
        let parent = self.active_panel().path.parent().map(|p| p.to_path_buf());
        if let Some(p) = parent {
            let old = self.active_panel().path.clone();
            self.active_panel_mut().path = p;
            self.active_panel_mut().refresh()?;
            // Try to select the directory we came from
            if let Some(idx) = self.active_panel().entries.iter().position(|e| *e == old) {
                self.active_panel_mut().selected = idx;
            }
        }
        Ok(())
    }

    pub fn delete(&mut self) -> Result<()> {
        let path = self.active_panel().selected_path().cloned();
        if let Some(p) = path {
            if p.is_dir() {
                fs::remove_dir_all(&p)?;
            } else {
                fs::remove_file(&p)?;
            }
            self.status = format!("Deleted: {}", p.display());
            self.active_panel_mut().refresh()?;
        }
        Ok(())
    }

    pub fn rename_start(&mut self) {
        if let Some(p) = self.active_panel().selected_path() {
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            self.input_mode = InputMode::Rename(name);
        }
    }

    pub fn new_dir_start(&mut self) {
        self.input_mode = InputMode::NewDir(String::new());
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn handle_char(&mut self, c: char) {
        match &mut self.input_mode {
            InputMode::Rename(s) | InputMode::NewDir(s) => {
                if c == '\n' {
                    self.confirm_input();
                } else {
                    s.push(c);
                }
            }
            InputMode::Normal => {}
        }
    }

    fn confirm_input(&mut self) {
        let mode = std::mem::replace(&mut self.input_mode, InputMode::Normal);
        match mode {
            InputMode::Rename(new_name) => {
                if let Some(old) = self.active_panel().selected_path().cloned() {
                    let new = old.parent().unwrap_or(&old).join(&new_name);
                    if let Err(e) = fs::rename(&old, &new) {
                        self.status = format!("Rename failed: {e}");
                    } else {
                        self.status = format!("Renamed to {new_name}");
                        let _ = self.active_panel_mut().refresh();
                    }
                }
            }
            InputMode::NewDir(name) => {
                if !name.is_empty() {
                    let path = self.active_panel().path.join(&name);
                    if let Err(e) = fs::create_dir(&path) {
                        self.status = format!("Failed to create dir: {e}");
                    } else {
                        self.status = format!("Created: {name}");
                        let _ = self.active_panel_mut().refresh();
                    }
                }
            }
            InputMode::Normal => {}
        }
    }
}

fn dirs_next() -> Result<PathBuf> {
    Ok(std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/")))
}
