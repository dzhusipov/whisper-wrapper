use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, ExchangeDeclareOptions, ExchangeType, Publish,
    QueueDeclareOptions,
};
use log::{debug, error};
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let dotenvy_res = dotenvy::dotenv();

    match dotenvy_res {
        Ok(_) => debug!("dotenvy loaded"),
        Err(e) => error!("dotenvy error: {}", e),
    }

    let rabbit_login = dotenvy::var("RABBIT_LOGIN").unwrap();
    let rabbit_password = dotenvy::var("RABBIT_PASSWORD").unwrap();
    let rabbit_host = dotenvy::var("RABBIT_HOST").unwrap();
    let rabbit_port = dotenvy::var("RABBIT_PORT").unwrap();

    let rabbit_conn_url =
        format!("amqp://{rabbit_login}:{rabbit_password}@{rabbit_host}:{rabbit_port}");
    // Connect to RabbitMQ
    let mut connection = Connection::insecure_open(rabbit_conn_url.as_str()).unwrap();
    let channel = connection.open_channel(None).unwrap();

    let sender_channel = connection.open_channel(None).unwrap();

    let _sender_queue = sender_channel
        .queue_declare("transcription_results", QueueDeclareOptions::default())
        .unwrap();

    // Declare the transcription queue
    let queue = sender_channel
        .queue_declare("files_to_transcribe", QueueDeclareOptions::default())
        .unwrap();

    let _results_exchange = channel
        .exchange_declare(
            ExchangeType::Direct,
            "fanout",
            ExchangeDeclareOptions::default(),
        )
        .unwrap();

    // Start consuming messages from the queue
    let consumer = queue.consume(ConsumerOptions::default()).unwrap();
    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let headers = delivery.properties.headers().as_ref();
                let filename_amqp: Option<&amiquip::AmqpValue> =
                    headers.expect("empty_filename").get("filename");
                let filename: &amiquip::AmqpValue = filename_amqp.unwrap();
                let filename_string = match filename {
                    amiquip::AmqpValue::LongString(s) => s.clone(),
                    _ => panic!("Expected LongString variant"),
                };

                debug!("Filename String: {}", filename_string);
                // Download the audio file
                println!("{:?}", &delivery.properties.headers());
                let mut file = File::create(filename_string.clone()).unwrap();
                file.write_all(&delivery.body).unwrap();

                let mut audio_file_wav = PathBuf::from(filename_string.clone());
                audio_file_wav.set_extension("wav");

                // Convert the audio file to WAV using ffmpeg
                let ffmpeg = Command::new("ffmpeg")
                    .arg("-i")
                    .arg(filename_string.clone())
                    .arg("-acodec")
                    .arg("pcm_s16le")
                    .arg("-ar")
                    .arg("16000")
                    .arg("-ac")
                    .arg("1")
                    .arg(format!("ffmpeg-{}",audio_file_wav.to_str().unwrap()))
                    .output()
                    .unwrap();

                // Print the output of the ffmpeg command
                debug!("ffmpeg output: {}", String::from_utf8_lossy(&ffmpeg.stdout));

                // ./main -otxt -m models/ggml-base.en.bin -f samples/jfk.wav
                let whisper = Command::new("/Users/dasm/develop/others/whisper.cpp/main")
                    .arg("-oj") // json output
                    .arg("-m")
                    .arg("/Users/dasm/develop/others/whisper.cpp/models/ggml-base.bin")
                    .arg("-f")
                    .arg(format!("/Users/dasm/develop/rust/whisper-rabbit/ffmpeg-{}",audio_file_wav.to_str().unwrap()))
                    .output()
                    .unwrap();

                debug!("{}", String::from_utf8_lossy(&whisper.stdout));
                // Publish a message to the results queue
                sleep(Duration::from_secs(1));
                let mut file: File = File::open(format!("{}.json", audio_file_wav.to_str().unwrap())).unwrap();
                let mut json_data = String::new();
                file.read_to_string(&mut json_data).unwrap();

                let publish: Publish = Publish::new(json_data.as_bytes(), "transcription_results");
                let send_result = sender_channel.basic_publish("", publish);
                match send_result {
                    Ok(_) => debug!("Message sent"),
                    Err(e) => error!("Error sending message: {}", e),
                }
                // Acknowledge the message
                consumer.ack(delivery).unwrap();
            }
            other => {
                debug!("Consumer ended: {:?}", other);
                break;
            }
        }
    }
}
