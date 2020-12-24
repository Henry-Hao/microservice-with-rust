use failure::Error;
use microservice_with_rust::Remote;
use std::env;
use futures::executor;

fn main() -> Result<(), Error> {
    let next = env::var("NEXT")?.parse()?;
    let remote = Remote::new(next)?;
    let resp = remote.start_roll_call();
    executor::block_on(resp)
        .map(|_| ())
        .map_err(|err| failure::format_err!("Error:{}", err))
}
