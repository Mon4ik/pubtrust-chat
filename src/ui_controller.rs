use std::cmp::min;
use std::io::{self, stdout};
use std::sync::mpsc::{Receiver, Sender};

use crossterm::{event, ExecutableCommand, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::widgets::block::Title;

use crate::utils::{UIAction, UIHelpCommand, UIMessage};

pub struct UIController {
    history: Vec<Line<'static>>,
    prompt: String,

    ui_message_receiver: Receiver<UIMessage>,
    ui_action_sender: Sender<UIAction>,
}

impl UIController {
    pub fn new(ui_message_receiver: Receiver<UIMessage>, ui_action_sender: Sender<UIAction>) -> Self {
        // let mut lines = vec![];
        // lines.push(Line::from(vec![
        //     Span::styled("Hello ", Style::default().fg(Color::Yellow)),
        //     Span::styled("World", Style::default().fg(Color::Blue).bg(Color::White)),
        // ]));
        //

        Self {
            history: vec![],
            prompt: String::new(),
            ui_message_receiver,
            ui_action_sender,
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let mut should_quit = false;
        while !should_quit {
            terminal.draw(|frame| {
                self.ui(frame);
            })?;
            should_quit = self.handle_events()?;
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<bool> {
        match self.ui_message_receiver.try_recv() {
            Ok(ui_message) => {
                self.history.push(self.format_ui_message(ui_message))
            }
            _ => {}
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char(char) => {
                            if char == 'c' && key.modifiers.contains(KeyModifiers::CONTROL) {
                                return Ok(true);
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
        Ok(false)
    }

    fn ui(&mut self, frame: &mut Frame) {
        let chat_rect = Rect::new(0, 0, frame.size().width, frame.size().height - 3);

        let chat_block = Block::bordered()
            .border_type(BorderType::Double)
            .border_style(Style::new().gray())
            .title(Title::from("[PubTrust Chat]".reset()).alignment(Alignment::Center));

        let offset_v: u16 = if (self.history.len() as u16) <= chat_rect.height - 2 {
            0
        } else {
            (self.history.len() as u16) - (chat_rect.height - 2)
        };

        let chat = Paragraph::new(
            Text::from(self.history.clone())
        ).scroll((offset_v, 0));

        frame.render_widget(
            chat.block(chat_block),
            chat_rect
        );

        frame.render_widget(
            Paragraph::new(format!(" {}", self.prompt))
                .block(
                    Block::bordered()
                        .border_style(Style::new().gray())
                ),
            Rect::new(0, frame.size().height - 3, frame.size().width, 3),
        );
    }

    fn format_ui_message(&self, ui_message: UIMessage) -> Line<'static> {
        match ui_message {
            UIMessage::System(message) => {
                Line::from(vec![
                    Span::styled(
                        "[SYSM]",
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Blue),
                    ),
                    Span::styled(" ", Style::default()),
                    Span::styled(
                        message,
                        Style::default()
                            .fg(Color::Blue),
                    ),
                ])
            }

            UIMessage::SystemError(message) => {
                Line::from(vec![
                    Span::styled(
                        "[SYSE]",
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Red),
                    ),
                    Span::styled(" ", Style::default()),
                    Span::styled(
                        message,
                        Style::default()
                            .fg(Color::Red),
                    ),
                ])
            }

            UIMessage::Chat(author, message) => {
                Line::from(vec![
                    Span::styled(
                        "[CHAT]",
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::DarkGray),
                    ),
                    " ".into(),
                    author.alias.clone()
                        .bold(),
                    " ".into(),
                    author.get_pubkey_hash().unwrap_or("......".to_string())
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::DIM),
                    ": ".into(),
                    Span::styled(
                        message,
                        Style::default(),
                    ),
                ])
            }

            UIMessage::DM(author1, author2, message) => {
                Line::from(vec![
                    "[ DM ]"
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

    fn prompt_enter(&mut self) {
        let mut prompt = self.prompt.trim_start();

        if prompt.starts_with("/") {
            // is a command
            let mut props: Vec<&str> = prompt.split(" ").collect();

            match props[0] {
                "/dm" => {
                    self.history.push(self.format_ui_message(
                        UIMessage::SystemError(
                            "Not implemented.".to_string()
                        )
                    ));
                }
                "/exit" | "/q" => {
                    // self.quit = true;
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
                    // style("Available commands:")
                    //     .attribute(Attribute::Bold),
                    // style(" /exit, /q - Exit from chat"),
                    // style(" /list     - "),
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

                    // self.history.push(self.format_ui_message(
                    //     UIMessage::System(
                    //         style("Available commands:")
                    //             .attribute(Attribute::Bold)
                    //             .to_string()
                    //     )
                    // ));
                    // for command in commands {
                    //     self.history.push(self.format_ui_message(
                    //         UIMessage::System(
                    //             format!(
                    //                 " {}{} {} {}",
                    //                 style(&command.name)
                    //                     .attribute(Attribute::Bold)
                    //                     .to_string(),
                    //                 " ".repeat(max_size - command.name.len()),
                    //                 "-".with(crossterm::style::Color::Blue),
                    //                 command.description
                    //                     .with(crossterm::style::Color::Blue)
                    //             )
                    //         )
                    //     ));
                    // }
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
                UIAction::SendMessage(prompt.to_string())
            ).unwrap()
        }
    }
}