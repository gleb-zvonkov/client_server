Gleb Zvonkov 1011806636 gleb.zvonkov@mail.utoronto.ca
Mingcheng He 1004869573 mc.he@mail.utoronto.ca

# Final Project ECE1724: WebSocket Client-Server with Rocket
This project implements a real-time WebSocket chat server using the Rocket framework in Rust.

It allows a user to register, login, send text messages, create and join a groupchat, and send files.

Video demo: https://drive.google.com/file/d/1gIBaWLfH8IoCl7nAVk74AL_x_lkYrWMi/view?usp=sharing

## Motivation
We chose to build a client-server program because it serves as an excellent entry-level project with potential for continuous iteration and feature expansion.
Initially, we focused on implementing basic client messaging.
From there, we extended functionality to include broadcasting messages to multiple clients, creating chat rooms, and eventually enabling file sharing, among other features.
This iterative development process allowed us to maintain a functional project from the early stages.
This was critical for building confidence and ensuring we were progressing in the right direction.
This approach was particularly valuable given our limited experience with Rust and other lower-level compiled languages, enabling us to learn incrementally while achieving tangible milestones.

Another reason we were motivated to build a client-server program was Rust's unique ownership model.
A chat program, which involves multiple clients sending and receiving messages simultaneously, naturally introduces potential concurrency issues.
Rust is particularly well-suited for such challenges due to its strict ownership rules, which ensure memory safety and eliminate data races at compile time.
This makes Rust a powerful choice for building robust and reliable concurrent systems, such as a chat application.

Another reason we were motivated to build a client-server program was our desire to deepen our understanding of key programming concepts.
For example asynchronous programming, chat programs inherently involve handling multiple events simultaneously, such as receiving messages from various clients while sending responses.
This makes asynchronous programming essential, as it allows the system to manage multiple tasks concurrently without blocking the execution of others.
Other concepts we explored include network programming for establishing and managing socket connections.
Error handling to ensure the program could gracefully recover from unexpected issues.
And string manipulation, such as formatting and parsing messages.

From a user's perspective, a chat application is a fundamental use of modern technology.
Platforms such as WhatsApp, Slack, and Microsoft Teams demonstrate how essential chat-based technology is, with billions of active users worldwide relying on them to stay connected.
Communication technology fulfills the innate human need for connection, essential for forming and maintaining social bonds.
In the modern era, chat applications have become lifelines for relationships, community-building, and navigating an interconnected world.

Though many client-server implementations exist in the Rust ecosystem, our chat application stands out as a practical example of a fully-featured program.
It demonstrates key concepts like asynchronous programming and concurrency while providing a solid foundation for adding new features and functionalities.


## Objectives
The primary objective of this project was to develop a real-time chat application that operates entirely within the terminal.
Functionality: To create a seamless real-time chat experience for users through the terminal.
Performance: To make the application lightweight and efficient, minimizing resource usage while maintaining responsiveness.
Scalability: To design the system to handle multiple concurrent users without degradation in performance.


## Features
- **Registration**: The program will allow the users to create new accounts with passwords.
                    The program checks the account name in the backend to ensure there are no duplicates of usernames.
                    The program encrypts the users password.

- **Login**: After an account is created, the program allows the user to login to the account with the account name and password.

- **Online Status**: Login causes the user to be added to a list of online users.

- **Direct Messaging**: The user can send messages directly to another user with username.
                        A message can be sent to multiple users at once.

- **Group Chat**: The user can create a group chat, join an existing group chat and send message to this group chat.
                  Messages sent to a group chat are received by all chat members.

- **File Sharing**: The user can send a file to one other user.
                    The file is converted into a binary format to be sent and recieved.


## Users Guide
A user can use each of the main features by following the command list that is printed when a client is started.
A developper can explore the several modules on the client and server side.
On the server side the command_handler module contains functions that handle commands from client.
On the server side the user_manager module contains functions that creates, modifies, and saves a structure of users.
On the client side the the file_handler module contains functions for sending and receiving a file.


