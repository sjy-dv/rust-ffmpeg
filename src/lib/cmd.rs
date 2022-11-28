//cmd line

use std::{process::Child, time::Duration};

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    SinkExt,
};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
};

use crate::lib::opt::{FfmpegBuilder, Parameter};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Ffmpeg {
    pub progress: UnboundedReceiver<Result<Progress>>,
    pub process: Child,
}

#[derive(Debug, Default)]
pub struct Progress {
    pub frame: Option<u64>,
    pub fps: Option<f64>,
    pub total_size: Option<u64>,
    pub out_time: Option<Duration>,
    pub dup_frames: Option<u64>,
    pub drop_frames: Option<u64>,
    pub speed: Option<f64>,
    pub status: Status,
}

#[derive(Debug)]
pub enum Status {
    Continue,
    End,
}

impl Default for Status {
    fn default() -> Self {
        Self::Continue
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Io Error: {0}")]
    IoError(
        #[source]
        #[from]
        std::io::Error,
    ),
    #[error("Invalid key=value pair: {0}")]
    KeyValueParseError(String),
    #[error("Unknown status: {0}")]
    UnknownStatusError(String),
    #[error("Parse Error: {0}")]
    OtherParseError(#[source] Box<dyn std::error::Error + Send>, String),
}

impl<'a> FfmpegBuilder<'a> {
    pub async fn run(mut self) -> Result<Ffmpeg> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let prog_url = format!("tcp://127.0.0.1:{}", port);

        self = self.option(Parameter::KeyValue("progress", &prog_url));
        let mut command = self.to_command();
        let child = command.spawn()?;

        let conn = listener.accept().await?.0;

        let (mut tx, rx) = mpsc::unbounded();

        tokio::spawn(async move {
            let mut reader = BufReader::new(conn);
            let mut progress: Progress = Default::default();

            loop {
                let mut line = String::new();
                let read = reader.read_line(&mut line).await;

                match read {
                    Ok(n) => {
                        if n == 0 {
                            tx.close_channel();
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.into())).await;
                        tx.close_channel();
                    }
                }

                if let Some((key, value)) = parse_line(&line) {
                    match key {
                        "frame" => match value.parse() {
                            Ok(x) => progress.frame = Some(x),
                            Err(e) => handle_parse_error(&mut tx, e, value).await,
                        },
                        "fps" => match value.parse() {
                            Ok(x) => progress.fps = Some(x),
                            Err(e) => handle_parse_error(&mut tx, e, value).await,
                        },
                        "total_size" => match value.parse() {
                            Ok(x) => progress.total_size = Some(x),
                            Err(e) => handle_parse_error(&mut tx, e, value).await,
                        },
                        "out_time_us" => match value.parse() {
                            Ok(us) => progress.out_time = Some(Duration::from_micros(us)),
                            Err(e) => handle_parse_error(&mut tx, e, value).await,
                        },
                        "dup_frames" => match value.parse() {
                            Ok(x) => progress.dup_frames = Some(x),
                            Err(e) => handle_parse_error(&mut tx, e, value).await,
                        },
                        "drop_frames" => match value.parse() {
                            Ok(x) => progress.drop_frames = Some(x),
                            Err(e) => handle_parse_error(&mut tx, e, value).await,
                        },
                        "speed" => {
                            let num = &value[..(value.len() - 1)];
                            match num.parse() {
                                Ok(x) => progress.speed = Some(x),
                                Err(e) => handle_parse_error(&mut tx, e, num).await,
                            }
                        }
                        "progress" => {
                            progress.status = match value {
                                "continue" => Status::Continue,
                                "end" => Status::End,
                                x => {
                                    let _ = tx.feed(Err(Error::UnknownStatusError(x.to_owned())));
                                    tx.close_channel();

                                    Status::End
                                }
                            };
                            match tx.feed(Ok(progress)).await {
                                Ok(_) => {}
                                Err(e) => {
                                    if e.is_disconnected() {
                                        tx.close_channel();
                                    }
                                }
                            }
                            progress = Default::default();
                        }
                        _ => {}
                    }
                } else {
                    let _ = tx.send(Err(Error::KeyValueParseError(line)));
                    tx.close_channel();
                }
            }
        });

        Ok(Ffmpeg {
            progress: rx,
            process: child,
        })
    }
}

fn parse_line<'a>(line: &'a str) -> Option<(&'a str, &'a str)> {
    let trimmed = line.trim();
    let mut iter = trimmed.splitn(2, '=');

    let mut key = iter.next()?;
    key = key.trim_end();

    let mut value = iter.next()?;
    value = value.trim_start();

    Some((key, value))
}

async fn handle_parse_error(
    tx: &mut UnboundedSender<Result<Progress>>,
    e: impl std::error::Error + Send + 'static,
    x: &str,
) {
    let _ = tx
        .send(Err(Error::OtherParseError(Box::new(e), x.to_owned())))
        .await;
    tx.close_channel();
}