/*
This program implements a WebSocket-based chat client with file upload functionality.
It connects to a WebSocket server at the specified address, `ws://127.0.0.1:1111/app`, and facilitates communication between the client and the server using multiple threads.
The client runs in a loop, where it listens for user input and sends it asynchronously via a channel to a WebSocket server.
It uses separate threads for reading messages from the server and sending messages to the server to ensure non-blocking behavior.
*/

use std::io::stdin; //to read user input from terminal
use std::path::PathBuf;
use std::thread; //to create threads //to buffer file

use std::sync::mpsc::channel; // message passing channel, for communication between threads
use std::sync::mpsc::{Receiver, Sender}; //reciev and send messages into the channel

use std::net::TcpStream; //to interact with tcp
use websocket::client::ClientBuilder; // used to create a WebSocket client
use websocket::sync::{Reader, Writer}; // used to read and write for websocket
use websocket::{Message, OwnedMessage}; //Regular WebSocket message and ownwed message when you need to take ownerhsip

mod file_handler; //contains functions to recieve and send a file

const CONNECTION: &'static str = "ws://127.0.0.1:1111/app"; //string slice that has lifetime of entire program

fn print_commands() {
    println!("Welcome to the Chat Client!");
    println!("1. Register: `reg -u userName -p password`");
    println!("2. Login: `login -u userName -p password`");
    println!("3. Send message to one user: `text -u otherUser message`");
    println!("4. Send message to multiple users: `textMultiple -u user1 user2 -t message`");
    println!("5. Start chat: `startchat chatName`");
    println!("6. Join chat: `joinchat chatName`");
    println!("7. Message chat: `message hey chat`");
    println!("8. Send file: `file -u otherUser filePath`");

    println!("Type a command to get started.\n");
}

//Loop to read user input from channel and send it to the websocket which is the server
//Parameters are the channel we recieve from and the socket we sent to
fn send_loop(channel_reciever: Receiver<OwnedMessage>, mut socket_sender: Writer<TcpStream>) {
    loop {
        let message = match channel_reciever.recv() {
            //receive message from the channel
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error recieving from channel in send loop: {:?}", e);
                return;
            }
        };
        match message {
            //match the message incase its a close message
            OwnedMessage::Close(_) => {
                let _ = socket_sender.send_message(&message); //send close message
                return; // return since we have sent our last message
            }
            _ => (),
        }
        match socket_sender.send_message(&message) {
            //otherwise just send the message to the socket
            Ok(()) => (),
            Err(e) => {
                eprintln!("Error sending message to web socket in send loop: {:?}", e); //print error messsage
                let _ = socket_sender.send_message(&Message::close()); // send a close message on error
                return;
            }
        }
    } //end loop
}

// The recieve loop recieves messages from the sockets and sends into the channel incase of an error
// Parameters are the socket that we use to recieve message from the server and the channel that we send close messsges to
fn receive_loop(mut socket_receiver: Reader<TcpStream>, channel_sender: Sender<OwnedMessage>) {
    let mut file_buffer: Option<(PathBuf, Vec<u8>)> = None;

    for message in socket_receiver.incoming_messages() {
        //for all messages in the web socket
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error recieving message from web socket {:?}", e);
                let _ = channel_sender.send(OwnedMessage::Close(None)); //if we encounter an error we send a close messsage to the channel
                return; // return from the loop
            }
        };

        match message {
            //Match the type of message it either close, binary data, text data, or other
            OwnedMessage::Close(_) => {
                let _ = channel_sender.send(OwnedMessage::Close(None));
                return;
            }
            OwnedMessage::Binary(data) => {
                if let Err(err) = file_handler::handle_binary_message(data, &mut file_buffer) {
                    //binary messages can be file chuncks so we need to handle this
                    eprintln!("Error handling binary message: {}", err);
                }
            }
            OwnedMessage::Text(text) => {
                println!("Message received: {}", text);
            }
            _ => {
                println!("Uknown message type received: {:?}", message);
            }
        } //end match
    } //end for loop
} //end recieve loop

// Reads input from the user and sends it into the channel that is read from asynchronously
// Parameter is the channel_sender, since we want to send into the channel, so send loop can read from the channel and send it to web socket
fn input_to_channel(channel_sender: Sender<OwnedMessage>) {
    loop {
        let mut input = String::new(); //string for user input
        stdin().read_line(&mut input).unwrap(); //get user input
        let message = input.trim(); //get rid of white space around input

        let message = if message.starts_with("file -u") {
            file_handler::handle_file_upload(channel_sender.clone(), message); //this is a special case when the user requests sending a file
            continue;
        } else if message == "quit" {
            channel_sender.send(OwnedMessage::Close(None)).unwrap(); //this is special case when user sends quit
            break;
        } else {
            OwnedMessage::Text(message.to_string()) //this is the regular case
        };

        match channel_sender.send(message) {
            //send the message to the channel
            Ok(()) => (),
            Err(e) => {
                eprintln!("Error sending user input into channel: {:?}", e);
                break;
            }
        }
    }
}

fn main() {
    let client_result = ClientBuilder::new(CONNECTION)
        .unwrap()
        .add_protocol("rust-websocket")
        .connect_insecure();

    match client_result {
        Ok(client) => {
            print_commands();

            // Split the WebSocket into receiver and sender
            let (socket_receiver, socket_sender) = client.split().unwrap();

            // Create a channel for communication
            let (channel_sender, channel_receiver) = channel();

            // Clone the sender to use it in multiple places
            let channel_sender_copy = channel_sender.clone();

            // Thread to send messages asynchronously
            let send_thread = thread::spawn(move || {
                send_loop(channel_receiver, socket_sender);
            });

            // Thread to receive messages asynchronously
            let receive_thread = thread::spawn(move || {
                receive_loop(socket_receiver, channel_sender_copy);
            });

            // Handle user input from stdin and send to channel
            input_to_channel(channel_sender);

            // Wait for child threads to exit
            println!("Waiting for child threads to exit");
            let _ = send_thread.join(); // Wait for completion of the send thread
            let _ = receive_thread.join(); // Wait for completion of the receive thread
            println!("Exited");
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e); // Handle connection error
        }
    }
}
