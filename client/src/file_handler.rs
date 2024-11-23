/*
These functions handle the file upload and download process for clients.
The use a buffer to read in and recieve the file.
*/

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender; //reciev and send messages into the channel
use websocket::OwnedMessage; //Regular WebSocket message and ownwed message when you need to take ownerhsip

//This functions handles a client attempting to send a file to another client.
//It does preliminary checks and then buffers it too the server.
pub fn handle_file_upload(channel_sender: Sender<OwnedMessage>, message: &str) {
    let parts: Vec<&str> = message.split_whitespace().collect();
    if parts.len() < 4 {
        eprintln!("Invalid message format. Expected: `file -u user file_path`.");
        return;
    }

    let user = parts[2]; // Extract user
    let file_path = parts[3]; // Extract file path

    let path = Path::new(file_path);
    if !path.exists() {
        eprintln!("The file specified does not exist: {}", file_path);
        return;
    }

    // Attempt to open the file
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file '{}': {:?}", file_path, e);
            return;
        }
    };

    // Extract the file name from the path
    let file_name = match path.file_name().and_then(|name| name.to_str()) {
        Some(name) => name.to_string(),
        None => {
            eprintln!("Invalid file name for path: {}", file_path);
            return;
        }
    };

    // Send a message to signify the beginning of the file transfer
    let start_msg = OwnedMessage::Text(format!("file -u {} {}", user, file_name));
    if let Err(e) = channel_sender.send(start_msg) {
        eprintln!("Error sending start message: {:?}", e);
        return;
    }

    // Read the file in chunks and send each chunk
    let mut buffer = vec![0u8; 1024];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break, // End of file reached
            Ok(n) => {
                let chunk = &buffer[..n];
                let message = OwnedMessage::Binary(chunk.to_vec());
                if let Err(e) = channel_sender.send(message) {
                    eprintln!("Error sending file chunk: {:?}", e);
                    return;
                }
            }
            Err(e) => {
                eprintln!("Error reading file '{}': {:?}", file_path, e);
                return;
            }
        }
    }

    // Notify the server that the file transfer is complete by sending EOF
    let end_msg = OwnedMessage::Text(format!("EOF: {}", file_name));
    if let Err(e) = channel_sender.send(end_msg) {
        eprintln!("Error sending EOF message: {:?}", e);
    }
}

//This functions handles binary messages.
//It  check if its a file message if it its is starts recicing chunck via the buffer.
//Otherwise it just converts the bytes to text.
pub fn handle_binary_message(
    //this included recieving a file
    data: Vec<u8>,
    file_buffer: &mut Option<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    let content = String::from_utf8_lossy(&data); // Convert the byte into string

    if content.starts_with("Filename:") {
        // Parse file metadata
        let parts: Vec<&str> = content.split_whitespace().collect();
        let filename = parts[1].to_string();
        println!("Starting file reception: {}", filename);
        *file_buffer = Some((PathBuf::from(filename), Vec::new()));
    } else if let Some((_, ref mut buffer)) = file_buffer {
        if data == b"EOF" {
            // Check for EOF marker
            println!("End of file received. Saving file...");
            if let Some((file_path, buffer)) = file_buffer.take() {
                if save_file(file_path, &buffer).is_ok() {
                    println!("File received and saved successfully.");
                } else {
                    return Err("Failed to save received file.".to_string());
                }
            }
        } else {
            buffer.extend_from_slice(&data);
            println!("Received file chunk: {} bytes", data.len());
        }
    } else {
        println!(
            "Message received {}",
            match String::from_utf8(data) {
                Ok(text) => text,
                Err(_) => "<Invalid UTF-8>".to_string(),
            }
        );
    }
    Ok(())
}

/// Saves the received file to disk.
pub fn save_file(file_path: PathBuf, data: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(data)?;
    Ok(())
}
