mod ring;
pub mod ring_grpc;

use crate::ring::Empty;
use crate::ring_grpc::RingClient;
use grpc::{ ClientConf, ClientStubExt, Error as GrpcError, RequestOptions };
use std::net::SocketAddr;

pub struct Remote {
    client: RingClient
}

pub enum Action {
    StartRollCall,
    MarkItself
}

impl Remote {
    pub fn new(addr: SocketAddr) -> Result<Self, GrpcError> {
        let host = addr.ip().to_string();
        let port = addr.port();
        let client = RingClient::new_plain(&host, port, ClientConf::default())?;
        Ok(Remote { client })
    }

    pub async fn start_roll_call(&self) -> Result<Empty, GrpcError> {
        self.client
            .start_roll_call(RequestOptions::default(), Empty::new())
            .drop_metadata().await
    }

    pub async fn mark_itself(&self) -> Result<Empty, GrpcError> {
        self.client
            .mark_itself(RequestOptions::default(), Empty::new())
            .drop_metadata().await
    }
}

