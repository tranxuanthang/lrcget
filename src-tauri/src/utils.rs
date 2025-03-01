use collapse::collapse;
use regex::Regex;
use secular::lower_lay_string;

pub fn prepare_input(input: &str) -> String {
    let mut prepared_input = lower_lay_string(input);

    let special_chars_re = Regex::new(r#"[`~!@#$%^&*()_|+\-=?;:",.<>\{\}\[\]\\\/]"#)
        .expect("Invalid regex pattern for special characters");
    prepared_input = special_chars_re
        .replace_all(&prepared_input, " ")
        .to_string();

    let apostrophe_re = Regex::new(r#"['']"#).expect("Invalid regex pattern for apostrophes");
    prepared_input = apostrophe_re.replace_all(&prepared_input, "").to_string();

    prepared_input = prepared_input.to_lowercase();
    prepared_input = collapse(&prepared_input);

    prepared_input
}

pub fn strip_timestamp(synced_lyrics: &str) -> String {
    let timestamp_re = Regex::new(r"^\[(.*)\] *").expect("Invalid regex pattern for timestamps");
    let plain_lyrics = timestamp_re.replace_all(synced_lyrics, "");
    plain_lyrics.to_string()
}

pub fn strip_timestamp_from_id(synced_lyrics: &str) -> String {
    let timestamp_re = Regex::new(r"^\[(.*)\] *").expect("Invalid regex pattern for timestamps");
    let plain_lyrics = timestamp_re.replace_all(synced_lyrics, "");
    plain_lyrics.to_string()
}
