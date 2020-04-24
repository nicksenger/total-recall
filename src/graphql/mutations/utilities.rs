use google_translate_tts;
use reqwest;
use select::{
  document::Document,
  predicate::{And, Attr, Name},
};
use std::{
  error, fmt,
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

impl fmt::Display for TRCError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      TRCError::Request(ref err) => write!(f, "Request error: {}", err),
      TRCError::FileSystem(ref err) => write!(f, "File system error: {}", err),
      TRCError::Unknown(ref err) => write!(f, "Unknown error: {}", err),
    }
  }
}

impl error::Error for TRCError {
  fn cause(&self) -> Option<&(dyn error::Error)> {
    match *self {
      TRCError::Request(ref err) => Some(err),
      TRCError::FileSystem(ref err) => Some(err),
      _ => None,
    }
  }
}

impl From<std::io::Error> for TRCError {
  fn from(err: std::io::Error) -> TRCError {
    TRCError::FileSystem(err)
  }
}

impl From<reqwest::Error> for TRCError {
  fn from(err: reqwest::Error) -> TRCError {
    TRCError::Request(err)
  }
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
  ))?;

  let text = search_response.text()?;
  let document = Document::from_read(::std::io::Cursor::new(text.into_bytes()))?;
  let url = document
    .find(And(
      Attr("style", "border:1px solid #ccc;padding:1px"),
      Name("img"),
    ))
    .filter_map(|n| n.attr("src"))
    .nth(0)
    .ok_or(TRCError::Unknown(
      "Failed to find first image in document".to_owned(),
    ))?;
  let mut source = reqwest::get(url)?;

  if !Path::new(&format!("./static/images/{}", language_abbr)).exists() {
    create_dir(&format!("./static/images/{}", language_abbr))?;
  }

  let mut dest = File::create(format!(
    "./static/images/{}/{}.jpg",
    language_abbr, sanitized
  ))?;

  copy(&mut source, &mut dest)?;

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
  let mut source = reqwest::get(&url)?;

  if !Path::new(&format!("./static/audio/{}", language_abbr)).exists() {
    create_dir(&format!("./static/audio/{}", language_abbr))?;
  }

  let mut dest = File::create(format!(
    "./static/audio/{}/{}.mp3",
    language_abbr, sanitized
  ))?;

  copy(&mut source, &mut dest)?;

  Ok(())
}
