mod ring;
mod ring_grpc;
use ring::Empty;
use ring_grpc::{ Ring, RingServer };
use failure::Error;
use grpc::{ Error as GrpcError, ServerBuilder, SingleResponse };
use microservice_with_rust::{ Action, Remote };
use log::{ debug, trace };
use std::env;
use std::net::SocketAddr;
use std::sync::{ Mutex, mpsc::{ channel, Receiver, Sender } };


macro_rules! try_or_response {
    ($x:expr) => {
        match ($x) {
            Ok(value) => value,
            Err(err) => {
                let error = GrpcError::Panic(err.to_string());
                return SingleResponse::err(error);
            }
        }
    };
}

struct RingImpl {
    sender: Mutex<Sender<Action>>
}

impl RingImpl {
    fn new(sender: Sender<Action>) -> Self {
        RingImpl {
            sender: Mutex::new(sender)
        }
    }

    fn send_action(&self, action: Action) -> SingleResponse<Empty> {
        let sender = try_or_response!(self.sender.lock());
        try_or_response!(sender.send(action));
        let result = Empty::new();
        SingleResponse::completed(result)
    }
}

impl Ring for RingImpl {
    fn start_roll_call(&self, _o: grpc::ServerHandlerContext, _: grpc::ServerRequestSingle<Empty>, resp: grpc::ServerResponseUnarySink<Empty>) -> grpc::Result<()> {
        trace!("START_ROLL_CALL");
        self.send_action(Action::StartRollCall);
        resp.finish(Empty::new())
    }

    fn mark_itself(&self, _o: grpc::ServerHandlerContext, _: grpc::ServerRequestSingle<Empty>, resp: grpc::ServerResponseUnarySink<Empty>) -> grpc::Result<()> {
        trace!("MARK_ITSELF");
        self.send_action(Action::MarkItself);
        resp.finish(Empty::new())
    }
}


async fn worker_loop(receiver: Receiver<Action>) -> Result<(), Error> {
    let next = env::var("NEXT")?.parse()?;
    let remote = Remote::new(next)?;
    let mut in_roll_call = false;
    for action in receiver.iter() {
        match action {
            Action::StartRollCall => {
                if !in_roll_call {
                    if remote.start_roll_call().await.is_ok() {
                        debug!("ON");
                        in_roll_call = true;
                    }
                } else {
                    if remote.mark_itself().await.is_ok() {
                        debug!("OFF");
                        in_roll_call = false;
                    }
                }
            },
            Action::MarkItself => {
                if in_roll_call {
                    if remote.mark_itself().await.is_ok() {
                        debug!("OFF");
                        in_roll_call = false;
                    }
                } else {
                    debug!("SKIP");
                }
            }
        };
    };
    Ok(())

}

fn main() -> Result<(), Error> {
    env_logger::init();
    let (sender, receiver) = channel();
    let addr: SocketAddr = env::var("ADDRESS")?.parse()?;
    let mut server = ServerBuilder::new_plain();
    server.http.set_addr(addr)?;
    let ring = RingImpl::new(sender);
    server.add_service(RingServer::new_service_def(ring));
    let _server = server.build()?;

    let worker = worker_loop(receiver);
    futures::executor::block_on(worker)

}
