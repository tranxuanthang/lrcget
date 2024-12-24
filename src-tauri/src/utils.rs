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
    let re = Regex::new(r"^\[(.*)\] *").unwrap();
    let plain_lyrics = re.replace_all(synced_lyrics, "");
    plain_lyrics.to_string()
}