### Reproducibility Guide
We tested our program on macOS Sonoma.
An easy way try all the feature is to follow along with the video demo.

Open a terminal for the server.
`cd server` and `cargo run`.

Open any number of terminals for the clients, each terminal will have one client instance running.
For each client terminal `cd cleint` and `cargo run`.

The rest of the guide assumes you are running three clients.

In each client terminal register a user.
Terminal 1: `reg -u user1 -p password`.
Terminal 2: `reg -u user2 -p password`.
Terminal 3: `reg -u user3 -p password`.

In each cleint terminal login a user.
Terminal 1: `login -u user1 -p password`.
Terminal 2: `login -u user2 -p password`.
Terminal 3: `login -u user3 -p password`.

Send a message from user1 to user2.
Terminal 1: `text -u user2 hi other user hows it going`.

Send a message from user1 to user2 and user3.
Terminal 1: `textMultiple -u user2 user3 -t hi two users hows it going`.

Start a groupchat from user1.
Terminal 1: `startchat chatx`.

Join the groupchat from user2 and user3.
Terminal 2: `joinchat chatx`.
Terminal 3: `joinchat chatx`.

Send a message in the groupchat.
Terminal 1: `message hey guys`.
Terminal 2: `message hey the groupchats working`.
Terminal 3: `message yes it should be working`.

Send a file from user1 to user3.
We have placed a testsend.txt in the client_server folder.
By sending it to a another user it should appear in the client_server/client subfolder.
Terminal 1:  `file -u user2 ../testsend.txt`.
Naviagete to client_server/client testsend.txt should now appear there.


## Contribution by Each Team Member
The two members of the team made equal contribution to the project.

Mingcheng:
He created the foundation of the program, including the server backend and frontend.
He was responsible for the registration and direct messaging function.
He also made updates to Gleb's README file.
The bulk of his work can be viewed here: https://github.com/MingchengHe/Project1724.

Gleb:
He refractored the code and modularized the functions used for handling commands and managing users on the server side.
He created the group chat and file sharing functions.
He created the original version of README file.
The bulk of his work can be viewed here: https://github.com/gleb-zvonkov/client_server.
The commit history on Github shows only Gleb.
That is because Gleb joined later in the process and the original link sent by Mingcheng was not working.
So Gleb proceeded with the source code and uploaded everything to a seperate repository.


## Lessons Learned
The team gained valuable insights to Rust development.
The team learned how to effectively leverage WebSockets for real-time communication, enhancing our understanding of asynchronous programming.
Working with Rocket gave us hands-on experience in building web applications.
The experience underscored the importance of testing and debugging in Rust, especially when dealing with concurrency and real-time data flow.


### Commands
#### User Registration & Login:
1. Register a new account:
   reg -u userName -p password
   Example: `reg -u JohnDoe -p mySecurePassword`

2. Login to an existing account:
   login -u userName -p password
   Example: `login -u JohnDoe -p mySecurePassword`

#### Messaging:
3. Send a message to another user:
   text -u otherUser message
   Example: `text -u JaneDoe Hello, how are you?`

4. Send a message to multiple users:
   textMultiple -u user1 user2 -t message
   Example: `textMultiple -u John -t Meeting at 5 PM!`

#### Group Chat:
5. Start a new group chat:
   startchat chatName
   Example: `startchat WorkTeam`

6. Join an existing group chat:
   joinchat chatName
   Example: `joinchat WorkTeam`

7. Send a message to a group chat: message sentence
   Example: `message Good morning everyone!`

8. Quit an existing group chat:
   quitchat chatName
   Example: `quit WorkTeam`

#### File Sharing:
9. Send a file to another user:
   file -u otherUser filePath
   Example: `file -u JaneDoe ../testsend.txt`


## Appendix
## Server/src
### main.rs
async fn cleanup(user_manager: Arc<Mutex<UserManager>>, current_user: Option<String>);
#[get("/app")]
fn app_channel(user_manager: &State<Context>, ws: ws::WebSocket) -> ws::Channel<'static>;
#[launch]
fn rocket() -> _;

