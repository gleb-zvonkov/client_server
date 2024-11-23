/*
This is the command handler module.
It implments the logic for handling user registration, login, messaging, group chats, and file upload.
*/

use crate::user_manager::{User, UserManager};
use futures::channel::mpsc::UnboundedSender; //used unbounded channel to communicate with existing clients
use futures::lock::Mutex; //used to lock the the User Manager
use tokio::io;

use rocket::futures::{SinkExt, StreamExt};
use std::{collections::HashSet, sync::Arc}; //to store participants in groupchat
use ws::stream::DuplexStream; //socket stream used to send message from the server //use the User and UserManager we created

//********************************************************************************
//Below is an enum for all valid commands that our server handles
//For each command we have a parse function and a handle function.
pub enum Command {
    Register { name: String, password: String }, //register a new user
    Login { name: String, password: String },    //login user
    Text { name: String, content: String },      //text one client
    TextMultiple { names: Vec<String>, content: String }, //text multiple clients
    StartChat { name: String },                  // Start a new chat
    JoinChat { name: String },                   // Join an existing chat
    MessageChat { content: String },             // Message the chat
    QuitChat { name: String },                   //Quit the chat
    File { name: String, file_path: String },
    Quit, //Client quits altogether
}

//*****************************************************************************
//First the parse functions

//parse the registration to reutn Command:Register with name and password
//reg -u user4 -p secret
fn parse_reg(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect(); //split input line into a vector of string slices
    if args.len() > 4 && args[0] == "reg" && args[1] == "-u" && args[3] == "-p" {
        return Some(Command::Register {
            name: args[2].to_string(),
            password: args[4].to_string(),
        });
    }
    None
}

//parse quit to to return Command:Quit
//quit
fn parse_quit(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect();
    if args.len() > 0 && args[0] == "quit" {
        return Some(Command::Quit);
    }
    None
}

//parse the login and return Command:Login with name and password
//login -u user123 -p secret
fn parse_login(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect();
    if args.len() > 4 && args[0] == "login" && args[1] == "-u" && args[3] == "-p" {
        return Some(Command::Login {
            name: args[2].to_string(),
            password: args[4].to_string(),
        });
    }
    None
}

//parse text and return Command:Text with username and content
//text -u user123 messagehere
fn parse_text(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect();
    if args.len() > 3 && args[0] == "text" && args[1] == "-u" {
        let content = args[3..].join(" "); //get all the text after the first 3 arguments
        return Some(Command::Text {
            name: args[2].to_string(),
            content,
        });
    }
    None
}

//Parse text multiple and return Command:TextMultiple with usernames and content
//textMultiple -u user2 user3 -t hello to two different users
fn parse_text_multiple(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect(); //collect all the arguments
    if args.len() > 3 && args[0] == "textMultiple" && args[1] == "-u" {
        // Check if the command starts with "textMultiple" and "-u"
        if let Some(t_index) = args.iter().position(|&x| x == "-t") {
            // Find the position of "-t"
            let names: Vec<String> = args[2..t_index] // The usernames are the arguments between "-u" and "-t"
                .iter()
                .map(|&name| name.to_string())
                .collect(); //collect the usernames
            let content = args[t_index + 1..].join(" "); // The message content comes after "-t" and can be multiple words
            return Some(Command::TextMultiple { names, content });
        }
    }
    None
}

//Parse start chat and return Command:StartChat with chat name
//starchat chat_name
fn parse_start_chat(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect();
    if args.len() > 1 && args[0] == "startchat" {
        return Some(Command::StartChat {
            name: args[1].to_string(),
        });
    }
    None
}

//Parse join chat and return Command:JoinChat with chat name
//joinchat chat_name
fn parse_join_chat(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect();
    if args.len() > 1 && args[0] == "joinchat" {
        return Some(Command::JoinChat {
            name: args[1].to_string(),
        });
    }
    None
}

//Parse message chat and retunr Command: MessageChat with content of message
//message hey guys
fn parse_message_chat(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect();
    if args.len() > 1 && args[0] == "message" {
        let content = args[1..].join(" "); //join everything after message
        return Some(Command::MessageChat { content });
    }
    None
}

//Parse quit chat and return Command:QuitChat with the name of the chat
//quitchat chat_name
fn parse_quit_chat(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect();
    if args.len() > 1 && args[0] == "quitchat" {
        return Some(Command::QuitChat {
            name: args[1].to_string(),
        });
    }
    None
}

