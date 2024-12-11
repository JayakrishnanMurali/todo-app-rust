
use ncurses::*;

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

enum Tabs {
    Todo,
    Done,
}

impl Tabs {
    fn toggle(&self) -> Self {
        match self {
            Tabs::Todo => Tabs::Done,
            Tabs::Done => Tabs::Todo,
        }
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

fn list_transfer (list_dst: &mut Vec<String>, list_src: &mut Vec<String>, list_src_curr: &mut usize) {
    if *list_src_curr < list_src.len() {
        list_dst.push(list_src.remove(*list_src_curr));
        if *list_src_curr >= list_src.len() {
            *list_src_curr = list_src.len().saturating_sub(1);
        }
    }
}

fn main() {
    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    let mut quit = false;
    let mut todos: Vec<String> = vec![
        "Write a todo app".to_string(),
        "Learn Rust".to_string(),
        "Invent a time machine".to_string(),
    ];
    let mut done: Vec<String> = vec![
        "Listen to music".to_string(),
        "Hey this is done".to_string(),
    ];

    let mut todo_curr: usize = 0;
    let mut done_curr: usize = 0;

    let mut tabs = Tabs::Todo;

    let mut ui = Ui::default();
    while !quit {
        erase();
        ui.begin(0, 0);
        {
            match tabs {
                Tabs::Todo => {
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

                Tabs::Done => {
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
            'w' => match tabs {
                Tabs::Todo => list_up(&todos, &mut todo_curr),
                Tabs::Done => list_up(&done, &mut done_curr),
            },
            's' => match tabs {
                Tabs::Todo => list_down(&todos, &mut todo_curr),
                Tabs::Done => list_down(&done, &mut done_curr),
            },

            '\t' => {
                tabs = tabs.toggle();
            }

            '\n' => match tabs {
                Tabs::Todo => list_transfer(&mut done, &mut todos, &mut todo_curr),

                Tabs::Done => list_transfer(&mut todos, &mut done, &mut done_curr),
            },

            _ => {}

            }
    }

    endwin();
}
