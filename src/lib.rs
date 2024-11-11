use std::{
    cell::RefCell, env, fs::{self}, path::Path, rc::Rc
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState},
};

struct MyState {
    ls: Rc<RefCell<Vec<MyFile>>>,
    ls2: Rc<RefCell<Vec<MyFile>>>,
    list_state: Rc<RefCell<ListState>>,
    list2_state: Rc<RefCell<ListState>>,
    selected_list_state: Rc<RefCell<ListState>>,
    selected_list: Rc<RefCell<Vec<MyFile>>>,
    path: Rc<RefCell<String>>,
    path2: Rc<RefCell<String>>,
    selected_path:Rc<RefCell<String>>,
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
        let ls = Rc::new(RefCell::new(Vec::new()));
        let ls2 = Rc::new(RefCell::new(Vec::new()));
        let list_state = Rc::new(RefCell::new(ListState::default().with_selected(Some(0))));
        let list2_state = Rc::new(RefCell::new(ListState::default().with_selected(Some(0))));
        let path2 = Rc::new(RefCell::new(String::new()));
        let path = Rc::new(RefCell::new(String::new()));
        let selected_path = Rc::clone(&path2);
        let mut res = MyState {
            ls: Rc::clone(&ls),
            ls2: Rc::clone(&ls2),
            list2_state: Rc::clone(&list2_state),
            selected_list: Rc::clone(&ls2),
            selected_list_state: Rc::clone(&list2_state),
            list_state: Rc::clone(&list_state),
            path,
            selected_path,
            path2
        };
        res.update_dir(&cur_path);
        res.selected_list = Rc::clone(&res.ls);
        res.selected_list_state = Rc::clone(&res.list_state);
        res.selected_path = res.path.clone();
        res.update_dir(&cur_path);
        res
    }
    fn update_dir(&mut self, new_path: &str) {
        let ns = Path::new((*self.selected_path).borrow_mut().as_str())
        .join(new_path)
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
        (*(self.selected_path).borrow_mut()) = ns;
        let mut ls: Vec<MyFile> = fs::read_dir((*self.selected_path).borrow().as_str())
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
        *(*self.selected_list).borrow_mut() = ls;
        (*self.selected_list_state).borrow_mut().select_first();
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
                    KeyCode::Up => (*my_state.selected_list_state)
                        .borrow_mut()
                        .select_previous(),
                    KeyCode::Down => (*my_state.selected_list_state).borrow_mut().select_next(),
                    KeyCode::Enter => {
                        let (is_dir, name) = {
                            let file: &MyFile = &(*my_state.selected_list).borrow()
                                [(*my_state.selected_list_state).borrow().selected().unwrap()];
                            (file.is_dir, file.name.clone())
                        };
                        if is_dir {
                            my_state.update_dir(&name);
                        }
                    }
                    KeyCode::Left => {
                        my_state.selected_list_state = Rc::clone(&my_state.list_state);
                        my_state.selected_list = Rc::clone(&my_state.ls);
                        my_state.selected_path = my_state.path.clone();
                    }
                    KeyCode::Right => {
                        my_state.selected_list_state = Rc::clone(&my_state.list2_state);
                        my_state.selected_list = Rc::clone(&my_state.ls2);
                        my_state.selected_path = my_state.path2.clone();
                    }
                    _ => {}
                }
            }
        }
    }
    ratatui::restore();
}

fn draw(frame: &mut ratatui::Frame, state: &mut MyState) {
    let ns1 = state.path.borrow();
    let ns2 = state.path2.borrow();
    let text = Text::raw(ns1.as_str());
    let text2 = Text::raw(ns2.as_str());
    let mut style1 = Style::new();
    let mut style2 = Style::new();
    if Rc::ptr_eq(&state.selected_list_state, &state.list2_state) {
        style2 = style2.yellow();
    } else {
        style1 = style1.yellow();
    }

    let list = List::new((*state.ls).borrow().iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("First Files List")
                .border_style(style1),
        )
        .highlight_style(Style::new().reversed());

    let list2 = List::new((*state.ls2).borrow().iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Second Files List")
                .border_style(style2),
        )
        .highlight_style(Style::new().reversed());

    let laylists = Layout::horizontal([Constraint::Fill(1); 2]);
    let lay = Layout::vertical([Constraint::Percentage(90), Constraint::Fill(1)]);
    let rect = laylists.split(frame.area());
    let listsrect = lay.split(rect[0]);
    let listsrect2 = lay.split(rect[1]);

    frame.render_stateful_widget(list, listsrect[0], &mut (*state.list_state).borrow_mut());
    frame.render_stateful_widget(list2, listsrect2[0], &mut (*state.list2_state).borrow_mut());
    frame.render_widget(text, listsrect[1]);
    frame.render_widget(text2, listsrect2[1]);
}
