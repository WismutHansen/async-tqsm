use crate::config::SegmentOptions;
use crate::error::{Result, SegmenterError};
use libtqsm::{get_language, GraphemeCursor, Language}; // Language trait is now needed
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation; // Add this line

pub struct Segmenter {
    buffer: String,
    options: SegmentOptions,
    language: &'static (dyn Language + Send + Sync),
}

impl Segmenter {
    pub fn new(options: SegmentOptions) -> Result<Self> {
        let language_impl = get_language(&options.language)
            .ok_or_else(|| SegmenterError::UnsupportedLanguage(options.language.clone()))?;

        Ok(Self {
            buffer: String::with_capacity(options.max_buffer / 4),
            options,
            language: language_impl,
        })
    }

    pub fn feed(&mut self, chunk: &str) -> Result<Vec<String>> {
        if self.buffer.len() + chunk.len() > self.options.max_buffer {
            return Err(SegmenterError::BufferOverflow(self.options.max_buffer));
        }
        self.buffer.push_str(chunk);
        self.process_buffer()
    }

    fn process_buffer(&mut self) -> Result<Vec<String>> {
        let mut completed_sentences = Vec::new();
        let mut current_offset = 0;

        loop {
            let buffer_len = self.buffer.len();
            if current_offset >= buffer_len {
                break;
            }

            let mut boundary_found_in_iteration = false;
            let remaining_buffer_slice = &self.buffer[current_offset..];

            // --- Create GraphemeCursor locally ---
            let grapheme_indices: HashMap<usize, &str> =
                remaining_buffer_slice.grapheme_indices(false).collect();
            let mut grapheme_offsets: Vec<usize> = grapheme_indices.keys().copied().collect();
            grapheme_offsets.sort_unstable();
            // Use the public constructor (assuming you added `pub fn new(...)` to libtqsm)
            let cursor = GraphemeCursor::new(grapheme_offsets);
            // ---

            let skippable_ranges = self.language.get_skippable_ranges(remaining_buffer_slice);
            let mut best_boundary: Option<(usize, usize)> = None; // (relative_pos, absolute_pos_in_buffer)

            for mtch in self
                .language
                .sentence_break_regex()
                .find_iter(remaining_buffer_slice)
            {
                let (match_start, match_end) = (mtch.start(), mtch.end());

                // --- Handle skippable ranges *before* calling find_boundary ---
                let mut in_range = false;
                for (skip_start, skip_end) in skippable_ranges.iter() {
                    if match_start >= *skip_start && match_end <= *skip_end {
                        if match_end == *skip_end && self.language.is_punctuation_between_quotes() {
                            // It's the closing punctuation of a skippable range
                            // Treat this match end as the potential boundary point
                            best_boundary = Some((*skip_end, current_offset + *skip_end));
                            boundary_found_in_iteration = true;
                            in_range = true; // Mark as handled within range logic
                            break; // Process this boundary
                        } else {
                            // Boundary is fully inside skip range, ignore it
                            in_range = true;
                            break; // Stop checking ranges for this match
                        }
                    }
                }
                if in_range {
                    if best_boundary.is_some() {
                        break;
                    }
                    // Break outer loop if boundary was found
                    else {
                        continue;
                    } // Continue to next match if ignored
                }
                // --- End skippable range handling ---

                // Call the original find_boundary from the trait
                // Ensure grapheme_indices are mapped correctly if necessary (here assumed relative to slice)
                if let Some((relative_boundary_end, is_num_ref)) = self.language.find_boundary(
                    remaining_buffer_slice,
                    &grapheme_indices,
                    &cursor,
                    mtch,
                ) {
                    let absolute_boundary_end = current_offset + relative_boundary_end;

                    if is_num_ref || buffer_len >= absolute_boundary_end + self.options.lookahead {
                        best_boundary = Some((relative_boundary_end, absolute_boundary_end));
                        boundary_found_in_iteration = true;
                        break; // Process this boundary
                    } else {
                        boundary_found_in_iteration = false;
                        break; // Need more input
                    }
                }
            }

            if let Some((_relative_end, absolute_end)) = best_boundary {
                let sentence = self.buffer[..absolute_end].to_string();
                completed_sentences.push(sentence.trim_matches(' ').to_string());
                self.buffer.drain(..absolute_end);
                current_offset = 0;
                continue;
            }

            if !boundary_found_in_iteration && best_boundary.is_none() {
                break;
            }
            if completed_sentences.is_empty() && best_boundary.is_none() {
                break;
            }
        }

        Ok(completed_sentences)
    }

    pub fn flush(&mut self) -> Result<Option<String>> {
        if self.buffer.is_empty() {
            Ok(None)
        } else {
            let last_sentence = std::mem::take(&mut self.buffer);
            Ok(Some(last_sentence.trim_matches(' ').to_string()))
        }
    }
}
