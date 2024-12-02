Gleb Zvonkov 1011806636
Mingcheng He 1004869573

# Final Project ECE1724: WebSocket Chat Server with Rocket
This project implements a real-time WebSocket chat server using the Rocket framework in Rust.
It allows a user to register, login, send text messages, create and join a groupchat, and send files.

## How it works
Users connect to the server via WebSocket at the /app endpoint.
Once connected, users can send messages, which are processed by the server and passed to other clients.
An unbounded channel is used for asynchronous message handling, allowing efficient communication without blocking operations.

## Setup
Start the server via cargo run.
Start any number of clients via cargo run.
Follow the instructions for inputing commands using the client.

## Commands
### User Registration & Login: 
1. Register a new account: 
   reg -u userName -p password   
   Example: reg -u JohnDoe -p mySecurePassword

2. Login to an existing account: 
   login -u userName -p password   
   Example: login -u JohnDoe -p mySecurePassword

### Messaging:
3. Send a message to another user: 
   text -u otherUser message   
   Example: text -u JaneDoe Hello, how are you?

4. Send a message to multiple users: 
   textMultiple -u user1 user2 -t message    
   Example: textMultiple -u John -t Meeting at 5 PM!

### Group Chat: 
5. Start a new group chat: 
   startchat chatName  
   Example: startchat WorkTeam 

6. Join an existing group chat: 
   joinchat chatName  
   Example: joinchat WorkTeam 

7. Send a message to a group chat: message sentence          
   Example: message Good morning everyone!    

### File Sharing: 
8. Send a file to another user: 
   file -u otherUser filePath 
   Example: file -u JaneDoe /Users/glebzvonkov/Downloads/rust/client_server/testsend.txt 


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
)
async fn handle_quit_chat(
    user_manager: &mut UserManager,
    cur: Option<String>,
    name: String,
    stream: &mut DuplexStream,
)
async fn handle_file_message(
    user_manager: &mut UserManager,
    current_user: Option<String>,
    filename: String,
    file_path: String,
    stream: &mut DuplexStream,
)
async fn receive_and_forward_file(
    stream: &mut DuplexStream,
    channel: &mut UnboundedSender<Vec<u8>>,
) -> io::Result<()>
async fn send_response(stream: &mut DuplexStream, message: &str)
async fn notify_chat_users(
    user_manager: &mut UserManager,
    sender_name: &str,
    chat_users: HashSet<String>,
    message: &str,
)
fn parse_command(cmd: &str) -> Option<Command>
pub async fn handle_command(
    user_manager: &Arc<Mutex<UserManager>>,
    current_user: &mut Option<String>,
    stream: &mut DuplexStream,
    sender: UnboundedSender<Vec<u8>>,
    message: &str,
)

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
