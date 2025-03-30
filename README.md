# async-tqsm

Asynchronous, streaming sentence segmentation based on [tqsm](https://github.com/mush42/tqsm).

This project extends the rule-based sentence segmentation capabilities of `libtqsm` to support asynchronous, chunk-by-chunk processing, making it suitable for real-time applications and Unix pipelines.

## Features

- **Streaming Segmentation:** Processes text incrementally as it arrives, emitting sentences with low latency.
- **Asynchronous:** Built with Tokio for non-blocking I/O, ideal for integration into async Rust applications.
- **Rule-Based:** Leverages the robust, multilingual rule sets from `libtqsm` (abbreviations, terminators).
- **Configurable Buffering:** Tune lookahead and buffer size to balance latency vs. accuracy.
- **Dual Interface:** Usable as both a standalone CLI tool and a Rust library.
- **Non-Destructive:** Preserves original whitespace and punctuation, allowing text reconstruction.

## Installation

### CLI Tool

1. Clone the repository:

   ```bash
   git clone https://github.com/WismutHansen/async-tqsm.git
   cd async-tqsm
   ```

2. Build the release binary:

   ```bash
   cargo build --release
   ```

3. The executable will be at `./target/release/async-tqsm`. You can copy it to a location in your `$PATH`.

### Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
async-tqsm = "0.1.0" # Or the latest version / git / path
```

_(Note: This project might uses a fork of `libtqsm` to expose necessary internals for streaming. Ensure the dependency points to the correct source if applicable.)_

## Usage

### Command-Line Interface (CLI)

Pipe text into `async-tqsm`. Sentences will be printed to standard output, separated by newlines.

```bash
# Example: Stream output from another command
some_command_generating_text | async-tqsm [OPTIONS]

# Example: Segment a file
async-tqsm --input-file input.txt --output-file sentences.txt

# Example: Use German rules with custom lookahead
cat story.de.txt | async-tqsm --language de --lookahead 5
```

**Common CLI Options:**

- `-l`, `--language <CODE>`: Set language (default: `en`).
- `--lookahead <CHARS>`: Set minimum lookahead characters (default: `10`).
- `--max-buffer <CHARS>`: Set maximum internal buffer size (default: `8192`).
- `-i`, `--input-file <FILE>`: Read from file instead of stdin.
- `-o`, `--output-file <FILE>`: Write to file instead of stdout.
- `--help`: Show all options.

### Library

Use the `sentences_stream` function to process any asynchronous reader.

```rust
use async_tqsm::{sentences_stream, SegmentOptions};
use tokio::io::stdin;
use futures::StreamExt; // For stream.next()

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = SegmentOptions {
        language: "en".to_string(),
        lookahead: 10,
        max_buffer: 8192,
    };

    // Example using stdin
    let reader = stdin();
    let mut stream = sentences_stream(reader, options);

    while let Some(sentence_result) = stream.next().await {
        match sentence_result {
            Ok(sentence) => println!("{}", sentence),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

## License

Licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
