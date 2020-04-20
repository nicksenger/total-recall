use google_translate_tts;
use reqwest;
use select::{
  document::Document,
  predicate::{And, Attr, Name},
};
use std::{
  fs::{create_dir, File},
  io::{copy, Error, ErrorKind, Result},
  path::Path,
};

pub fn get_image_from_google(language_abbr: &str, word: &str, sanitized: &str) -> Result<()> {
  if Path::new(&format!(
    "./static/images/{}/{}.jpg",
    language_abbr, sanitized
  ))
  .exists()
  {
    return Ok(());
  }

  match reqwest::get(&format!(
    "https://www.google.com/search?q={}&tbm=isch&tbs=ift:jpg",
    word
  )) {
    Ok(mut search_response) => match search_response.text() {
      Ok(text) => match Document::from_read(::std::io::Cursor::new(text.into_bytes()))?
        .find(And(
          Attr("style", "border:1px solid #ccc;padding:1px"),
          Name("img"),
        ))
        .filter_map(|n| n.attr("src"))
        .nth(0)
      {
        Some(url) => match reqwest::get(url) {
          Ok(mut source) => {
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
          Err(_) => Err(Error::new(ErrorKind::Other, "Failed to retrieve image.")),
        },
        None => Err(Error::new(
          ErrorKind::Other,
          "Failed to determine image URL.",
        )),
      },
      Err(_) => Err(Error::new(ErrorKind::Other, "Image search failed.")),
    },
    Err(_) => Err(Error::new(
      ErrorKind::Other,
      "Failed to parse response body.",
    )),
  }
}

pub fn get_audio_from_google(language_abbr: &str, word: &str, sanitized: &str) -> Result<()> {
  if Path::new(&format!(
    "./static/audio/{}/{}.mp3",
    language_abbr, sanitized
  ))
  .exists()
  {
    return Ok(());
  }

  let url = google_translate_tts::url(word, language_abbr);
  match reqwest::get(&url) {
    Ok(mut source) => {
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
    Err(_) => Err(Error::new(ErrorKind::Other, "Failed to retrieve audio.")),
  }
}
