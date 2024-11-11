use std::{
    env,
    fs::{self},
    path::Path,
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState},
};

struct MyState {
    ls: Vec<Vec<MyFile>>,
    list_states: Vec<ListState>,
    selected: usize,
    paths: Vec<String>,
}

#[derive(PartialEq, Eq)]
struct MyFile {
    name: String,
    is_dir: bool,
}

impl Ord for MyFile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for MyFile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}
impl MyState {
    fn new(cur_path: String) -> Self {
        let mut ls = Vec::new();
        ls.push(Vec::new());
        ls.push(Vec::new());
        let mut list_states = Vec::new();
        list_states.push(ListState::default());
        list_states.push(ListState::default());
        let mut paths = Vec::new();
        paths.push(String::new());
        paths.push(String::new());

        let mut res = MyState {
            ls,
            list_states,
            paths,
            selected: 0,
        };
        res.update_dir(&cur_path);
        res.selected = 1;
        res.update_dir(&cur_path);
        res
    }
    fn update_dir(&mut self, new_path: &str) {
        let ns = Path::new(&self.paths[self.selected])
            .join(new_path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut ls: Vec<MyFile> = fs::read_dir(&ns)
            .unwrap()
            .map(|file| MyFile {
                name: file
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .to_string(),
                is_dir: file.as_ref().unwrap().file_type().unwrap().is_dir(),
            })
            .collect();
        ls.push(MyFile {
            name: "..".to_string(),
            is_dir: true,
        });
        ls.sort();
        self.ls[self.selected] = ls;
        self.list_states[self.selected].select_first();
        self.paths[self.selected] = ns;
    }

    fn select_next(&mut self) {
        self.selected = (self.selected + 1).min(self.list_states.len() - 1);
    }
    fn select_previous(&mut self) {
        self.selected = (self.selected + 1).max(2) - 2;
    }

    fn add_list(&mut self, idx: usize){
        let path = self.paths[self.selected].clone();
        self.paths.insert(idx, path.clone());
        self.list_states.insert(idx, ListState::default().with_selected(Some(0)));
        self.ls.insert(idx,Vec::new());
        self.selected = idx;
        self.update_dir(&path);
    }
    fn del_list(&mut self, idx: usize){
        if self.ls.len() > 1{
        self.paths.remove(idx);
        self.list_states.remove(idx);
        self.ls.remove(idx);
        self.select_previous();
        }
    }
}

impl<'a> Into<ListItem<'a>> for &MyFile {
    fn into(self) -> ListItem<'a> {
        ListItem::new(format!("{}", self.name))
    }
}

pub fn run() {
    let mut term = ratatui::init();
    let cur_path = env::current_dir().unwrap().to_str().unwrap().to_string();
    let mut my_state = MyState::new(cur_path);
    loop {
        term.draw(|frame| draw(frame, &mut my_state)).unwrap();
        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Up => my_state.list_states[my_state.selected].select_previous(),
                    KeyCode::Down => my_state.list_states[my_state.selected].select_next(),
                    KeyCode::Enter => {
                        let (is_dir, name) = {
                            let file: &MyFile = &my_state.ls[my_state.selected]
                                [my_state.list_states[my_state.selected].selected().unwrap()];
                            (file.is_dir, file.name.clone())
                        };
                        if is_dir {
                            my_state.update_dir(&name);
                        }
                    }
                    KeyCode::Left => {
                        my_state.select_previous();
                    }
                    KeyCode::Right => {
                        my_state.select_next();
                    }
                    KeyCode::Char('v') => {
                        my_state.add_list(my_state.selected);
                    }
                    KeyCode::Backspace => {
                        my_state.del_list(my_state.selected);
                    }
                    _ => {}
                }
            }
        }
    }
    ratatui::restore();
}

fn draw(frame: &mut ratatui::Frame, state: &mut MyState) {
    let mut paths: Vec<Text> = Vec::new();
    state
        .paths
        .iter()
        .for_each(|path| paths.push(Text::raw(path)));
    let mut styles: Vec<Style> = Vec::new();
    
    state.list_states.iter().enumerate().for_each(|(idx, _)| {
        if state.selected == idx {
            styles.push(Style::new().yellow());
        }
        else{
            styles.push(Style::new())
        }
    });
    let mut lists = Vec::new();
    state.ls.iter().enumerate().for_each(|(idx, ls)| {
        let list = List::new(ls)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Files List")
                    .border_style(styles[idx]),
            )
            .highlight_style(Style::new().reversed());
        lists.push(list);
    });

    let mut constraints = Vec::new();
    let len = lists.len();
    let layout_help = Layout::vertical([Constraint::Percentage(99), Constraint::Fill(1)]);
    lists.iter().for_each(|_| constraints.push(Constraint::Ratio(1, len as u32)));
    let laylists = Layout::horizontal(constraints);
    let lay = Layout::vertical([Constraint::Percentage(95), Constraint::Fill(1)]);
    let main_rect = layout_help.split(frame.area());
    let rect = laylists.split(main_rect[0]);
    let final_layout = lists.iter().enumerate().map(|(idx, _)| lay.split(rect[idx]));

    final_layout.enumerate().for_each(|(idx, lay)| {
        frame.render_widget(&paths[idx], lay[1]);
        frame.render_stateful_widget(&lists[idx], lay[0], &mut state.list_states[idx]);
    });
    frame.render_widget(Text::raw("V: Add explorer; BackSpace: Del explorer; <> Navigate"), main_rect[1]);
}
