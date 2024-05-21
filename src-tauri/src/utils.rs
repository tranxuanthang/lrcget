use regex::Regex;

pub fn strip_timestamp(synced_lyrics: &str) -> String {
  let re = Regex::new(r"^\[(.*)\] *").unwrap();
  let plain_lyrics = re.replace_all(synced_lyrics, "");
  plain_lyrics.to_string()
}