### user_manager.rs
impl UserManager {
    pub fn new(file_path: &str) -> UserManager;
    pub fn load(file_path: &str) -> Result<UserManager, ChatError>;
    pub fn save(self: &Self) -> Result<(), ChatError>;
    pub fn add(self: &mut Self, user: &User) -> Result<(), ChatError>;
    pub fn get_current_chat(&self, user_name: &str) -> Option<String>;
    pub fn set_current_chat(
        &mut self,
        user_name: &str,
        chat: Option<String>,
    ) -> Result<(), ChatError>;
    pub fn authenticate(&self, name: &str, password: &str) -> Result<bool, ChatError>;
}

### command_handlers.rs
fn parse_reg(line: &str) -> Option<Command>;
fn parse_login(line: &str) -> Option<Command>;
fn parse_text(line: &str) -> Option<Command>;
fn parse_text_multiple(line: &str) -> Option<Command>;
fn parse_start_chat(line: &str) -> Option<Command>;
fn parse_join_chat(line: &str) -> Option<Command>;
fn parse_message_chat(line: &str) -> Option<Command>;
fn parse_quit_chat(line: &str) -> Option<Command>;
fn parse_file_command(line: &str) -> Option<Command>;
fn parse_quit(line: &str) -> Option<Command>;
async fn handle_register(
    user_manager: &mut UserManager,
    name: String,
    password: String,
    stream: &mut DuplexStream,
);
async fn handle_login(
    user_manager: &mut UserManager,
    name: String,
    password: String,
    current_user: &mut Option<String>,
    sender: futures::channel::mpsc::UnboundedSender<Vec<u8>>,
    stream: &mut DuplexStream,
);
async fn handle_text_message(
    user_manager: &mut UserManager,
    current_user: Option<String>,
    name: String,
    content: String,
    stream: &mut DuplexStream,
);
async fn handle_text_multiple(
    user_manager: &mut UserManager,
    current_user: Option<String>,
    names: Vec<String>,
    content: String,
    stream: &mut DuplexStream,
);
async fn handle_start_chat(
    user_manager: &mut UserManager,
    current_user: Option<String>,
    name: String,
    stream: &mut DuplexStream,
);
async fn handle_message_chat(
    user_manager: &mut UserManager,
    cur: Option<String>,
    content: String,
    stream: &mut DuplexStream,
);
async fn handle_quit_chat(
    user_manager: &mut UserManager,
    cur: Option<String>,
    name: String,
    stream: &mut DuplexStream,
);
async fn handle_file_message(
    user_manager: &mut UserManager,
    current_user: Option<String>,
    filename: String,
    file_path: String,
    stream: &mut DuplexStream,
);
async fn receive_and_forward_file(
    stream: &mut DuplexStream,
    channel: &mut UnboundedSender<Vec<u8>>,
) -> io::Result<()>
async fn send_response(stream: &mut DuplexStream, message: &str);
async fn notify_chat_users(
    user_manager: &mut UserManager,
    sender_name: &str,
    chat_users: HashSet<String>,
    message: &str,
);
fn parse_command(cmd: &str) -> Option<Command>
pub async fn handle_command(
    user_manager: &Arc<Mutex<UserManager>>,
    current_user: &mut Option<String>,
    stream: &mut DuplexStream,
    sender: UnboundedSender<Vec<u8>>,
    message: &str,
);

## Client/src
### main.rs
fn print_commands();
fn send_loop(channel_reciever: Receiver<OwnedMessage>, mut socket_sender: Writer<TcpStream>);
fn receive_loop(mut socket_receiver: Reader<TcpStream>, channel_sender: Sender<OwnedMessage>);
fn input_to_channel(channel_sender: Sender<OwnedMessage>);
fn main();

### file_handler.rs
pub fn handle_file_upload(channel_sender: Sender<OwnedMessage>, message: &str);
pub fn handle_binary_message(
    data: Vec<u8>,
    file_buffer: &mut Option<(PathBuf, Vec<u8>)>,
) -> Result<(), String>;
pub fn save_file(file_path: PathBuf, data: &[u8]) -> std::io::Result<()>;
