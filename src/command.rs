use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::select;
use zenoh::Session;
use zenoh_ext::{z_deserialize, z_serialize};
pub struct Command {
    cb: Box<dyn FnMut(Vec<f32>) -> Option<Vec<f32>> + Send + Sync>,
}

impl Command {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(Vec<f32>) -> Option<Vec<f32>> + Send + Sync + 'static,
    {
        Command { cb: Box::new(f) }
    }
}

pub struct CommandExecutor {
    inner: Arc<Mutex<CommandExecutorInner>>,
}
impl CommandExecutor {
    pub fn new() -> Self {
        CommandExecutor {
            inner: Arc::new(Mutex::new(CommandExecutorInner::new())),
        }
    }
    pub fn add_command(&self, key: String, command: Command) {
        self.inner.lock().unwrap().add_command(key, command);
    }
}

struct CommandExecutorInner {
    commands: HashMap<String, Command>,
}
impl CommandExecutorInner {
    pub fn new() -> Self {
        CommandExecutorInner {
            commands: HashMap::new(),
        }
    }
    pub fn add_command(&mut self, key: String, command: Command) {
        self.commands.insert(key, command);
    }
    fn execute_command_from_parts(&mut self, key: String, arg: Vec<f32>) -> Option<Vec<f32>> {
        match self.commands.get_mut(&key) {
            Some(command) => (command.cb)(arg),
            None => None,
        }
    }
    fn execute_command(&mut self, source: &str) -> Option<Vec<f32>> {
        println!("{}", source);
        let mut tokens: Vec<&str> = source.split_whitespace().collect();
        tokens.reverse();
        let mut args: Vec<f32> = Vec::new();
        let mut return_val: Option<Vec<f32>> = None;

        for token in tokens.iter() {
            match token.parse::<f32>() {
                Ok(value) => args.push(value),
                Err(_) => {
                    return_val = self.execute_command_from_parts(token.to_string(), args.clone());
                    args.clear();
                }
            }
        }
        return_val
    }
}

pub struct CommandListener {
    executor: Arc<Mutex<CommandExecutorInner>>,
    key_expr: String,
}

impl CommandListener {
    pub fn new(key_expr: String, executor: &CommandExecutor) -> Self {
        CommandListener {
            key_expr,
            executor: Arc::clone(&executor.inner),
        }
    }

    pub async fn start(&mut self, session: &Session) {
        println!("Declaring Subscriber on '{}'...", &self.key_expr);
        let subscriber = session.declare_subscriber(&self.key_expr).await.unwrap();

        println!("Declaring Queryable on '{}'...", &self.key_expr);
        let queryable = session.declare_queryable(&self.key_expr).await.unwrap();
        let execuor = Arc::clone(&self.executor);

        tokio::task::spawn(async move {
            loop {
                select! {
                    sample = subscriber.recv_async() => {
                        let sample = sample.unwrap();
                        let payload: String = z_deserialize(sample.payload()).unwrap_or_else(|e| e.to_string().into());

                        println!(">> [Subscriber] Received ('{}': '{}')", sample.key_expr().as_str(),payload);

                        execuor
                            .lock()
                            .unwrap()
                            .execute_command(payload.as_ref());
                    },
                    query = queryable.recv_async() => {
                        let query = query.unwrap();

                        match query.payload(){
                            None => println!(">> [Queryable ] Received Query '{}'", query.selector()),
                            Some(payload)=>{
                                let deserialized_payload: String = z_deserialize(payload)
                                    .unwrap_or_else(|e| e.to_string().into());

                                println!(
                                    ">> [Queryable ] Received Query '{}' with payload '{}'",
                                    query.selector(),
                                    deserialized_payload
                                );

                                let reply = execuor.lock()
                                    .unwrap()
                                    .execute_command(deserialized_payload.as_ref())
                                    .unwrap_or(Vec::<f32>::new());

                                println!(
                                    ">> [Queryable ] Responding ('{}': '{:?}')",
                                    query.key_expr().as_str(),
                                    reply
                                );

                                query.reply(query.key_expr().clone(), z_serialize(&reply)).await.unwrap();

                            }
                        }
                    }
                }
            }
        });
    }
}
