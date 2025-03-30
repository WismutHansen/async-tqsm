use async_stream::stream;
use futures::stream::Stream;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
pub mod config;
mod error;
mod segmenter;

pub use config::SegmentOptions;
pub use error::{Result, SegmenterError};
pub use segmenter::Segmenter;

/// Creates an asynchronous stream of sentences from a reader.
///
/// Reads data from the provided `AsyncRead` source, segments it into sentences
/// based on the `SegmentOptions`, and yields each sentence as a `String`.
///
/// # Arguments
///
/// * `reader`: An asynchronous reader (e.g., `tokio::io::Stdin`, `tokio::fs::File`).
/// * `options`: Configuration for the sentence segmenter.
///
/// # Returns
///
/// An implementation of `Stream` that yields `Result<String, SegmenterError>`.
/// Errors during I/O or segmentation will be returned as `Err` variants in the stream.
pub fn sentences_stream<R>(reader: R, options: SegmentOptions) -> impl Stream<Item = Result<String>>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    stream! {
        let mut segmenter = match Segmenter::new(options) {
            Ok(s) => s,
            Err(e) => {
                yield Err(e);
                return;
            }
        };

        let mut buf_reader = BufReader::new(reader);
        let mut buffer = [0; 4096]; // Read in 4KB chunks

        loop {
            match buf_reader.read(&mut buffer).await {
                Ok(0) => {
                    // EOF reached
                    break;
                }
                Ok(n) => {
                    // Process the chunk
                    // Need to handle potential UTF-8 errors if a character is split across chunks
                    // BufReader should mitigate this for read_line, but read might still split.
                    // A safer approach involves a dedicated UTF-8 aware buffer/decoder if read() is used directly.
                    // For simplicity with `read`, we'll attempt direct conversion and handle errors.
                    match std::str::from_utf8(&buffer[..n]) {
                         Ok(chunk_str) => {
                              match segmenter.feed(chunk_str) {
                                   Ok(sentences) => {
                                       for sentence in sentences {
                                           yield Ok(sentence);
                                       }
                                   }
                                   Err(e) => {
                                       yield Err(e);
                                       // Decide whether to stop streaming on error
                                       // return;
                                   }
                              }
                         }
                         Err(e) => {
                             yield Err(SegmenterError::Utf8Error(e));
                             // Decide whether to stop streaming on UTF-8 error
                             // return;
                         }
                    }
                }
                Err(e) => {
                    yield Err(SegmenterError::IoError(e));
                    // Stop streaming on I/O error
                    return;
                }
            }
        }

        // Flush any remaining text after EOF
        match segmenter.flush() {
            Ok(Some(last_sentence)) => {
                if !last_sentence.is_empty() {
                    yield Ok(last_sentence);
                }
            }
            Ok(None) => { /* No remaining text, do nothing */ }
            Err(e) => {
                yield Err(e);
            }
        }
    }
}

// Example Usage (Optional, for testing within the lib)
#[cfg(test)]
mod tests {
    use super::*;
    use futures::pin_mut;
    use tokio::io::Result as TokioResult; // Alias to avoid conflict

    // A simple mock reader
    struct MockReader {
        data: Vec<&'static str>,
        pos: usize,
    }

    impl AsyncRead for MockReader {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<TokioResult<()>> {
            if self.pos < self.data.len() {
                let chunk = self.data[self.pos];
                buf.put_slice(chunk.as_bytes());
                self.pos += 1;
                std::task::Poll::Ready(Ok(()))
            } else {
                std::task::Poll::Ready(Ok(())) // EOF
            }
        }
    }

    #[tokio::test]
    async fn test_stream_basic() -> anyhow::Result<()> {
        let options = SegmentOptions::default();
        let reader = MockReader {
            data: vec!["Hello Mr. Smith", ". How are", " you today? Good!"],
            pos: 0,
        };
        pin_mut!(reader); // Pin the reader

        let stream = sentences_stream(reader, options);
        pin_mut!(stream); // Pin the stream

        let mut results = Vec::new();
        while let Some(res) = stream.next().await {
            results.push(res?);
        }

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], "Hello Mr. Smith.");
        assert_eq!(results[1], "How are you today?");
        assert_eq!(results[2], "Good!"); // Final flush

        Ok(())
    }

    #[tokio::test]
    async fn test_stream_lookahead() -> anyhow::Result<()> {
        let options = SegmentOptions {
            lookahead: 5,
            ..Default::default()
        }; // Need lookahead > " How" length
        let reader = MockReader {
            data: vec!["Hello Mr. Smith.", " How are you?"], // Boundary '.' is close to end of first chunk
            pos: 0,
        };
        pin_mut!(reader);

        let stream = sentences_stream(reader, options);
        pin_mut!(stream);

        let mut results = Vec::new();
        while let Some(res) = stream.next().await {
            results.push(res?);
        }

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "Hello Mr. Smith.");
        assert_eq!(results[1], "How are you?");

        Ok(())
    }

    #[tokio::test]
    async fn test_stream_no_final_punctuation() -> anyhow::Result<()> {
        let options = SegmentOptions::default();
        let reader = MockReader {
            data: vec!["First sentence.", " Second sentence has no end"],
            pos: 0,
        };
        pin_mut!(reader);

        let stream = sentences_stream(reader, options);
        pin_mut!(stream);

        let mut results = Vec::new();
        while let Some(res) = stream.next().await {
            results.push(res?);
        }

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "First sentence.");
        assert_eq!(results[1], "Second sentence has no end"); // Flushed content

        Ok(())
    }
}
