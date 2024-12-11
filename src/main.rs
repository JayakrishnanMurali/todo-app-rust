use ncurses::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::{env, process};

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

type Id = usize;

#[derive(Default)]
struct Ui {
    list_curr: Option<Id>,
    row: usize,
    col: usize,
}

impl Ui {
    fn begin(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }

    fn begin_list(&mut self, id: Id) {
        assert!(self.list_curr.is_none(), "Nested lists are not supported");

        self.list_curr = Some(id);
    }

    fn label(&mut self, text: &str, pair: i16) {
        mv(self.row as i32, self.col as i32);
        attron(COLOR_PAIR(pair));
        let _ = addstr(text);
        attroff(COLOR_PAIR(pair));
        self.row += 1;
    }

    fn list_element(&mut self, list: &str, id: Id) -> bool {
        let id_curr = self.list_curr.expect("List not started");

        self.label(
            list,
            if id_curr == id {
                HIGHLIGHT_PAIR
            } else {
                REGULAR_PAIR
            },
        );

        return false;
    }

    fn end_list(&mut self) {
        self.list_curr = None;
    }

    fn end(&mut self) {}
}

#[derive(Debug)]
enum Status {
    Todo,
    Done,
}

impl Status {
    fn toggle(&self) -> Self {
        match self {
            Status::Todo => Status::Done,
            Status::Done => Status::Todo,
        }
    }
}

fn parse_item(line: &str) -> Option<(Status, &str)> {
    if line.starts_with("TODO: ") {
        return Some((Status::Todo, &line[6..].trim()));
    } else if line.starts_with("DONE: ") {
        return Some((Status::Done, &line[6..].trim()));
    } else {
        return None;
    }
}

fn list_up(_: &Vec<String>, list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr -= 1
    }
}

fn list_down(list: &Vec<String>, list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        *list_curr += 1
    }
}

fn list_transfer(
    list_dst: &mut Vec<String>,
    list_src: &mut Vec<String>,
    list_src_curr: &mut usize,
) {
    if *list_src_curr < list_src.len() {
        list_dst.push(list_src.remove(*list_src_curr));
        if *list_src_curr >= list_src.len() {
            *list_src_curr = list_src.len().saturating_sub(1);
        }
    }
}

fn load_state (todos: &mut Vec<String>, done: &mut Vec<String>, file_path: &str){
        let file = File::open(file_path).unwrap();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            match parse_item(&line.unwrap()) {
                Some((Status::Todo, item)) => todos.push(item.to_string()),
                Some((Status::Done, item)) => done.push(item.to_string()),
                None => {
                    eprintln!("{}:{} - ERROR: Illegal line format", file_path, index + 1,);

                    process::exit(1);
                }
            }
        }
    }

fn save_state(todos: &Vec<String>, done: &Vec<String>, file_path: &str) {
    let mut file = File::create(file_path).unwrap();
    for todo in todos.iter() {
        let _ = writeln!(file, "TODO: {}", todo).unwrap();
    }

    for done in done.iter() {
        let _ = writeln!(file, "DONE: {}", done).unwrap();
    }
}

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    let file_path = {
        match args.next() {
            Some(file_path) => file_path,
            None => {
                eprintln!("Usage: todo <file>");
                eprintln!("ERROR: Missing file path");
                process::exit(1);
            }
        }
    };



    let mut todos = Vec::<String>::new();
    let mut done = Vec::<String>::new();
    let mut todo_curr: usize = 0;
    let mut done_curr: usize = 0;

    load_state(&mut todos, &mut done, &file_path);

    

    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    let mut quit = false;

    let mut status = Status::Todo;

    let mut ui = Ui::default();
    while !quit {
        erase();
        ui.begin(0, 0);
        {
            match status {
                Status::Todo => {
                    ui.label("[TODO] DONE ", REGULAR_PAIR);
                    ui.label("------------", REGULAR_PAIR);
                    ui.begin_list(todo_curr);
                    for (index, todo) in todos.iter().enumerate() {
                        ui.list_element(
                            &format!(" [{}] {}", if index == todo_curr { "-" } else { " " }, todo),
                            index,
                        );
                    }
                    ui.end_list();
                }

                Status::Done => {
                    ui.label(" TODO [DONE]", REGULAR_PAIR);
                    ui.label("------------", REGULAR_PAIR);
                    ui.begin_list(done_curr);
                    for (index, done) in done.iter().enumerate() {
                        ui.list_element(&format!(" [x] {}", done), index);
                    }
                    ui.end_list();
                }
            }
        }
        ui.end();

        refresh();

        let key = getch();
        match key as u8 as char {
            'q' => quit = true,

            'w' => match status {
                Status::Todo => list_up(&todos, &mut todo_curr),
                Status::Done => list_up(&done, &mut done_curr),
            },
            's' => match status {
                Status::Todo => list_down(&todos, &mut todo_curr),
                Status::Done => list_down(&done, &mut done_curr),
            },

            '\t' => {
                status = status.toggle();
            }

            '\n' => match status {
                Status::Todo => list_transfer(&mut done, &mut todos, &mut todo_curr),

                Status::Done => list_transfer(&mut todos, &mut done, &mut done_curr),
            },

            _ => {}
        }
    }

    save_state(&todos, &done, &file_path);
    endwin();
}
