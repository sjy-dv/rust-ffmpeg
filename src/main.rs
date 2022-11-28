use std::process::{Stdio, Command};
use futures::{future::ready, StreamExt};

#[macro_use]
mod lib;

use crate::lib::opt::{FfmpegBuilder, File, Parameter};

#[tokio::main]
async fn main() {
    // reduce_no_rotate_video().await;
    // reduce_video().await;
    // convert_hls().await;
    // rtsp_stream().await;
}

// reduce video size && rotate delete
async fn reduce_no_rotate_video() {
    // extention is not important
    let builder = FfmpegBuilder::new()
    .stderr(Stdio::piped())
    .option(Parameter::Single("nostdin"))
    .option(Parameter::Single("y"))
    .input(File::new("./input/index.mp4"))
    .output(File::new("./output/index.mp4")
        .option(Parameter::KeyValue("vcodec", "libx264"))
        .option(Parameter::KeyValue("crf", "28"))
    );

    let out = builder.run().await.unwrap();

    out.progress.for_each(|x| {
        dbg!(x.unwrap());
        ready(())
    }).await;

    let output = out.process.wait_with_output().unwrap();
    
    println!("success {}", std::str::from_utf8(&output.stderr).unwrap());
}

// reduce video
async fn reduce_video() {
    let builder = FfmpegBuilder::new()
    .stderr(Stdio::piped())
    .option(Parameter::Single("nostdin"))
    .option(Parameter::Single("y"))
    .input(File::new("./input/index.mp4"))
    .output(File::new("./output/index.mp4")
        .option(Parameter::KeyValue("vcodec", "copy"))
        .option(Parameter::KeyValue("crf", "28"))
    );

    let out = builder.run().await.unwrap();

    out.progress.for_each(|x| {
        dbg!(x.unwrap());
        ready(())
    }).await;

    let output = out.process.wait_with_output().unwrap();
    
    println!("success {}", std::str::from_utf8(&output.stderr).unwrap());
}

// video => m3u8
// example mp4 => m3u8
async fn convert_hls() {
    let builder = FfmpegBuilder::new()
    .stderr(Stdio::piped())
    .option(Parameter::Single("nostdin"))
    .option(Parameter::Single("y"))
    .input(File::new("./input/index.mp4"))
    .output(File::new("./output/index.m3u8")
        .option(Parameter::KeyValue("vcodec", "libx264"))
        .option(Parameter::KeyValue("crf", "28"))
        .option(Parameter::KeyValue("hls_time", "5"))
    );

    let out = builder.run().await.unwrap();

    out.progress.for_each(|x| {
        dbg!(x.unwrap());
        ready(())
    }).await;

    let output = out.process.wait_with_output().unwrap();
    
    println!("success {}", std::str::from_utf8(&output.stderr).unwrap());
}

async fn rtsp_stream() {
    let rtsp_url = "rtsp://wowzaec2demo.streamlock.net/vod/mp4:BigBuckBunny_115k.mp4";
    Command::new("ffmpeg")
    .args([
        "-y",
        "-fflags",
        "nobuffer",
        "-rtsp_transport",
        "tcp",
        "-i",
        rtsp_url,
        "-c:v",
        "copy",
        "-crf",
        "28",
        "-preset",
        "veryfast",
        "-c:a",
        "copy",
        "-f",
        "hls",
        "-hls_time",
        "1",
        "-hls_list_size",
        "5",
        "-hls_flags",
        "delete_segments",
        "./output/index.m3u8",
      ]).spawn().expect("command failed");
}