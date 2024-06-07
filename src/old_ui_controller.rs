// use std::io::{stdout, Write};
// use std::ops::Index;
// use std::process::exit;
// use std::sync::mpsc::{Receiver, Sender};
// use std::time::Duration;
//
// use crossterm::{cursor, QueueableCommand, style};
// use crossterm::event::{Event, KeyCode, KeyModifiers, poll, read};
// use crossterm::style::{Attribute, Color, style, Stylize};
// use crossterm::terminal;
//
// use crate::utils::{UIAction, UIHelpCommand, UIMessage};
//
// struct Terminal {
//     width: u16,
//     height: u16,
// }
//
// pub struct OldUIController {
//     terminal: Terminal,
//     history: Vec<String>,
//     prompt: String,
//     quit: bool,
//
//     ui_message_receiver: Receiver<UIMessage>,
//     ui_action_sender: Sender<UIAction>,
// }
//
// impl OldUIController {
//     pub fn new(ui_message_receiver: Receiver<UIMessage>, ui_action_sender: Sender<UIAction>) -> Self {
//         Self {
//             terminal: Terminal {
//                 width: 0,
//                 height: 0,
//             },
//             history: vec![],
//             prompt: "".to_string(),
//             quit: false,
//             ui_message_receiver,
//             ui_action_sender,
//         }
//     }
//
//     pub fn start(&mut self) {
//         let mut stdout = stdout();
//         let (_w, _h) = terminal::size().unwrap_or((16, 16));
//         self.terminal.width = _w;
//         self.terminal.height = _h;
//
//
//         stdout.queue(terminal::EnterAlternateScreen).unwrap();
//         stdout.queue(cursor::MoveTo(0, 0)).unwrap();
//         terminal::enable_raw_mode().unwrap();
//         stdout.flush().unwrap();
//
//         let mut screen_updated = false;
//
//         while !self.quit {
//             if screen_updated {
//                 self.draw();
//                 screen_updated = false
//             }
//
//             match self.ui_message_receiver.try_recv() {
//                 Ok(ui_message) => {
//                     screen_updated = true;
//                     self.history.push(self.format_ui_message(ui_message))
//                 }
//                 _ => {}
//             }
//
//             if poll(Duration::from_millis(100)).expect("Cannot poll events.") {
//                 screen_updated = true;
//
//                 // Event is available:
//                 match read().unwrap() {
//                     Event::Key(event) => {
//                         match event.code {
//                             KeyCode::Char(char) => {
//                                 if char == 'c' && event.modifiers.contains(KeyModifiers::CONTROL) { // ^C -> exit
//                                     self.quit = true;
//                                 } else {
//                                     self.prompt.push(char);
//                                 }
//                             }
//                             KeyCode::Backspace => {
//                                 self.prompt.pop();
//                             }
//                             KeyCode::Enter => {
//                                 self.prompt_enter();
//                                 self.prompt = "".to_string()
//                             }
//                             _ => {}
//                         }
//                     }
//                     Event::Paste(content) => {
//                         self.prompt.push_str(&content);
//                     }
//                     Event::Resize(width, height) => {
//                         self.terminal.width = width;
//                         self.terminal.height = height;
//                     }
//                     _ => {}
//                 }
//             }
//         }
//
//
//         stdout.queue(terminal::LeaveAlternateScreen).unwrap();
//         terminal::disable_raw_mode().unwrap();
//         exit(0)
//     }
//
//     fn draw(&self) {
//         let mut stdout = stdout();
//
//         let term = &self.terminal;
//         stdout
//             .queue(terminal::Clear(terminal::ClearType::All)).unwrap()
//             .queue(cursor::MoveTo(0, 0)).unwrap();
//
//         for i in 0..term.height - 2 {
//             if self.history.len() <= i as usize { break; }
//             let data = self.history.index(self.history.len() - i as usize - 1);
//
//             stdout
//                 .queue(cursor::MoveTo(0, term.height - 3 - i)).unwrap()
//                 .queue(style::Print(data)).unwrap();
//         }
//
//         stdout
//             .queue(cursor::MoveTo(0, term.height - 2)).unwrap()
//             .queue(style::Print("─".repeat(term.width as usize))).unwrap();
//
//         stdout
//             .queue(cursor::MoveTo(1, term.height - 1)).unwrap()
//             .queue(style::Print(&self.prompt)).unwrap();
//
//         stdout.flush().unwrap()
//     }
//
//
//
//
// }