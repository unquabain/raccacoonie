use raccacoonie::prelude::{
    Result as RacResult,
    *
};
use anyhow::{
    Result,
    *
};
use std::fmt::{Display, Formatter, Error as FmtError};

#[derive(Clone)]
struct Person {
    name: &'static str,
    phone: &'static str,
}

impl Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(),FmtError> {
        write!(f, "{}", self.name)
    }
}
#[derive(Clone)]
struct Team {
    name: &'static str,
    people: Vec<Person>,
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(),FmtError> {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone)]
struct Department {
    name: &'static str,
    teams: Vec<Team>,
}

impl Display for Department {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(),FmtError> {
        write!(f, "{}", self.name)
    }
}

struct Form {
    tab_controller: TabController,
    departments: ListView<Department>,
    teams: ListView<Team>,
    people: ListView<Person>,
    name: InputControl,
    phone: InputControl,
    ok: Button,
    cancel: Button,
    pub chosen_person: Option<(String, String)>
}

impl Form {
    fn new(company: Vec<Department>) -> Self {
        Self{
            tab_controller: TabController::new(7),
            departments: ListView::new("Departments", company),
            teams: ListView::new("Teams", std::iter::empty()),
            people: ListView::new("People", std::iter::empty()),
            name: InputControl::from_label("Name"),
            phone: InputControl::from_label("Phone"),
            ok: Button::new("OK", Message::Yes),
            cancel: Button::new("Cancel", Message::No),
            chosen_person: None,
        }
    }
    fn update_choosers(&mut self, msg: &Message) -> Message {
        macro_rules! choose {
            ( $from:ident => $to:ident ) => {
                match self.$from.chosen.clone() {
                    None => Message::Noop,
                    Some(item) => {
                        self.$to = self.$to.with_items(item.$to);
                        self.tab_controller.next();
                        self.$to.init()
                    }
                }
            }
        }
        match *msg {
            Message::Choice(_) => {
                match self.tab_controller.get_current_index() {
                    0 => choose!( departments => teams ),
                    1 => choose!( teams => people ),
                    2 => {
                        match self.people.chosen.clone() {
                            None => Message::Noop,
                            Some(person) => {
                                self.name = InputControl::from_label_and_value("Name", person.name);
                                self.phone = InputControl::from_label_and_value("Phone", person.phone);
                                Message::Redraw
                            }
                        }
                    }
                    _ => Message::Noop,
                }
            }
            _ => Message::Noop,
        }
    }
    fn update_controls(&mut self, msg: Message) -> Message {
        match self.tab_controller.get_current_index() {
            0 => self.departments.update(msg),
            1 => self.teams.update(msg),
            2 => self.people.update(msg),
            3 => self.name.update(msg),
            4 => self.phone.update(msg),
            5 => self.ok.update(msg),
            6 => self.cancel.update(msg),
            _ => Message::Noop,
        }
    }
    fn update_actions(&mut self, msg: &Message) -> Message {
        match *msg {
            Message::Yes => {
                self.chosen_person = Some((self.name.value(), self.phone.value()));
                Message::Quit
            }
            Message::No => {
                self.chosen_person = None;
                Message::Quit
            }
            _ => Message::Noop,
        }
    }
}

