use failure::Error;
use jsonrpc::client::Client;
use jsonrpc::error::Error as ClientError;
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_core::{ IoHandler, Error as ServerError, Value };
use log::{ debug, error, trace };
use serde::Deserialize;
use std::{ env, fmt, net::SocketAddr, thread };
use std::sync::{ Mutex, mpsc::{ channel, Sender }};

const START_ROLL_CALL: &str = "start_roll_call";
const MARK_ITSELF: &str = "mark_itself";

struct Remote {
    client: Client
}

enum Action {
    StartRollCall,
    MarkItself
}

impl Remote {
    fn new(addr: SocketAddr) -> Self {
        let url = format!("http://{}", addr);
        let client = Client::new(url, None, None);
        Remote {client}
    }

    fn call_method<T>(&self, method: &str, args: &[Value]) -> Result<T, ClientError> 
    where T: for<'de> Deserialize<'de>{
        let request = self.client.build_request(method, args);
        self.client.send_request(&request).and_then(|response| response.into_result::<T>()) 
    }

    fn start_roll_call(&self) -> Result<bool, ClientError> {
        self.call_method(START_ROLL_CALL, &[])
    }

    fn mark_itself(&self) -> Result<bool, ClientError> {
        self.call_method(MARK_ITSELF, &[])
    }
}

fn spawn_worker() -> Result<Sender<Action>, Error> {
    let (sender, receiver) = channel();
    let next: SocketAddr = env::var("NEXT")?.parse()?;
    thread::spawn(move || {
        let remote = Remote::new(next);
        let mut in_roll_call = false;
        for action in receiver.iter() {
            match action {
                Action::StartRollCall => {
                    if !in_roll_call {
                        if remote.start_roll_call().is_ok() {
                            debug!("ON");
                            in_roll_call = true;
                        }
                    } else {
                        if remote.mark_itself().is_ok() {
                            debug!("OFF");
                            in_roll_call = false;
                        }
                    }
                },
                Action::MarkItself => {
                    if in_roll_call {
                        if remote.mark_itself().is_ok() {
                            debug!("OFF");
                            in_roll_call = false;
                        }
                    } else {
                        debug!("SKIP");
                    }
                }
            }
        }
    });
    Ok(sender)
}


fn main() -> Result<(), Error> {
    env_logger::init();
    let original_sender = spawn_worker()?;
    let addr: SocketAddr = env::var("ADDRESS")?.parse()?;
    let mut io = IoHandler::new();
    let sender = Mutex::new(original_sender.clone());
    io.add_sync_method(START_ROLL_CALL, move |_| {
        trace!("START_ROLL_CALL");
        let sender = sender
            .lock()
            .map_err(to_internal)?;
        sender.send(Action::StartRollCall)
            .map_err(to_internal)
            .map(|_| Value::Bool(true))
    });

    let sender = Mutex::new(original_sender.clone());
    io.add_sync_method(MARK_ITSELF, move |_| {
        trace!("MARK_ITSELF");
        let sender = sender.lock().map_err(to_internal)?;
        sender.send(Action::MarkItself)
            .map_err(to_internal)
            .map(|_| Value::Bool(true))
    });

    let server = ServerBuilder::new(io).start_http(&addr)?;

    Ok(server.wait())
}


fn to_internal<E:fmt::Display>(err: E) -> ServerError {
    error!("Error:{}", err);
    ServerError::internal_error()
}
