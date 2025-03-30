use clap::Parser;
use std::path::PathBuf;

/// Asynchronous, streaming sentence segmenter based on tqsm
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Minimum lookahead (in characters) required before finalizing a sentence.
    /// Lower values mean lower latency but potentially lower accuracy.
    #[arg(long, value_name = "CHARS", default_value_t = 10)]
    pub lookahead: usize,

    /// Maximum internal buffer size in characters. Helps prevent excessive memory use.
    /// May force splits if exceeded, potentially impacting accuracy.
    #[arg(long, value_name = "CHARS", default_value_t = 8192)] // Increased default
    pub max_buffer: usize,

    /// Language code for segmentation rules (e.g., "en", "de", "es").
    #[arg(long, short, value_name = "CODE", default_value = "en")]
    pub language: String,

    /// Optional input file path. If not provided, reads from stdin.
    #[arg(long, short, value_name = "FILE")]
    pub input_file: Option<PathBuf>,

    /// Optional output file path. If not provided, writes to stdout.
    #[arg(long, short, value_name = "FILE")]
    pub output_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct SegmentOptions {
    /// Minimum lookahead (in characters) required before finalizing a sentence.
    pub lookahead: usize,
    /// Maximum buffer length before potentially forcing a split or erroring.
    pub max_buffer: usize,
    /// Language code for segmentation rules.
    pub language: String,
    // Potentially store the loaded language object directly if desired
    // pub(crate) language_impl: &'static (dyn Language + Send + Sync),
}

impl Default for SegmentOptions {
    fn default() -> Self {
        // Corresponds to clap defaults
        Self {
            lookahead: 10,
            max_buffer: 8192,
            language: "en".to_string(),
            // language_impl: libtqsm::get_language("en").unwrap(), // Or load dynamically
        }
    }
}

impl From<CliArgs> for SegmentOptions {
    fn from(args: CliArgs) -> Self {
        Self {
            lookahead: args.lookahead,
            max_buffer: args.max_buffer,
            language: args.language,
            // language_impl: libtqsm::get_language(&args.language).unwrap_or_else(|_| { /* handle error or default */}),
        }
    }
}
