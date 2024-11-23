/*
This the main of our server function.
It defines a Rocket web application that uses WebSockets to handle real-time communication.
Integrates an unbounded channel for asynchronous message passing between clients and the server.
*/

use futures::{channel::mpsc::unbounded, lock::Mutex, select}; //unbounded channel for sending/recieving messages, lock for locking user manager, select for selecting between socket and channel
use rocket::{
    futures::{SinkExt, StreamExt}, //SinkExt used for  stream.send(), StreamExt used for methods like stream.next()
    get,    //get macro used to define a route handler in Rocket for HTTP GET requests.
    launch, //used to mark the function that will run the Rocket application
    routes, //used to define a collection of route handlers
    State,  //used to pass application wide State, UserManager in this situation
};
use std::sync::Arc; //Atomic refrence counting so the user manager is shared safely

mod command_handler;
use command_handler::handle_command; //get only the handle command function which allows registration, login, sendining text, groupchat

mod user_manager;
use user_manager::UserManager; //get the UserManager crate

type Context = Arc<Mutex<UserManager>>; //thread safe refrence point counter

//This functions performs cleanup by removing a user from online state
async fn cleanup(user_manager: Arc<Mutex<UserManager>>, current_user: Option<String>) {
    if let Some(name) = current_user {
        let mut user_manager = user_manager.lock().await; //lock the user manager so other threads dont mess with it
        user_manager.onlines.remove(name.as_str()); //remove online user
        println!("User {} has quit.", name); //print the user has quit for debugging purposes
    }
}

#[get("/app")] //respond to HTTP GET request on /app endpoints
               //Parameters are State<Context> which is shared across requests and web socket
               //returns a web socket channel abstraction
fn app_channel(user_manager: &State<Context>, ws: ws::WebSocket) -> ws::Channel<'static> {
    let user_manager = user_manager.inner().clone(); //Cloning user manager for use within WebSocket handler

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut current_user: Option<String> = None; // variable to keep track of current logged in user
            let (sender, mut receiver) = unbounded::<Vec<u8>>(); // unbounded channel for sending/recieving messsges

            loop {
                //continously evalutate the select
                select! {  //which ever task completes first execute its corresponding block two taks possible  1.read from the socket stream  2.recieve from the channel in order to send to socket stream

                    message = stream.next() => { //1. get the next incoming WebSocket message
                        if message.is_none() { //if message stream is closed, break the loop
                            break;
                        }
                        if let Some(Ok(ws::Message::Text(message))) = message {
                            // If the message is valid, pass the `message` to the handler
                            handle_command(&user_manager, &mut current_user, &mut stream, sender.clone(), &message).await; //handle the command sent, this does registration, login, text and group chat
                        } else if let Some(Err(e)) = message {
                            eprintln!("Error reading message: {:?}", e);  //print the error
                            break; // Break the loop if there is an error
                        }
                    },  //end of select 1

                    content = receiver.next()  => { //2.listen for messsges in channel to send to web socket
                        if content.is_some() {
                            if let Some(content) = content {
                                let _ = stream.send(ws::Message::Binary(content)).await;
                            } else {
                                eprintln!("Error recieving content in channel.");
                            }
                        } //end if conent
                    } //end of select 2

                } //end select
            } //end infininte loop

            cleanup(user_manager.clone(), current_user).await; //clean up before returing
            Ok(()) //return Ok to closure, since nothing failed
        }) //end of Box
    }) //end ws.channel closure
} //end fn app channel

#[launch]
fn rocket() -> _ {
    let user_manager = UserManager::load("users.json").unwrap(); //load the user manager

    let figment = rocket::Config::figment().merge(("port", 1111)); //create modified configuration for Rocket application

    rocket::custom(figment) //use the modified configuration
        .manage(Arc::new(Mutex::new(user_manager))) //the user manager will be safely shared across between rocket routes
        .mount("/", routes![app_channel]) //start rocket with app_channel that definine the behaviour of the endpoint
}
