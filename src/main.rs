#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]
// This attr will remove the windows CMD if the console feature is not compiled.


extern crate windows;

use std::{ffi::OsString, time::Duration, sync::Mutex};

use lazy_static::lazy_static;
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

lazy_static!{
    static ref SHUTTLE: Mutex<Shuttle> = Mutex::new(Shuttle::new());
    static ref TRAY: Mutex<TrayItem> = {
        let mut tray = TrayItem::new("ShutleXpress-rs", "systrayicon").unwrap();
        tray.add_label("ShuttleXpress-rs").unwrap();
        Mutex::new(tray)
    };
}




use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect,
    Win32::{System::LibraryLoader::GetModuleHandleA, UI::Input::{RAWINPUTDEVICE, RIDEV_INPUTSINK, RegisterRawInputDevices, RAWINPUTHEADER, GetRawInputData, HRAWINPUT, RID_INPUT, RAWINPUT}}, Win32::UI::WindowsAndMessaging::*,
};

use std::{mem, ffi::c_void};

pub fn read_input_buffer(lparam: LPARAM) {
    let mut data_size: u32 = 0;
    unsafe {
        GetRawInputData(
            HRAWINPUT(lparam.0),
            RID_INPUT,
            None,
            &mut data_size,
            mem::size_of::<RAWINPUTHEADER>() as u32,
        );
    }

    println!("DataSize {data_size}");

    if data_size > 0 {
        let array_alloc: [u8; 1024] = [0; 1024];

        let second_data_size: u32 = unsafe {
            GetRawInputData(
                HRAWINPUT(lparam.0),
                RID_INPUT,
                Some(array_alloc.as_ptr() as *mut c_void),
                &mut data_size,
                mem::size_of::<RAWINPUTHEADER>() as u32,
            )
        };

        if second_data_size == data_size {
            let header_slice = &array_alloc[0..mem::size_of::<RAWINPUTHEADER>()];

            let header: RAWINPUTHEADER =
                unsafe { std::ptr::read(header_slice.as_ptr() as *const _) };

            println!("Header: {:?}", header);

            let data_slice = &array_alloc[0..(header.dwSize as usize)];

            let full_data: RAWINPUT = unsafe { std::ptr::read(data_slice.as_ptr() as *const _) };

            if full_data.header.dwType == 1 {
                println!("Data: {:?}", unsafe { full_data.data.keyboard });
            } else {
                println!("Data: {:?}", unsafe { full_data.data.hid });

                let slice = unsafe {
                    let ptr = full_data.data.hid.bRawData.as_ptr();
                    std::slice::from_raw_parts(
                        ptr,
                        (full_data.data.hid.dwSizeHid * full_data.data.hid.dwCount) as usize,
                    )
                };

                println!("Header Size {:?}", mem::size_of::<RAWINPUTHEADER>());

                println!("Got data : {:?}", slice);

                let mut fixed_input: [u8; 6] = [0; 6];

                for i in 0..slice.len() {
                    fixed_input[i] = slice[i];
                }

                SHUTTLE.lock().unwrap().update(fixed_input);
            }
        }
    }
}

fn register_devices(hwnd: HWND) {
    let mut rid_vec: Vec<RAWINPUTDEVICE> = Vec::new();

    rid_vec.push(RAWINPUTDEVICE {
        usUsagePage: 0x000C,
        usUsage: 0x0001,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: hwnd,
    });

    unsafe {
        if !RegisterRawInputDevices(&rid_vec, mem::size_of::<RAWINPUTDEVICE>() as u32).as_bool()
        {
            panic!("Registration of Controller Failed");
        }
    }
}

fn setup_message_window() -> HWND {
    let handle = unsafe {
        let instance = GetModuleHandleA(None).unwrap();
        debug_assert!(instance.0 != 0);
        let window_class = s!("window");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_HAND).unwrap(),
            hInstance: instance,
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        let handle = CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("Sample Window"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        );
        handle
    };

    handle
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(window, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_INPUT => {
                println!("----------------------");
                println!("WM_INPUT");
                println!("wparam {:?}", wparam);
                read_input_buffer(lparam);

                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}


fn main() {
    let hwnd = setup_message_window();
    register_devices(hwnd);

    let mut message = MSG::default();

    TRAY.lock().unwrap().add_menu_item("Quit", move || {
        std::process::exit(0);
    }).unwrap();

    unsafe {
        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&message);
        }
    }
}

