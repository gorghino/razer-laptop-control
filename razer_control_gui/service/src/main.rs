extern crate tiny_nix_ipc;


mod rgb;
mod core;
mod effects;
use rand::Rng;
use std::os::unix::net::{UnixStream, UnixListener};
use std::{error::Error, thread, time, process};
use signal_hook::{iterator::Signals, SIGTERM, SIGINT};


// Main function for daemon
fn main() {

    // Signal handler - cleanup if we are told to exit
    let signals = Signals::new(&[SIGINT, SIGINT]).unwrap();
    thread::spawn(move || {
        for _ in signals.forever() {
            println!("Received signal, cleaning up");
            if std::fs::metadata(core::SOCKET_PATH).is_ok() {
                std::fs::remove_file(core::SOCKET_PATH).unwrap();
            }
            std::process::exit(0);
        }
    });


    // Setup driver core frameworks
    let mut drv_core = core::DriverHandler::new().expect("Error. Is kernel module loaded?");
    
    let e1 = effects::BreathEffect::new(255, 255, 255, 1000); // New breathing layer


    let mut eManager = effects::EffectManager::new(); // New effect manager
    let mut matrix : [bool; 90] = [true; 90]; // Layer 0 creation - All keys should use the effect
    eManager.push_effect(Box::new(e1), &matrix); // Push effect 0 to manager

    while(true) {
        eManager.update(&mut drv_core); // Update the effects and render!
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    if let Ok(_) = std::fs::metadata(core::SOCKET_PATH) {
        eprintln!("UNIX Socket already exists at {}. Is another daemon running?", core::SOCKET_PATH);
        std::process::exit(1);
    }
    if let Ok(listener) = UnixListener::bind(core::SOCKET_PATH) {
        let mut perms = std::fs::metadata(core::SOCKET_PATH).unwrap().permissions();
        perms.set_readonly(false);
        std::fs::set_permissions(core::SOCKET_PATH, perms);
        println!("Listening to events!");
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(|| handle_data(stream));
                }
                Err(err) => {
                    eprintln!("WARN: Broken stream, ignored");
                    break;
                }
            }
        }
    } else {
        eprintln!("Could not create Unix socket!");
        std::process::exit(1);
    }

}

fn handle_data(stream: UnixStream) {
    println!("Data!");
}