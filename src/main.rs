use midir::{MidiInput, MidiOutput};
use midir::Ignore;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

// Define the MIDI key mapping for steno keyboard
const KEY_MAPPING: [(u8, u8); 2] = [
    (48, 60), // Mapping C3 to C4
    (50, 62), // Mapping D3 to D4
];

fn main() {
    let midi_in = MidiInput::new("midir test input").unwrap();
    let midi_out = MidiOutput::new("midir test output").unwrap();

    let in_ports = midi_in.ports();
    let out_ports = midi_out.ports();

    if in_ports.is_empty() {
        println!("No input port found");
        return;
    }

    if out_ports.is_empty() {
        println!("No output port found");
        return;
    }

    let in_port = &in_ports[0];
    let out_port = &out_ports[0];

    println!("Opening input port: {}", midi_in.port_name(in_port).unwrap());
    let (tx, rx) = channel();
    let in_conn = midi_in.connect(in_port, "midir-test-in", move |_, message, _| {
        // Process incoming MIDI messages
        let processed_message = remap_midi_key(message);
        // Send the processed message to the main thread for further handling
        if let Err(_) = tx.send(processed_message) {
            eprintln!("Error sending MIDI message to main thread");
        }
    }, Ignore::All).unwrap();

    println!("Opening output port: {}", midi_out.port_name(out_port).unwrap());
    let mut out_conn = midi_out.connect(out_port, "midir-test-out").unwrap();

    out_conn.send(&[0x90, 60, 0x64]).unwrap();
    out_conn.send(&[0x90, 60, 0x00]).unwrap();

    // std::thread::sleep(std::time::Duration::from_secs(1));
    out_conn.send(&[0x80, 20, 0x00]).unwrap();


    println!("Listening for MIDI messages. Press Ctrl+C to exit.");
    // Spawn a thread to handle incoming MIDI messages
    thread::spawn(move || {
        while let Ok(message) = rx.recv() {
            // Handle the processed MIDI message (e.g., send to custom board)
            println!("Processed MIDI message: {:?}", message);
            // Add your logic to send the processed MIDI message to the custom board
        }
    });

    // Keep the main thread alive to continue listening for MIDI messages
    loop {
        thread::sleep(std::time::Duration::from_millis(100));
    }

    // in_conn.close();
    // out_conn.close();
}

// Function to remap MIDI key according to steno keyboard mapping
fn remap_midi_key(message: &[u8]) -> Vec<u8> {
    if message.len() == 3 {
        let (status, data1, data2) = (message[0], message[1], message[2]);
        for &(original_key, mapped_key) in &KEY_MAPPING {
            if data1 == original_key {
                return vec![status, mapped_key, data2];
            }
        }
    }
    message.to_vec() // Return original message if no mapping is found
}
