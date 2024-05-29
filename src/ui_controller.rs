use std::io::{stdout, Write};
use std::ops::Index;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use crossterm::{cursor, QueueableCommand, style};
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind, poll, read};
use crossterm::style::{Attribute, Color, style, Stylize};
use crossterm::terminal;

use crate::utils::{UIAction, UIMessage, UIMessageType};

struct Terminal {
    width: u16,
    height: u16,
}

pub struct UIController {
    terminal: Terminal,
    history: Vec<String>,
    prompt: String,

    ui_message_receiver: Receiver<UIMessage>,
    ui_action_sender: Sender<UIAction>,
}

impl UIController {
    pub fn new(ui_message_receiver: Receiver<UIMessage>, ui_action_sender: Sender<UIAction>) -> Self {
        Self {
            terminal: Terminal {
                width: 0,
                height: 0,
            },
            history: vec![],
            prompt: "".to_string(),
            ui_message_receiver,
            ui_action_sender,
        }
    }

    pub fn start(&mut self) {
        let mut stdout = stdout();
        let (_w, _h) = terminal::size().unwrap_or((16, 16));
        self.terminal.width = _w;
        self.terminal.height = _h;


        stdout.queue(terminal::EnterAlternateScreen).unwrap();
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        terminal::enable_raw_mode().unwrap();
        stdout.flush().unwrap();

        let mut quit = false;

        while !quit {
            self.draw();

            match self.ui_message_receiver.try_recv() {
                Ok(ui_message) => self.history.push(self.format_ui_message(ui_message)),
                _ => {}
            }

            if poll(Duration::from_millis(100)).expect("Cannot poll events.") {
                // Event is available:
                match read().unwrap() {
                    Event::Key(event) => {
                        match event.code {
                            KeyCode::Char(char) => {
                                if char == 'c' && event.modifiers.contains(KeyModifiers::CONTROL) { // ^C -> exit
                                    quit = true;
                                } else {
                                    self.prompt.push(char);
                                }
                            }
                            KeyCode::Backspace => {
                                self.prompt.pop();
                            }
                            KeyCode::Enter => {
                                self.prompt_enter();
                                self.prompt = "".to_string()
                            }
                            _ => {}
                        }
                    }
                    Event::Paste(content) => {
                        self.prompt.push_str(&content);
                    }
                    Event::Resize(width, height) => {
                        self.terminal.width = width;
                        self.terminal.height = height;
                    }
                    _ => {}
                }
            }
        }


        stdout.queue(terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();
        exit(0)
    }

    fn draw(&self) {
        let mut stdout = stdout();

        let term = &self.terminal;
        stdout
            .queue(terminal::Clear(terminal::ClearType::All)).unwrap()
            .queue(cursor::MoveTo(0, 0)).unwrap();

        for i in 0..term.height - 2 {
            if self.history.len() <= i as usize { break; }
            let data = self.history.index(self.history.len() - i as usize - 1);

            stdout
                .queue(cursor::MoveTo(0, term.height - 3 - i)).unwrap()
                .queue(style::Print(data)).unwrap();
        }

        stdout
            .queue(cursor::MoveTo(0, term.height - 2)).unwrap()
            .queue(style::Print("─".repeat(term.width as usize))).unwrap();

        stdout
            .queue(cursor::MoveTo(1, term.height - 1)).unwrap()
            .queue(style::Print(&self.prompt)).unwrap();

        stdout.flush().unwrap()
    }

    fn format_ui_message(&self, ui_message: UIMessage) -> String {
        match ui_message.message_type {
            UIMessageType::System => {
                format!(
                    "{} {}",
                    style("[SYSM]")
                        .with(Color::White)
                        .on(Color::Blue)
                        .to_string(),
                    style(ui_message.message)
                        .with(Color::Blue)
                        .to_string()
                )
            }
            UIMessageType::SystemError => {
                format!(
                    "{} {}",
                    style("[SYSE]")
                        .with(Color::White)
                        .on(Color::Red)
                        .to_string(),
                    style(ui_message.message)
                        .with(Color::Red)
                        .to_string()
                )
            }
            UIMessageType::Chat => {
                format!(
                    "{} {}: {}",
                    style("[CHAT]")
                        .with(Color::White)
                        .on(Color::DarkGrey)
                        .to_string(),
                    style(ui_message.author)
                        .attribute(Attribute::Bold)
                        .to_string(),
                    style(ui_message.message)
                        .to_string()
                )
            }
            UIMessageType::DM => {
                format!(
                    "{} {}: {}",
                    style("[ DM ]")
                        .with(Color::White)
                        .on(Color::Magenta)
                        .to_string(),
                    style(ui_message.author)
                        .attribute(Attribute::Bold)
                        .to_string(),
                    style(ui_message.message)
                        .to_string()
                )
            }
        }
    }

    fn prompt_enter(&mut self) {
        let mut prompt = self.prompt.trim_start();

        if prompt.starts_with("/") {
            // is a command
            let mut props: Vec<&str> = prompt.split(" ").collect();

            match props[0] {
                "/dm" => {
                    // self.history.push(self.format_ui_message(UIMessage {
                    //     message_type: UIMessageType::DM,
                    //     author: String::from("gigachad → gigachad2"),
                    //     message: "I love dming publicly".to_string(),
                    // }));
                }
                "/alias" => {
                    if props.len() != 2 {
                        self.history.push(self.format_ui_message(
                            UIMessage::system_error(
                                "Not enough arguments. Usage: /alias <your_alias>".to_string()
                            )
                        ));
                        return;
                    }

                    self.ui_action_sender.send(
                        UIAction::ChangeAlias(props[1].to_string())
                    ).unwrap();
                }
                _ => {
                    self.history.push(self.format_ui_message(
                        UIMessage::system_error(format!("Unknown command \"{}\".", props[0]))
                    ));
                }
            }
        } else {
            // is a chat message
            self.ui_action_sender.send(
                UIAction::SendMessage(prompt.to_string())
            ).unwrap()
        }
    }
}