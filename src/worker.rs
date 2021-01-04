use image::ImageResult;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct WorkerRequest {
    pub buffer: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub sender: oneshot::Sender<WorkerResponse>,
}

pub type WorkerResponse = ImageResult<Vec<u8>>;

pub fn start_worker() -> mpsc::Sender<WorkerRequest> {
    let (sender, mut receiver) = mpsc::channel::<WorkerRequest>(1);
    tokio::spawn(async move {
        while let Some(req) = receiver.recv().await {
            let resp = convert(req.buffer,req.width,req.height);
            req.sender.send(resp).ok();
        }
    });
    sender
}

fn convert(data: Vec<u8>, width: u16, height: u16) -> ImageResult<Vec<u8>> {
    let format = image::guess_format(&data)?;
    let img = image::load_from_memory(&data)?;
    let scaled = img.resize(
        width as u32,
        height as u32,
        image::imageops::FilterType::Lanczos3,
    );
    let mut result = Vec::new();
    scaled.write_to(&mut result, format)?;
    Ok(result)
}
