use std::io::Read;

use serde::Deserialize;

use crate::{ButtonEventType, EventType};

#[derive(Deserialize, Clone, Copy)]
enum EventReaction {
    None,
    KeyStroke(u16),
}

#[derive(Deserialize)]
struct EventConfig {
    buttons: [EventReaction; 5],
    jog_changed: EventReaction,
    wheel_up: EventReaction,
    wheel_down: EventReaction,
}

pub struct Shuttle {
    jog: i8,
    wheel: u8,
    buttons: [bool; 5],
    config: EventConfig,
}

impl Shuttle {
    pub fn new() -> Shuttle {
        let file = std::fs::File::open("config.ron").expect("The config file is needed.");
        let config: EventConfig =
            ron::de::from_reader(file).expect("The config file is wrongly formatted.");

        Shuttle {
            jog: 0_i8,
            wheel: 0_u8,
            buttons: [false; 5],
            config,
        }
    }

    pub fn update(&mut self, buffer: &[u8]) {
        let new_buttons = [
            (buffer[3] & 0b00010000) > 0,
            (buffer[3] & 0b00100000) > 0,
            (buffer[3] & 0b01000000) > 0,
            (buffer[3] & 0b10000000) > 0,
            (buffer[4] & 0b00000001) > 0,
        ];

        for index in 0..5 {
            if self.buttons[index] != new_buttons[index] {
                self.on_event(EventType::Button(
                    index,
                    if self.buttons[index] {
                        ButtonEventType::Release
                    } else {
                        ButtonEventType::Press
                    },
                ));
            }
        }

        self.buttons[0] = new_buttons[0];
        self.buttons[1] = new_buttons[1];
        self.buttons[2] = new_buttons[2];
        self.buttons[3] = new_buttons[3];
        self.buttons[4] = new_buttons[4];

        let new_jog = buffer[0] as i8;

        if self.jog != new_jog {
            self.on_event(EventType::Jog(new_jog));
        }
        self.jog = new_jog;

        let new_wheel = buffer[1];
        if self.wheel != new_wheel {
            self.on_event(EventType::Wheel(
                ((new_wheel as i16) - (self.wheel as i16)) as i8,
                new_wheel,
            ));
        }

        self.wheel = new_wheel;
    }

    fn handle_event(event: EventReaction, event_type: ButtonEventType) {
        match event {
            EventReaction::None => (),
            EventReaction::KeyStroke(key) => unsafe {
                println!("Key: {key}, {event_type:?}");

                use winapi::um::winuser::*;
                let mut input = INPUT {
                    type_: INPUT_KEYBOARD,
                    u: std::mem::zeroed(),
                };

                *input.u.ki_mut() = KEYBDINPUT {
                    wVk: key,
                    wScan: 0,
                    dwFlags: match event_type {
                        ButtonEventType::Release => KEYEVENTF_KEYUP,
                        _ => 0,
                    },
                    time: 0,
                    dwExtraInfo: 1,
                };
                SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
            },
        }
    }

    fn on_event(&mut self, event: EventType) {
        match event {
            EventType::Jog(val) => println!("Jog changed to {}.", val),
            EventType::Wheel(delta, val) => {
                if delta > 0 {
                    Shuttle::handle_event(self.config.wheel_up, ButtonEventType::Press);
                } else {
                    Shuttle::handle_event(self.config.wheel_down, ButtonEventType::Press);
                }
            }
            EventType::Button(index, btn_event) => {
                Shuttle::handle_event(self.config.buttons[index], btn_event);
            }
        }
    }
}