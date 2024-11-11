use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState},
};

struct MyState {
    ls: Vec<Vec<MyFile>>,
    list_states: Vec<ListState>,
    selected: usize,
    reversed: bool,
    paths: Vec<String>,
    opened: Opened,
    last_oper: bool,
}

#[derive(PartialEq, Eq, Clone)]
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
#[derive(Clone)]
enum Opened {
    List,
    Move,
    Copy,
}

impl MyState {
    fn build(cur_path: String) -> Result<Self, &'static str> {
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
            reversed: false,
            opened: Opened::List,
            last_oper: true,
        };
        res.update_dir(&cur_path)?;
        res.selected = 1;
        res.update_dir(&cur_path)?;
        Ok(res)
    }
    fn update_dir(&mut self, new_path: &str) -> Result<(), &'static str> {
        let ns = Path::new(&self.paths[self.selected])
            .join(new_path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut ls: Vec<MyFile> = fs::read_dir(&ns)
            .map_err(|_| "Error at reading dir")?
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
        if self.reversed {
            ls[1..].reverse();
        }
        self.ls[self.selected] = ls;
        self.list_states[self.selected].select_first();
        self.paths[self.selected] = ns;
        Ok(())
    }

    fn select_next(&mut self) {
        self.selected = (self.selected + 1).min(self.list_states.len() - 1);
    }
    fn select_previous(&mut self) {
        self.selected = (self.selected + 1).max(2) - 2;
    }

    fn add_list(&mut self, idx: usize) {
        let path = self.paths[self.selected].clone();
        self.paths.insert(idx, path.clone());
        self.list_states
            .insert(idx, ListState::default().with_selected(Some(0)));
        self.ls.insert(idx, Vec::new());
        self.selected = idx;
        self.update_dir(&path).ok();
    }
    fn del_list(&mut self, idx: usize) {
        if self.ls.len() > 1 {
            self.paths.remove(idx);
            self.list_states.remove(idx);
            self.ls.remove(idx);
            self.select_previous();
        }
    }
    fn change_reversal(&mut self) {
        self.reversed = !self.reversed;

        self.ls.iter_mut().for_each(|v| {
            v.sort();
            if self.reversed {
                v[1..].reverse();
            }
        });
    }
    fn move_file(&mut self, file: &str) -> Result<(), std::io::Error> {
        if Path::new(file).file_name().is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        fs::rename(
            file,
            Path::new(&self.paths[self.selected]).join(Path::new(file).file_name().unwrap()),
        )?;
        self.refresh();
        Ok(())
    }
    fn copy_file(&mut self, file: &str) -> Result<(), std::io::Error> {
        if Path::new(file).file_name().is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        if PathBuf::from(file).is_file() {
            fs::copy(
                file,
                Path::new(&self.paths[self.selected]).join(Path::new(file).file_name().unwrap()),
            )?;
        } else {
            copy(
                file,
                Path::new(&self.paths[self.selected]).join(Path::new(file).file_name().unwrap()),
            )?;
        }
        self.refresh();
        Ok(())
    }
    fn get_current_file(&self) -> String {
        self.ls[self.selected][self.list_states[self.selected].selected().unwrap()]
            .name
            .clone()
    }

    fn del_file(&mut self) -> Result<(), std::io::Error> {
        let file = Path::new(&self.paths[self.selected]).join(self.get_current_file());
        if file.is_file() {
            fs::remove_file(file)?;
        } else {
            fs::remove_dir_all(file)?
        }
        Ok(())
    }

    fn refresh(&mut self) {
        let i = self.selected;
        let len = self.ls.len();
        for idx in 0..len {
            self.selected = idx;
            let path = self.paths[self.selected].clone();
            self.update_dir(&path).ok();
        }
        self.selected = i;
        self.refresh();
    }
}

pub fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {}
                }
            }
        }
    }

    Ok(())
}

impl<'a> Into<ListItem<'a>> for &MyFile {
    fn into(self) -> ListItem<'a> {
        ListItem::new(format!("{}", self.name))
    }
}

