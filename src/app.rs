use std::io::{self, stdout, Stdout};
use std::panic::{set_hook, take_hook};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use crossterm::{
    event,
    ExecutableCommand,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::widgets::block::*;

use crate::utils::{UIAction, UIHelpCommand, UIMessage};

pub struct App {
    history: Vec<Line<'static>>,
    prompt: String,

    ui_message_receiver: Receiver<UIMessage>,
    ui_action_sender: Sender<UIAction>,
    should_quit: bool,
}

impl App {
    pub fn new(ui_message_receiver: Receiver<UIMessage>, ui_action_sender: Sender<UIAction>) -> Self {
        Self {
            history: vec![],
            prompt: String::new(),
            should_quit: false,

            ui_message_receiver,
            ui_action_sender,
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        self.init_panic_hook();

        let mut terminal = self.init_terminal()?;
        terminal.show_cursor()?;

        while !self.should_quit {
            terminal.draw(|frame| {
                self.ui(frame);
            })?;
            self.handle_events()?;
        }

        Self::restore_terminal()?;
        Ok(())
    }

    fn init_panic_hook(&self) {
        let original_hook = take_hook();

        set_hook(Box::new(move |panic_info| {
            let _ = Self::restore_terminal();
            original_hook(panic_info);
        }));
    }

    fn init_terminal(&self) -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        Terminal::new(CrosstermBackend::new(stdout()))
    }

    fn restore_terminal() -> io::Result<()> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match self.ui_message_receiver.try_recv() {
            Ok(ui_message) => {
                self.history.push(self.format_ui_message(ui_message))
            }
            _ => {}
        }

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char(char) => {
                            if char == 'c' && key.modifiers.contains(KeyModifiers::CONTROL) {
                                self.should_quit = true;
                                return Ok(());
                            } else {
                                self.prompt.push(char);
                            }
                        }
                        KeyCode::Backspace => {
                            self.prompt.pop();
                        }
                        KeyCode::Enter => {
                            self.prompt_enter();
                            self.prompt.clear();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn ui(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3)
            ])
            .split(frame.size());

        let chat_rect = layout[0];
        let input_rect = layout[1];

        let offset_v: u16 = if (self.history.len() as u16) <= chat_rect.height {
            0
        } else {
            (self.history.len() as u16) - chat_rect.height
        };

        let chat = Paragraph::new(
            Text::from(self.history.clone())
        ).scroll((offset_v, 0));

        frame.render_widget(
            chat,
            chat_rect.inner(&Margin::new(1, 0)),
        );

        frame.render_widget(
            Paragraph::new(format!(" {}", self.prompt))
                .block(
                    Block::bordered()
                        .border_style(Style::new().dim())
                ),
            input_rect,
        );


        frame.set_cursor(input_rect.x + 2 + self.prompt.len() as u16, input_rect.y + 1);
    }

    fn prompt_enter(&mut self) {
        let prompt = self.prompt.trim_start();

        if prompt.starts_with("/") {
            // is a command
            let props: Vec<&str> = prompt.split(" ").collect();

            match props[0] {
                "/dm" => {
                    self.history.push(self.format_ui_message(
                        UIMessage::SystemError(
                            "Not implemented.".to_string()
                        )
                    ));
                }
                "/exit" | "/q" => {
                    self.should_quit = true;
                }
                "/list" => {
                    self.history.push(self.format_ui_message(
                        UIMessage::SystemError(
                            "Not implemented.".to_string()
                        )
                    ));
                }
                "/topic" => {
                    self.history.push(self.format_ui_message(
                        UIMessage::SystemError(
                            "Not implemented.".to_string()
                        )
                    ));
                }
                "/alias" => {
                    if props.len() != 2 {
                        self.history.push(self.format_ui_message(
                            UIMessage::SystemError(
                                "Not enough arguments. Usage: /alias <your_alias>".to_string()
                            )
                        ));
                        return;
                    }

                    self.ui_action_sender.send(
                        UIAction::ChangeAlias(props[1].to_string())
                    ).unwrap();
                }
                "/help" => {
                    let mut commands: Vec<UIHelpCommand> = vec![];

                    commands.push(UIHelpCommand {
                        name: String::from("/exit, /q"),
                        description: String::from("Exit from chat"),
                    });

                    commands.push(UIHelpCommand {
                        name: String::from("/list"),
                        description: String::from("Display list of announced clients"),
                    });

                    commands.push(UIHelpCommand {
                        name: String::from("/room <new_room>"),
                        description: String::from("Change room (MQTT Topic)"),
                    });

                    commands.push(UIHelpCommand {
                        name: String::from("/alias <new_alias>"),
                        description: String::from("Change alias"),
                    });

                    commands.push(UIHelpCommand {
                        name: String::from("/dm"),
                        description: String::from("Not implemented"),
                    });

                    let max_size = commands.iter()
                        .map(|cmd| cmd.name.len())
                        .max().unwrap();

                    self.history.push(self.format_ui_message(
                        UIMessage::System(
                            "Available commands:".to_string()
                        )
                    ));
                    for command in commands {
                        self.history.push(self.format_ui_message(
                            UIMessage::System(
                                format!(
                                    " {}{} - {}",
                                    command.name,
                                    " ".repeat(max_size - command.name.len()),
                                    command.description
                                )
                            )
                        ));
                    }
                }
                _ => {
                    self.history.push(self.format_ui_message(
                        UIMessage::SystemError(format!("Unknown command \"{}\".", props[0]))
                    ));
                }
            }
        } else {
            // is a chat message
            self.ui_action_sender.send(
                UIAction::SendMessage(prompt.trim().to_string())
            ).unwrap()
        }
    }

    fn format_ui_message(&self, ui_message: UIMessage) -> Line<'static> {
        match ui_message {
            UIMessage::System(message) => {
                Line::from(vec![
                    " SYSM "
                        .fg(Color::White)
                        .bg(Color::Blue),
                    " ".into(),
                    message
                        .fg(Color::Blue),
                ])
            }

            UIMessage::SystemError(message) => {
                Line::from(vec![
                    " SYSE "
                        .fg(Color::White)
                        .bg(Color::Red),
                    " ".into(),
                    message.fg(Color::Red),
                ])
            }

            UIMessage::Chat(author, message) => {
                Line::from(vec![
                    " CHAT "
                        .fg(Color::White)
                        .bg(Color::Gray),
                    " ".into(),
                    author.alias.clone()
                        .bold(),
                    " ".into(),
                    author.get_pubkey_hash().unwrap_or("......".to_string())
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::DIM),
                    ": ".into(),
                    message.into(),
                ])
            }

            UIMessage::DM(author1, author2, message) => {
                Line::from(vec![
                    "  DM  "
                        .bg(Color::White)
                        .fg(Color::Magenta),
                    " ".into(),
                    /* AUTHORS START */
                    author1.alias.clone()
                        .into(),
                    author1.get_pubkey_hash()
                        .unwrap_or("......".to_string())
                        .add_modifier(Modifier::DIM),
                    " → ".into(),
                    author2.alias.clone()
                        .into(),
                    author2.get_pubkey_hash()
                        .unwrap_or("......".to_string())
                        .add_modifier(Modifier::DIM),
                    /* AUTHORS END */

                    Span::styled(": ", Style::default()).into(),
                    Span::styled(
                        message,
                        Style::default(),
                    ).into(),
                ])
            }
        }
    }
}