impl Model for Form {
    fn help(&self) -> Option<String> {
        match self.tab_controller.get_current_index() {
            0 => self.departments.help(),
            1 => self.teams.help(),
            2 => self.people.help(),
            3 => self.name.help(),
            4 => self.phone.help(),
            5 => self.ok.help(),
            6 => self.cancel.help(),
            _ => None,
        }
    }
    fn init(&mut self) -> Message {
        self.departments.init()
    }
    fn update(&mut self, msg: Message) -> Message {
        self.tab_controller.update(&msg)
            .or_else(|| self.update_actions(&msg))
            .or_else(|| self.update_choosers(&msg))
            .or_else(|| self.update_controls(msg))
    }
    fn view(&mut self, frame: &mut Frame, area: Rect) -> RacResult<()> {
        use ratatui::widgets::Paragraph;
        let rows: [Rect; 4] = Layout::vertical([
            Constraint::Ratio(1, 3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(3),
        ]).areas(area);
        let pickers: [Rect; 3] = Layout::horizontal([
            Constraint::Ratio(1,3),
            Constraint::Ratio(1,3),
            Constraint::Ratio(1,3),
        ]).areas(rows[0]);
        let form_rows: [Rect; 2] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
        ]).areas(rows[1].inner(Margin::new(1,1)));
        let button_areas: [Rect; 2] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).areas(rows[2]);
        frame.render_widget(
            Paragraph::new("").block(
                raccacoonie::styles::STYLES.blur.block.clone()
            ),
            rows[1],
        );
        frame.render_widget(
            Paragraph::new("").block(
                raccacoonie::styles::STYLES.blur.block.clone()
            ),
            rows[2],
        );
        let control_areas = [
            pickers[0],
            pickers[1],
            pickers[2],
            form_rows[0],
            form_rows[1],
            button_areas[0],
            button_areas[1],
        ];
        for ( idx, area, focused ) in self.tab_controller.iter_with_areas(control_areas) {
            macro_rules! draw_model {
                ( $( ( $idx:literal => $control:ident ) )+ ) => {
                    {
                        match idx {
                            $(
                                $idx => {
                                    self.$control.set_focus(focused);
                                    self.$control.view(frame, area)?;
                                }
                            )+
                            _ => (),
                        }
                    }
                }
            }
            draw_model!(
                (0 => departments)
                (1 => teams)
                (2 => people)
                (3 => name)
                (4 => phone)
                (5 => ok)
                (6 => cancel)
            )
        }
        RacResult::Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging(log::LevelFilter::Debug)?;

    let company = vec![
        Department{
            name: "Product",
            teams: vec![
                Team{
                    name: "Mobile",
                    people: vec![
                        Person{
                            name: "Jim Bob",
                            phone: "555-555-1212",
                        },
                        Person{
                            name: "Kile Jenkins",
                            phone: "555-555-1212",
                        },
                        Person{
                            name: "Toby Perkins",
                            phone: "555-555-1212",
                        },
                        Person{
                            name: "Paul Paulson",
                            phone: "555-555-1212",
                        },
                    ],
                },
                Team{
                    name: "Web",
                    people: vec![
                        Person{
                            name: "Puddintaine",
                            phone: "555-555-1212",
                        },
                        Person{
                            name: "Peter Perkie",
                            phone: "555-555-1212",
                        },
                        Person{
                            name: "Eugene Thompson",
                            phone: "555-555-1212",
                        },
                    ],
                }
            ]
        },
        Department{
            name: "Sales",
            teams: vec![
                Team{
                    name: "US East",
                    people: vec![
                        Person{
                            name: "Roger McKracken",
                            phone: "555-555-1212",
                        },
                        Person{
                            name: "Sally Sherman",
                            phone: "555-555-1212",
                        },
                        Person{
                            name: "Polly Zane",
                            phone: "555-555-1212",
                        },
                    ],
                },
                Team{
                    name: "US West",
                    people: vec![
                        Person{
                            name: "Moonbeam Clinton",
                            phone: "555-555-1212",
                        },
                        Person{
                            name: "Neal Nielsen",
                            phone: "555-555-1212",
                        },
                        Person{
                            name: "Greg McGreggor",
                            phone: "555-555-1212",
                        },
                    ],
                },
            ],
        }
    ];

    let mut form = LogViewer::new(Form::new(company));
    form.run().await?;
    let form = form.into_model();
    match form.chosen_person {
        Some((name, phone)) => {
            Popup::show(format!("Chose {name}, who may be reached at {phone}")).await?;
        }
        None => {
            Popup::show("Declined to choose anybody.").await?;
        }
    }
    
    Ok(())
}
