use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;


fn main()  {
    let dotenvy_res = dotenvy::dotenv();

    match dotenvy_res {
        Ok(_) => println!("dotenvy loaded"),
        Err(e) => println!("dotenvy error: {}", e),
    }

    let rabbit_login = dotenvy::var("RABBIT_LOGIN").unwrap();
    let rabbit_password = dotenvy::var("RABBIT_PASSWORD").unwrap();
    let rabbit_host = dotenvy::var("RABBIT_HOST").unwrap();
    let rabbit_port = dotenvy::var("RABBIT_PORT").unwrap();

    let rabbit_conn_url = format!("amqp://{rabbit_login}:{rabbit_password}@{rabbit_host}:{rabbit_port}");
    // Connect to RabbitMQ
    let mut connection = Connection::insecure_open(rabbit_conn_url.as_str()).unwrap();
    let channel = connection.open_channel(None).unwrap();

    // Declare the transcription queue
    let queue = channel.queue_declare("files_to_transcribe", QueueDeclareOptions::default()).unwrap();

    // Start consuming messages from the queue
    let consumer = queue.consume(ConsumerOptions::default()).unwrap();
    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                // Download the audio file
                let mut file = File::create("audio.mp3").unwrap();
                file.write_all(&delivery.body).unwrap();

                // Convert the audio file to WAV using ffmpeg
                let ffmpeg = Command::new("ffmpeg")
                    .arg("-i")
                    .arg("audio.mp3")
                    .arg("-acodec")
                    .arg("pcm_s16le")
                    .arg("-ar")
                    .arg("16000")
                    .arg("-ac")
                    .arg("1")
                    .arg("audio.wav")
                    .output()
                    .unwrap();

                // Print the output of the ffmpeg command
                println!("ffmpeg: {}", String::from_utf8_lossy(&ffmpeg.stdout));

                // ./main -otxt -m models/ggml-base.en.bin -f samples/jfk.wav
                let whisper = Command::new("./main")
                    .arg("-oj")         // json output
                    .arg("-m")
                    .arg("models/ggml-base.bin")
                    .arg("-f")
                    .arg("audio.wav")
                    .output()
                    .unwrap();
                
                println!("{}", String::from_utf8_lossy(&whisper.stdout));
                // // Acknowledge the message
                // let message = "Transcription complete".as_bytes().to_vec();
                // results_queue.publish(Publish::new(&message, "", "transcription_results")).unwrap();
                
                consumer.ack(delivery).unwrap();
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }
}