//parse text and return Command:Text with username and content
//text -u user123 messagehere
//file -u user2 /Users/glebzvonkov/Downloads/rust/Project1724/testsend.txt
fn parse_file_command(line: &str) -> Option<Command> {
    let args: Vec<&str> = line.split_ascii_whitespace().collect();
    if args.len() > 3 && args[0] == "file" && args[1] == "-u" {
        let file_path = args[3..].join(" "); //get all the text after the first 3 arguments
                                             //let file_name = Path::new(&file_path).file_name().unwrap().to_str().unwrap();
        return Some(Command::File {
            name: args[2].to_string(),
            file_path,
        });
    }
    None
}

//*****************************************************************************
//Now the functions to handle different commands

//Handle a registration request
//Create a new user and attempt to add to user manager, respond accordings
async fn handle_register(
    user_manager: &mut UserManager,
    name: String,
    password: String,
    stream: &mut DuplexStream,
) {
    let user = User {
        name,
        password,
        current_chat: None,
    }; //create the new user

    let response = if user_manager.add(&user).is_err() {
        "Registration error, it is possible the username is already in use"
    } else {
        "Registration successful"
    }; //response according to if succesfully added user

    send_response(stream, response).await; //send response back to client
}

//Handle a login request
//Attempt to authenticate the user using bcrypt
//If successful set the current user and make the user online
//respond accordingly
async fn handle_login(
    user_manager: &mut UserManager,
    name: String,
    password: String,
    current_user: &mut Option<String>,
    sender: futures::channel::mpsc::UnboundedSender<Vec<u8>>,
    stream: &mut DuplexStream,
) {
    // Check if the user exists
    if user_manager.users.contains_key(&name) {
        match user_manager.authenticate(&name, &password) {
            Ok(true) => {
                *current_user = Some(name.clone()); //set the current user
                user_manager.onlines.insert(name, sender); //set the user online
                send_response(stream, "Login Successful").await; //send login success
            }
            Ok(false) => {
                send_response(stream, "Wrong Password").await; // Incorrect password
            }
            Err(e) => {
                send_response(stream, &format!("Authentication error: {}", e)).await;
                //Authentication Error
            }
        }
    } else {
        send_response(stream, "User does not exist").await; // User does not exist
    }
}

//Handle a text message request
//First check if we are logged in
//Next check if the user where sending to is online, then send him the message
//Respond accordingly
async fn handle_text_message(
    user_manager: &mut UserManager,
    current_user: Option<String>,
    name: String,
    content: String,
    stream: &mut DuplexStream, //reciving and sending using this stream
) {
    let cur = match current_user {
        Some(ref user) => user,
        None => {
            send_response(stream, "Login first").await; //if client hasent logged in dont want to continue
            return;
        }
    };

    if let Some(channel) = user_manager.onlines.get_mut(&name) {
        //check if the user where attempting to send to is online
        let msg = format!("from {}: {}", cur, content.as_str()); //format the messsge
        let _ = channel.send(msg.into_bytes()).await; //send the message to the other client
        send_response(stream, "Successfully Sent").await;
    } else {
        send_response(stream, "User is not online").await;
    }
}

//Handle a text message to multiple users
//First check if we are logged in
//Go throught all the names and attempt to send to each one
//Respond accordingly
async fn handle_text_multiple(
    user_manager: &mut UserManager,
    current_user: Option<String>,
    names: Vec<String>,
    content: String,
    stream: &mut DuplexStream,
) {
    let cur = match current_user {
        Some(ref user) => user,
        None => {
            send_response(stream, "Login first").await; //if client hasent logged in dont want to continue
            return;
        }
    };
    for name in names {
        //go throught all the usernames
        if let Some(s) = user_manager.onlines.get_mut(name.as_str()) {
            //check if each user is online
            let msg = format!("from {}: {}", cur, content);
            let _ = s.send(msg.into_bytes()).await; // Send message to each recipient
        } else {
            send_response(stream, &format!("User {} is not online", name)).await;
            //send a message back for each user that is not online
        }
    }
    send_response(stream, "Successfully Sent").await;
}

//Handle a start chat request
//Check if logged in
//Create the chat and insert into user mamanger and set current chat in user manager
async fn handle_start_chat(
    user_manager: &mut UserManager,
    current_user: Option<String>,
    name: String,
    stream: &mut DuplexStream,
) {
    let cur = match current_user {
        Some(ref user) => user,
        None => {
            send_response(stream, "Login first").await; //if not logged in dont want to continue
            return;
        }
    };
    let mut participants = HashSet::new(); //create a new hashet to store the users in a groupchats
    participants.insert(cur.clone()); //insert the current user into the hashet
    user_manager.chats.insert(name.clone(), participants); //add groupchat name and hashet to user manager
    let message = match user_manager.set_current_chat(&cur, Some(name.clone())) {
        //set the current chat variable for the user
        Ok(_) => format!("Chat '{}' started successfully", name),
        Err(_) => format!("Failed to start chat '{}'", name),
    };
    send_response(stream, &message).await; //send response to client
}

