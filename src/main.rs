use std::{env, thread};
use std::path::PathBuf;
use std::sync::mpsc::channel;

use clap::Parser;
use home::home_dir;
use serde::{Deserialize, Serialize};

use crate::mqtt_controller::MqttController;
use crate::ui_controller::UIController;
use crate::utils::{ClientSettings, UIAction, UIMessage};

mod packets;
mod ui_controller;
mod mqtt_controller;
mod utils;
mod data_client;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Topic to chat (default: pubtrust-chat/general)
    topic: Option<String>,

    /// Path to profile (default: ~/.pubtrust-profile)
    #[arg(short, long)]
    profile: Option<PathBuf>,

    /// Broker host
    #[arg(long, default_value = "broker.emqx.io")]
    host: String,

    /// Broker host
    #[arg(long, default_value_t = 1883)]
    port: u16,
}

fn main() {
    let args = Args::parse();
    let profile = if args.profile.is_some() {
        let mut profile = env::current_dir().expect("Couldn't get CWD");
        profile.push(args.profile.unwrap());

        profile
    } else {
        let mut profile = home_dir().unwrap_or(env::current_dir().expect("Couldn't get CWD"));
        profile.push(".pubtrust-profile");

        profile
    };

    let client_settings = ClientSettings {
        host: args.host,
        port: args.port,
        topic: args.topic.unwrap_or("pubtrust-chat/general".to_string()),
        profile,
    };

    let (ui_message_sender, ui_message_receiver) = channel::<UIMessage>();
    let (ui_action_sender, ui_action_receiver) = channel::<UIAction>();

    thread::spawn(move || {
        let mut mqtt_controller = MqttController::new(
            client_settings.clone(),
            ui_message_sender,
            ui_action_receiver,
        );

        mqtt_controller.start();
    });

    thread::spawn(move || {
        let mut ui_controller = UIController::new(
            ui_message_receiver,
            ui_action_sender,
        );

        ui_controller.start();
    });


    loop {}
}
