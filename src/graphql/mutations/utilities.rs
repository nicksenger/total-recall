use google_translate_tts;
use reqwest;
use select::{
  document::Document,
  predicate::{And, Attr, Name},
};
use std::{
  fs::{create_dir, File},
  io::copy,
  path::Path,
};

#[derive(Debug)]
pub enum TRCError {
  Request(reqwest::Error),
  FileSystem(std::io::Error),
  Unknown(String),
}

pub fn get_image_from_google(
  language_abbr: &str,
  word: &str,
  sanitized: &str,
) -> Result<(), TRCError> {
  if Path::new(&format!(
    "./static/images/{}/{}.jpg",
    language_abbr, sanitized
  ))
  .exists()
  {
    return Ok(());
  }

  let mut search_response = reqwest::get(&format!(
    "https://www.google.com/search?q={}&tbm=isch&tbs=ift:jpg",
    word
  ))
  .map_err(TRCError::Request)?;

  let text = search_response.text().map_err(TRCError::Request)?;
  let mut source = Document::from_read(::std::io::Cursor::new(text.into_bytes()))
    .map_err(TRCError::FileSystem)
    .and_then(|document| {
      document
        .find(And(
          Attr("style", "border:1px solid #ccc;padding:1px"),
          Name("img"),
        ))
        .filter_map(|n| n.attr("src"))
        .nth(0)
        .ok_or(TRCError::Unknown(
          "Failed to find first image in document".to_owned(),
        ))
        .and_then(|url| reqwest::get(url).map_err(TRCError::Request))
    })?;
  if !Path::new(&format!("./static/images/{}", language_abbr)).exists() {
    create_dir(&format!("./static/images/{}", language_abbr)).map_err(TRCError::FileSystem)?;
  }

  let mut dest = File::create(format!(
    "./static/images/{}/{}.jpg",
    language_abbr, sanitized
  ))
  .map_err(TRCError::FileSystem)?;

  copy(&mut source, &mut dest).map_err(TRCError::FileSystem)?;

  Ok(())
}

pub fn get_audio_from_google(
  language_abbr: &str,
  word: &str,
  sanitized: &str,
) -> Result<(), TRCError> {
  if Path::new(&format!(
    "./static/audio/{}/{}.mp3",
    language_abbr, sanitized
  ))
  .exists()
  {
    return Ok(());
  }

  let url = google_translate_tts::url(word, language_abbr);
  let mut source = reqwest::get(&url).map_err(TRCError::Request)?;

  if !Path::new(&format!("./static/audio/{}", language_abbr)).exists() {
    create_dir(&format!("./static/audio/{}", language_abbr)).map_err(TRCError::FileSystem)?;
  }

  let mut dest = File::create(format!(
    "./static/audio/{}/{}.mp3",
    language_abbr, sanitized
  ))
  .map_err(TRCError::FileSystem)?;

  copy(&mut source, &mut dest).map_err(TRCError::FileSystem)?;

  Ok(())
}