//Handle a join chat request
// Check if user logged in
// Check if chat exists
// Insert into the hashset
// Attempt setting the current chat
// Notify chat participants
async fn handle_join_chat(
    user_manager: &mut UserManager,
    cur: Option<String>,
    name: String,
    stream: &mut DuplexStream,
) {
    let joining_user = match cur {
        Some(ref user) => user,
        None => {
            send_response(stream, "Login first").await;
            return;
        }
    };

    let chat = match user_manager.chats.get_mut(&name) {
        Some(chat) => chat,
        None => {
            send_response(stream, "Chat not found").await;
            return;
        }
    };

    chat.insert(joining_user.clone()); //insert into hashset
    let chat_users = chat.clone();

    let message = match user_manager.set_current_chat(&joining_user, Some(name.clone())) {
        //set the current chat in the user structure
        Ok(_) => format!("Joined chat '{}'", name),
        Err(_) => format!("Failed to join chat '{}'", name),
    };
    send_response(stream, &message).await; //send response to client

    let message = format!("User '{}' has joined the chat '{}'", joining_user, name);
    notify_chat_users(user_manager, &joining_user, chat_users, &message).await; //notify other chat users that the user joined
}

//Handle messagess request
//Check if logged in
//Check if there is a current chat set
//Get all chat users
//send to all chat user
async fn handle_message_chat(
    user_manager: &mut UserManager,
    cur: Option<String>,
    content: String,
    stream: &mut DuplexStream,
) {
    let sender_name = match cur {
        Some(name) => name,
        None => {
            send_response(stream, "Login first").await;
            return;
        }
    };

    let chat_name = match user_manager.get_current_chat(&sender_name) {
        Some(name) => name,
        None => {
            send_response(stream, "Join or start a chat first").await;
            return;
        }
    };

    let chat_users = match user_manager.chats.get(&chat_name) {
        //get all the users in the chat
        Some(users) => users.clone(), // Clone the chat users to avoid borrowing issues
        None => {
            send_response(stream, "Chat not found").await;
            return;
        }
    };

    let message = format!("{}: {}", sender_name, content); //format the message
    notify_chat_users(user_manager, &sender_name, chat_users, &message).await; //send to all user in the chat
}

//Handle a quit chat request
//check if logged in
//get the hashset of chat users
//attempt removing user from chat
//send response
//notify all the users in the chat
async fn handle_quit_chat(
    user_manager: &mut UserManager,
    cur: Option<String>,
    name: String,
    stream: &mut DuplexStream,
) {
    let quitting_user = match cur {
        Some(user) => user,
        None => {
            send_response(stream, "Login first").await;
            return;
        }
    };

    let chat = match user_manager.chats.get_mut(&name) {
        Some(chat) => chat,
        None => {
            send_response(stream, "Chat not found").await;
            return;
        }
    };

    if !chat.remove(&quitting_user) {
        //remove the quitting user
        send_response(stream, "You are not in this chat").await;
        return;
    }

    send_response(stream, &format!("You have left the chat '{}'", name)).await;

    let message = format!("User '{}' has left the chat", quitting_user); //format quit notification
    let chat_users: Vec<_> = chat.iter().collect(); // Collect references to avoid borrowing issues
    for user in chat_users {
        if let Some(mut sender) = user_manager.onlines.get(user) {
            let _ = sender.send(message.clone().into_bytes()).await;
        }
    }
}

//This handle the file upload
//It sends a message signifying a file transfer will start.
//Uses the recive and send function.
async fn handle_file_message(
    user_manager: &mut UserManager,
    current_user: Option<String>,
    filename: String,
    file_path: String,         // The local path of the file to send
    stream: &mut DuplexStream, // Stream from the client where we will receive chunks
) {
    let _ = match current_user {
        Some(name) => name,
        None => {
            send_response(stream, "Login first").await;
            return;
        }
    };
    // Check if user is online and get their channel
    if let Some(channel) = user_manager.onlines.get_mut(&filename) {
        // Start receiving chunks from the client and forward them to the channel

        let _ = channel
            .send(format!("Filename: {}", file_path).into_bytes())
            .await; //make function that can handle this

        if let Err(err) = receive_and_forward_file(stream, channel).await {
            send_response(stream, &format!("Error receiving file: {}", err)).await;
        } else {
            send_response(stream, "File sent successfully").await;
        }

        let _ = channel.send("EOF".as_bytes().to_vec()).await; //sent to the reciever to indicate end of file
    } else {
        send_response(stream, "User is not online").await;
    }
}

