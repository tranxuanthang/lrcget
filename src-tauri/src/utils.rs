use collapse::collapse;
use regex::Regex;
use secular::lower_lay_string;

pub fn prepare_input(input: &str) -> String {
    let mut prepared_input = lower_lay_string(&input);

    let re = Regex::new(r#"[`~!@#$%^&*()_|+\-=?;:",.<>\{\}\[\]\\\/]"#).unwrap();
    prepared_input = re.replace_all(&prepared_input, " ").to_string();

    let re = Regex::new(r#"['’]"#).unwrap();
    prepared_input = re.replace_all(&prepared_input, "").to_string();

    prepared_input = prepared_input.to_lowercase();
    prepared_input = collapse(&prepared_input);

    prepared_input
}

pub fn strip_timestamp(synced_lyrics: &str) -> String {
    let re = Regex::new(r"(?m)^\[[^\]]*\]\s*").unwrap();
    let plain_lyrics = re.replace_all(synced_lyrics, "");
    plain_lyrics.to_string()
}

/// Prepare search input by removing bracketed content and normalizing
/// Removes content inside () and [] brackets, then applies prepare_input
pub fn prepare_search_input(title: &str) -> String {
    // Remove content inside () and [] brackets (including the brackets)
    let re = Regex::new(r"[\(\[][^\)\]]*[\]\)]").unwrap();
    let cleaned = re.replace_all(title, "");

    // Apply standard prepare_input normalization
    prepare_input(&cleaned)
}

/// Build an FTS5 MATCH query from raw user input.
/// Normalizes the input and turns each token into a prefix query.
/// Example: "Love Way" -> "love* way*"
pub fn build_fts_query(input: &str) -> String {
    let normalized = prepare_input(input);
    normalized
        .split_whitespace()
        .map(|word| format!("{}*", word))
        .collect::<Vec<_>>()
        .join(" ")
}
