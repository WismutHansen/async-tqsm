use async_tqsm::config::CliArgs;
use async_tqsm::{sentences_stream, SegmentOptions, SegmenterError};
use clap::Parser;
use futures::StreamExt; // Required for stream.next()
use std::process::exit;
use tokio::fs::File;
use tokio::io::{self, AsyncRead, AsyncWriteExt, BufReader, BufWriter}; // For exiting with error code

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use the struct via the library path
    let args = CliArgs::parse();
    // Pass the args directly to convert into options
    let options = SegmentOptions::from(args.clone());

    // Get the input reader
    let reader_result: Result<Box<dyn AsyncRead + Unpin + Send>, SegmenterError> =
        match args.input_file {
            Some(path) => match File::open(&path).await {
                Ok(file) => Ok(Box::new(file)), // Wrap file directly
                Err(e) => Err(SegmenterError::IoError(e)),
            },
            None => Ok(Box::new(io::stdin())), // Wrap stdin directly
        };

    let reader = match reader_result {
        Ok(r) => BufReader::new(r), // <<< Wrap input in BufReader here
        Err(e) => {
            eprintln!("Error opening input: {}", e);
            exit(1);
        }
    };

    // Get the output writer
    let writer_result: Result<Box<dyn tokio::io::AsyncWrite + Unpin + Send>, SegmenterError> =
        match args.output_file {
            Some(path) => match File::create(&path).await {
                Ok(file) => Ok(Box::new(file)),
                Err(e) => Err(SegmenterError::IoError(e)),
            },
            None => Ok(Box::new(io::stdout())),
        };

    let mut writer = match writer_result {
        Ok(w) => BufWriter::new(w),
        Err(e) => {
            eprintln!("Error opening output: {}", e);
            exit(1);
        }
    };

    // Create and process the stream
    // Pass the BufReader<impl AsyncRead> to the stream function
    let stream = sentences_stream(reader, options);
    futures::pin_mut!(stream);

    while let Some(sentence_result) = stream.next().await {
        match sentence_result {
            Ok(sentence) => {
                if let Err(e) = writer.write_all(sentence.as_bytes()).await {
                    eprintln!("Error writing to output: {}", e);
                    exit(1);
                }
                if let Err(e) = writer.write_all(b"\n").await {
                    eprintln!("Error writing newline to output: {}", e);
                    exit(1);
                }
                if let Err(e) = writer.flush().await {
                    eprintln!("Error flushing output: {}", e);
                    exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error during segmentation: {}", e);
                exit(1);
            }
        }
    }

    if let Err(e) = writer.flush().await {
        eprintln!("Error flushing output buffer: {}", e);
        exit(1);
    }

    Ok(())
}