pub fn run(cur_path: String) -> Result<(), &'static str> {
    let mut term = ratatui::init();
    let mut to_move: String = "".to_string();
    let mut my_state = MyState::build(cur_path.clone())?;
    loop {
        term.draw(|frame| draw(frame, &mut my_state)).unwrap();
        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                let open = my_state.opened.clone();
                my_state.last_oper = true;
                match open {
                    Opened::List => match key.code {
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
                                my_state
                                    .update_dir(&name)
                                    .inspect_err(|_| my_state.last_oper = false)
                                    .ok();
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
                        KeyCode::Tab => {
                            my_state.change_reversal();
                        }
                        KeyCode::Char('m') => {
                            to_move = Path::new(&my_state.paths[my_state.selected])
                                .join(
                                    my_state.ls[my_state.selected][my_state.list_states
                                        [my_state.selected]
                                        .selected()
                                        .unwrap()]
                                    .name
                                    .clone(),
                                )
                                .to_str()
                                .unwrap()
                                .to_string();
                            my_state.opened = Opened::Move;
                        }

                        KeyCode::Char('c') => {
                            to_move = Path::new(&my_state.paths[my_state.selected])
                                .join(
                                    my_state.ls[my_state.selected][my_state.list_states
                                        [my_state.selected]
                                        .selected()
                                        .unwrap()]
                                    .name
                                    .clone(),
                                )
                                .to_str()
                                .unwrap()
                                .to_string();
                            my_state.opened = Opened::Copy;
                        }
                        KeyCode::Char('d') => {
                            if key.modifiers.contains(KeyModifiers::SHIFT) {
                                my_state
                                    .del_file()
                                    .inspect_err(|_| my_state.last_oper = false)
                                    .ok();
                            }
                        }
                        _ => {}
                    },

                    Opened::Copy => match key.code {
                        KeyCode::Left => {
                            my_state.select_previous();
                        }
                        KeyCode::Right => {
                            my_state.select_next();
                        }
                        KeyCode::Enter => {
                            my_state
                                .copy_file(&to_move)
                                .inspect_err(|_| my_state.last_oper = false)
                                .ok();
                            my_state.opened = Opened::List;
                        }
                        KeyCode::Esc => {
                            my_state.opened = Opened::List;
                        }

                        _ => {}
                    },
                    Opened::Move => match key.code {
                        KeyCode::Left => {
                            my_state.select_previous();
                        }
                        KeyCode::Right => {
                            my_state.select_next();
                        }
                        KeyCode::Enter => {
                            my_state
                                .move_file(&to_move)
                                .inspect_err(|_| my_state.last_oper = false)
                                .ok();
                            my_state.opened = Opened::List;
                        }
                        KeyCode::Esc => {
                            my_state.opened = Opened::List;
                        }

                        _ => {}
                    },
                }
            }
        }
    }
    ratatui::restore();
    Ok(())
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
            if state.last_oper {
                styles.push(Style::new().green());
            } else {
                styles.push(Style::new().red());
            }
        } else {
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
    let layout_help = Layout::vertical([
        Constraint::Percentage(95),
        Constraint::Ratio(1, 2),
        Constraint::Ratio(1, 2),
    ]);
    lists
        .iter()
        .for_each(|_| constraints.push(Constraint::Ratio(1, len as u32)));
    let laylists = Layout::horizontal(constraints);
    let lay = Layout::vertical([Constraint::Percentage(95), Constraint::Fill(1)]);
    let main_rect = layout_help.split(frame.area());
    let rect = laylists.split(main_rect[0]);
    let final_layout = lists
        .iter()
        .enumerate()
        .map(|(idx, _)| lay.split(rect[idx]));

    final_layout.enumerate().for_each(|(idx, lay)| {
        frame.render_widget(&paths[idx], lay[1]);
        frame.render_stateful_widget(&lists[idx], lay[0], &mut state.list_states[idx]);
    });
    frame.render_widget(
        Text::raw("V: Add explorer; BackSpace: Del explorer; <> Navigate; ESC: exit;"),
        main_rect[1],
    );
    frame.render_widget(
        Text::raw("M: Mark file, select explorer and press enter to move it into or esc to cancel; C: same as moving, but copy"),
        main_rect[2],
    );
}