//***************************************************************
//Below are helper functions for the above functions.

//Recvie and send chuncks of the file
async fn receive_and_forward_file(
    stream: &mut DuplexStream,
    channel: &mut UnboundedSender<Vec<u8>>, // Where we send the chunks to clients
) -> io::Result<()> {
    while let Some(message) = stream.next().await {
        match message {
            Ok(ws::Message::Binary(chunk)) => {
                // Received a chunk of the file
                if let Err(e) = channel.send(chunk).await {
                    eprintln!("Error sending chunk to channel: {}", e);
                    return Err(io::Error::new(io::ErrorKind::Other, "Failed to send chunk"));
                }
            }
            _ => {
                println!("Done Sending File");
                break;
            }
        }
    }
    Ok(())
}

//Send a message to the webscoket
//Used to respond back to the client that sent a command
async fn send_response(stream: &mut DuplexStream, message: &str) {
    if let Err(e) = stream.send(ws::Message::Text(message.to_string())).await {
        eprintln!("Failed to send message: {:?}", e);
    }
}

//Used in chat functions to send to all users in chat but the current sender
async fn notify_chat_users(
    user_manager: &mut UserManager,
    sender_name: &str,
    chat_users: HashSet<String>,
    message: &str,
) {
    for user in chat_users.iter() {
        if user != sender_name {
            if let Some(channel) = user_manager.onlines.get_mut(user) {
                let _ = channel.send(message.to_string().into_bytes()).await;
            }
        }
    }
}

//**************************************************************************
//The two functions below put together all the functions above

//Evaluates each parser in order.
//If a command matches one of the parsers returns Command variant otherwise none
fn parse_command(cmd: &str) -> Option<Command> {
    parse_reg(cmd)
        .or(parse_login(cmd))
        .or(parse_text(cmd))
        .or(parse_text_multiple(cmd))
        .or(parse_start_chat(cmd))
        .or(parse_join_chat(cmd))
        .or(parse_message_chat(cmd))
        .or(parse_quit_chat(cmd))
        .or(parse_file_command(cmd))
        .or(parse_quit(cmd))
}

//Handle all possible commands
pub async fn handle_command(
    user_manager: &Arc<Mutex<UserManager>>,
    current_user: &mut Option<String>,
    stream: &mut DuplexStream,
    sender: UnboundedSender<Vec<u8>>,
    message: &str,
) {
    println!("{}", message); // Log the incoming message for debugging

    if let Some(cmd) = parse_command(&message) {
        let mut user_manager = user_manager.lock().await; // Lock user manager for thread safety
        match cmd {
            Command::Register { name, password } => {
                handle_register(&mut user_manager, name, password, stream).await
            }
            Command::Login { name, password } => {
                handle_login(
                    &mut user_manager,
                    name,
                    password,
                    current_user,
                    sender.clone(),
                    stream,
                )
                .await
            }
            Command::Text { name, content } => {
                handle_text_message(
                    &mut user_manager,
                    current_user.clone(),
                    name,
                    content,
                    stream,
                )
                .await
            }
            Command::TextMultiple { names, content } => {
                handle_text_multiple(
                    &mut user_manager,
                    current_user.clone(),
                    names,
                    content,
                    stream,
                )
                .await
            }
            Command::StartChat { name } => {
                handle_start_chat(&mut user_manager, current_user.clone(), name, stream).await
            }
            Command::JoinChat { name } => {
                handle_join_chat(&mut user_manager, current_user.clone(), name, stream).await
            }
            Command::MessageChat { content } => {
                handle_message_chat(&mut user_manager, current_user.clone(), content, stream).await
            }
            Command::QuitChat { name } => {
                handle_quit_chat(&mut user_manager, current_user.clone(), name, stream).await
            }
            Command::File { name, file_path } => {
                handle_file_message(
                    &mut user_manager,
                    current_user.clone(),
                    name,
                    file_path,
                    stream,
                )
                .await
            }
            Command::Quit => {
                return; // Simply exit the loop when the Quit command is received
            }
        }
    } else {
        send_response(stream, "Unknown command").await; // Send unknown command response
    }
}
