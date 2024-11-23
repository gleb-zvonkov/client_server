/*
This is the user manager module.
It stores information about each user and global information such as the users online.
It implements several methods like saving and reading user infromation from a json.
Not that currently the chat info does not persist after the server is restarted.
*/

use bcrypt::{hash, DEFAULT_COST}; //used to encrypt the password
use futures::channel::mpsc::UnboundedSender; //used to associate a channel with each user
use serde::{Deserialize, Serialize}; //used to store user info as json
use std::{
    collections::{HashMap, HashSet}, //used for user managers
    fs,                              //used to read the contents of files
    fs::File,                        //used to create a file
    io::{self, Write},               //for input and ouptus, Write used to write bytes
}; //serialize and deserialize messsages

type ChatError = Box<dyn std::error::Error + Send + Sync>; //Type alias error can be used safely in multithreaded environment

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub name: String,                 //user has a name
    pub password: String,             //and a password
    pub current_chat: Option<String>, //and a current chat tyhat can be None
}

#[derive(Clone)]
pub struct UserManager {
    pub users: HashMap<String, User>, //User name (key) -> User object
    pub onlines: HashMap<String, UnboundedSender<Vec<u8>>>, //User name (key) -> Channel sender
    pub chats: HashMap<String, HashSet<String>>, //Chat name (key) -> Set of user names (hash set only has unique values)
    pub file: String,                            //File where we store all user info
}

//Methods for UserManager struct
impl UserManager {
    //Intialize a new instance of UserManager
    pub fn new(file_path: &str) -> UserManager {
        UserManager {
            users: HashMap::new(),       //users
            file: file_path.to_string(), //file where it will be stored
            onlines: HashMap::new(),     //keep track online users
            chats: HashMap::new(),       //keep track of the existing chats
        }
    }

    //Load data from file (the user info) into UserManager instance
    pub fn load(file_path: &str) -> Result<UserManager, ChatError> {
        if fs::metadata(file_path).is_ok() {
            let content = fs::read_to_string(file_path)?;
            let users: Vec<User> =
                serde_json::from_str(&content).map_err(|e| Box::new(e) as ChatError)?; //if error convert to io error, wrap in ChatError
            let mut user_manager = UserManager::new(file_path); //create new user manager instance
            user_manager.users = HashMap::from_iter(users.into_iter().map(|x| (x.name.clone(), x))); //add users from file into userManager instance
                                                                                                     //deserialized Vec<User> is converted into a HashMap<String, User> where each User's name is the key (x.name.clone()) and the User object itself is the value.
            Ok(user_manager)
        } else {
            return Ok(UserManager::new(file_path)); //if there is no file create a new one
        }
    }

    //save current state of the users into a file
    pub fn save(self: &Self) -> Result<(), ChatError> {
        let mut f = File::create(self.file.as_str())?; //create or overwrite file
        let us: Vec<User> = self.users.values().cloned().collect();
        let res = serde_json::to_string(&us)?; //conver it to json
        f.write_all(res.as_bytes())?; //write to the file
        Ok(())
    }

    //add new user to UserManager's user hashmap
    pub fn add(self: &mut Self, user: &User) -> Result<(), ChatError> {
        if self.users.contains_key(&user.name) {
            //check if user already exists
            return Err(Box::new(std::io::Error::new(
                io::ErrorKind::Other,
                "ERROR: This user already exists.",
            )));
        } else {
            let hashed_password = hash(&user.password, DEFAULT_COST)?;
            let user_with_hashed_password = User {
                name: user.name.clone(),
                password: hashed_password,
                current_chat: None,
            };
            self.users
                .insert(user.name.clone(), user_with_hashed_password);
            self.save() //save the UserManager
        }
    }

    //get the current chat
    pub fn get_current_chat(&self, user_name: &str) -> Option<String> {
        self.users
            .get(user_name)
            .and_then(|user| user.current_chat.clone())
    }

    //Set the current chat
    pub fn set_current_chat(
        &mut self,
        user_name: &str,
        chat: Option<String>,
    ) -> Result<(), ChatError> {
        if let Some(user) = self.users.get_mut(user_name) {
            user.current_chat = chat;
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                io::ErrorKind::NotFound,
                "User not found",
            )))
        }
    }

    //Authenticate the user by checking his password using bcrypt
    pub fn authenticate(&self, name: &str, password: &str) -> Result<bool, ChatError> {
        if let Some(user) = self.users.get(name) {
            let matches = bcrypt::verify(password, &user.password)?;
            Ok(matches)
        } else {
            Err(Box::new(std::io::Error::new(
                io::ErrorKind::NotFound,
                "User not found",
            )))
        }
    }
}
