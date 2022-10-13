enum ButtonEventType {
    Press,
    Release,
}

enum EventType {
    Jog(i8),
    Wheel(i8, u8),
    Button(usize, ButtonEventType),
}

struct Shuttle {
    jog: i8,
    wheel: u8,
    buttons: [bool; 5],
}

impl Shuttle {
    pub fn new() -> Shuttle {
        Shuttle {
            jog: 0_i8,
            wheel: 0_u8,
            buttons: [false; 5],
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

    fn on_event(&mut self, event: EventType) {
        match event {
            EventType::Jog(val) => println!("Jog changed to {}.", val),
            EventType::Wheel(delta, val) => println!("Wheel changed by {delta} to {val}."),
            EventType::Button(index, btn_event) => {
                if index == 4 {
                    unsafe {
                        use winapi::um::winuser::*;
                        let mut input = INPUT {
                            type_: INPUT_KEYBOARD,
                            u: std::mem::zeroed(),
                        };
                        *input.u.ki_mut() = KEYBDINPUT {
                            wVk: VK_MEDIA_PLAY_PAUSE as u16,
                            wScan: 0,
                            dwFlags: match btn_event {
                                ButtonEventType::Release => KEYEVENTF_KEYUP,
                                _ => 0,
                            },
                            time: 0,
                            dwExtraInfo: 1,
                        };
                        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
                    }
                }
            }
        }
    }
}

fn main() {
    let api = hidapi::HidApi::new().unwrap();
    // Print out information about all connected devices
    for device in api.device_list() {
        println!("{:#?}", device);
        println!("{:?}", device.product_string());
    }

    let connection = api.open(2867, 32);
    let device = match connection {
        Ok(device) => device,
        Err(e) => panic!("Got an error {:?}", e),
    };

    let mut shuttle = Shuttle::new();

    loop {
        let mut buffer: [u8; 1024] = [0; 1024];

        let len = match device.read(&mut buffer) {
            Ok(len) => len,
            Err(e) => {
                dbg!("Error {}", e);
                0
            }
        };

        if len == 5 {
            shuttle.update(&buffer[0..5]);
        }
    }
}
