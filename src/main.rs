#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]
// This attr will remove the windows CMD if the console feature is not compiled.

use shuttle::Shuttle;
use tray_item::TrayItem;

mod shuttle;

#[derive(Debug)]
enum ButtonEventType {
    Press,
    Release,
}

enum EventType {
    Jog(i8),
    Wheel(i8, u8),
    Button(usize, ButtonEventType),
}

fn main() {
    // Add Systray-Icon to close the otherwise invisible app
    let mut tray = TrayItem::new("ShuttleXpress-rs", "systrayicon").unwrap();
    tray.add_label("ShuttleXpress-rs").unwrap();
    tray.add_menu_item("Quit", move || {
        std::process::exit(0);
    })
    .unwrap();

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
