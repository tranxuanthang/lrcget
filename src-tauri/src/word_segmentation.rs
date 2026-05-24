use charabia::Segment;

pub fn segment_words_for_timing(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let raw_segments: Vec<String> = text
        .segment_str()
        .filter(|segment| !segment.is_empty())
        .map(str::to_owned)
        .collect();

    merge_segment_tokens(raw_segments)
}

fn merge_segment_tokens(raw_segments: Vec<String>) -> Vec<String> {
    // Charabia can emit standalone whitespace/punctuation for some texts.
    // Merge those separators with neighboring word tokens so timing lanes do not
    // create standalone "word" blocks for spaces or punctuation.
    let mut merged_tokens: Vec<String> = Vec::new();
    let mut pending_prefix = String::new();

    let mut index = 0;
    while index < raw_segments.len() {
        let segment = &raw_segments[index];
        let is_separator_only = segment
            .chars()
            .all(|ch| ch.is_whitespace() || !ch.is_alphanumeric());

        if is_separator_only {
            let has_prev_word = merged_tokens
                .last()
                .is_some_and(|token| token.chars().any(char::is_alphanumeric));
            let next_is_word = raw_segments
                .get(index + 1)
                .is_some_and(|next| next.chars().any(char::is_alphanumeric));

            // Keep contractions like don't / can't / l'amour as a single token.
            if is_apostrophe_connector(segment) && has_prev_word && next_is_word {
                if let Some(last) = merged_tokens.last_mut() {
                    last.push_str(segment);
                    index += 1;
                    last.push_str(&raw_segments[index]);
                }
                index += 1;
                continue;
            }

            // Opening brackets should prefix the next token, not suffix the previous one.
            if is_opening_bracket_connector(segment) && next_is_word {
                pending_prefix.push_str(segment);
                index += 1;
                continue;
            }

            if let Some(last) = merged_tokens.last_mut() {
                last.push_str(segment);
            } else {
                pending_prefix.push_str(segment);
            }
            index += 1;
            continue;
        }

        if pending_prefix.is_empty() {
            merged_tokens.push(segment.to_owned());
        } else {
            let mut combined = pending_prefix.clone();
            combined.push_str(segment);
            merged_tokens.push(combined);
            pending_prefix.clear();
        }

        index += 1;
    }

    if !pending_prefix.is_empty() {
        if let Some(last) = merged_tokens.last_mut() {
            last.push_str(&pending_prefix);
        } else {
            merged_tokens.push(pending_prefix);
        }
    }

    merged_tokens
}

fn is_apostrophe_connector(segment: &str) -> bool {
    !segment.is_empty()
        && segment.chars().all(|ch| matches!(ch, '\'' | '’' | 'ʼ'))
}

fn is_opening_bracket_connector(segment: &str) -> bool {
    let mut has_open_bracket = false;

    for ch in segment.chars() {
        if ch.is_whitespace() {
            continue;
        }

        if matches!(
            ch,
            '('
                | '['
                | '{'
                | '<'
                | '（'
                | '［'
                | '｛'
                | '〈'
                | '《'
                | '「'
                | '『'
                | '【'
                | '〔'
                | '〖'
                | '〘'
                | '〚'
                | '⟨'
                | '⟪'
                | '⟮'
                | '⦗'
                | '⸢'
        ) {
            has_open_bracket = true;
            continue;
        }

        return false;
    }

    has_open_bracket
}

#[cfg(test)]
mod tests {
    use super::segment_words_for_timing;

    fn contains_alnum(text: &str) -> bool {
        text.chars().any(char::is_alphanumeric)
    }

    fn is_separator_only(text: &str) -> bool {
        text.chars()
            .all(|ch| ch.is_whitespace() || !ch.is_alphanumeric())
    }

    fn assert_segmentation_invariants(text: &str) {
        let merged = segment_words_for_timing(text);
        let reconstructed = merged.concat();

        assert_eq!(
            reconstructed, text,
            "Merged tokens must reconstruct original input"
        );

        if contains_alnum(text) {
            assert!(
                merged.iter().all(|token| !is_separator_only(token)),
                "No separator-only tokens should remain when text contains word characters"
            );
        }
    }

    #[test]
    fn english_with_punctuation_and_spaces_has_no_separator_tokens() {
        assert_segmentation_invariants("Hello, world!  It's me.");
    }

    #[test]
    fn japanese_text_preserves_reconstruction() {
        assert_segmentation_invariants("今日は、世界！ テスト中です。");
    }

    #[test]
    fn arabic_text_preserves_reconstruction() {
        assert_segmentation_invariants("مرحبا، بالعالم! كيف الحال؟");
    }

    #[test]
    fn mixed_emoji_and_symbols_preserves_reconstruction() {
        assert_segmentation_invariants("Hi 👋 -- version 2.0 (beta) ✨");
    }

    #[test]
    fn english_contractions_remain_single_tokens() {
        let merged = segment_words_for_timing("don't can't won't");
        assert_eq!(merged, vec!["don't ", "can't ", "won't"]);
    }

    #[test]
    fn curly_apostrophe_contractions_remain_single_tokens() {
        let merged = segment_words_for_timing("don’t can’t l’amour");
        assert_eq!(merged, vec!["don’t ", "can’t ", "l’amour"]);
    }

    #[test]
    fn opening_brackets_attach_to_next_token() {
        let merged = segment_words_for_timing("hello (world) [test] {ok}");
        assert_eq!(merged, vec!["hello ", "(world) ", "[test] ", "{ok}"]);
    }
}
