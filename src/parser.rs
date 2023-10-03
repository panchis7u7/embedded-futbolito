use log::error;
use regex::Regex;
use std::io::{Error, ErrorKind};

// ###################################################################
// Define the Argument trait
// ###################################################################

trait Argument {
    fn name(&self) -> &str;
    fn is_required(&self) -> bool;
}

// ###################################################################
// Define the RequiredArgument struct implementing the Argument trait.
// ###################################################################

pub struct RequiredArgument<T> {
    name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> RequiredArgument<T> {
    fn new(name: &str) -> Self {
        RequiredArgument {
            name: name.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Argument for RequiredArgument<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_required(&self) -> bool {
        true
    }
}

// ###################################################################
// Define the OptionalArgument struct implementing the Argument trait.
// ###################################################################

pub struct OptionalArgument<T> {
    name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> OptionalArgument<T> {
    fn new(name: &str) -> Self {
        OptionalArgument {
            name: name.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Argument for OptionalArgument<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_required(&self) -> bool {
        false
    }
}

// ###################################################################
// Structure for a final parsed command.
// ###################################################################

pub struct Command {
    pub command: String,
    pub required_arguments: Vec<(String, String)>,
    pub optional_arguments: Vec<(String, String)>,
}

impl Command {
    const INVALID_CMD: &str = "You have entered an invalid Command!";
    const NO_CMD: &str = "Command was not specified!";
    const INVALID_SYNTAX: &str = "Sorry, I could not understand you.";
    const MISSING_ARGS: &str = "Missing required variables.";

    pub fn invalid(error: &str) -> Error {
        Error::new(ErrorKind::InvalidData, format!("{}", error))
    }
}

// ###################################################################
// Define the Parser struct
// ###################################################################

pub struct Parser {
    commands: std::collections::HashMap<String, Vec<Box<dyn Argument>>>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            commands: std::collections::HashMap::new(),
        }
    }

    pub fn add_command(&mut self, name: &str, args: Vec<Box<dyn Argument>>) {
        self.commands.insert(name.to_string(), args);
    }

    pub fn get_command_arguments(&self, name: &str) -> Option<&Vec<Box<dyn Argument>>> {
        self.commands.get(name)
    }

    pub fn parse(&self, plain_string_message: String) -> Result<Command, Error> {
        let re = Regex::new(r"\/?\w+").unwrap();

        // Every command must initialize with "/"
        if !re.is_match(&plain_string_message) {
            error!("Error 1");
            Err::<Command, Error>(Command::invalid(Command::INVALID_CMD));
        }

        // If the command is correctly initialized, check if it is available as
        // a key within the hasmap.
        if !self.commands.contains_key(plain_string_message.as_str()) {
            Err::<Command, Error>(Command::invalid(Command::INVALID_CMD));
        }

        // If it is valid, check for any required arguments.
        for argument in self.commands.get(plain_string_message.as_str()).unwrap() {
            if argument.is_required() {}
        }

        let mut command = Command {
            command: "test".to_string(),
            required_arguments: Vec::new(),
            optional_arguments: Vec::new(),
        };

        Ok(command)
    }
}
