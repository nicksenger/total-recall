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

pub fn get_image_from_google(
  language_abbr: &str,
  word: &str,
  sanitized: &str,
) -> Result<(), String> {
  if Path::new(&format!(
    "./static/images/{}/{}.jpg",
    language_abbr, sanitized
  ))
  .exists()
  {
    return Ok(());
  }

  reqwest::get(&format!(
    "https://www.google.com/search?q={}&tbm=isch&tbs=ift:jpg",
    word
  ))
  .map_err(|err| err.to_string())
  .and_then(|mut search_response| search_response.text().map_err(|err| err.to_string()))
  .and_then(|text| {
    Document::from_read(::std::io::Cursor::new(text.into_bytes()))
      .map_err(|err| err.to_string())
      .and_then(|document| {
        document
          .find(And(
            Attr("style", "border:1px solid #ccc;padding:1px"),
            Name("img"),
          ))
          .filter_map(|n| n.attr("src"))
          .nth(0)
          .ok_or("No image found in document!".to_owned())
          .and_then(|url| reqwest::get(url).map_err(|err| err.to_string()))
      })
  })
  .and_then(|source| {
    if !Path::new(&format!("./static/images/{}", language_abbr)).exists() {
      create_dir(&format!("./static/images/{}", language_abbr)).map_err(|err| err.to_string())?;
    }
    File::create(format!(
      "./static/images/{}/{}.jpg",
      language_abbr, sanitized
    ))
    .map_err(|err| err.to_string())
    .map(|dest| (source, dest))
  })
  .and_then(|(mut source, mut dest)| copy(&mut source, &mut dest).map_err(|err| err.to_string()))
  .and_then(|_| Ok(()))
}

pub fn get_audio_from_google(
  language_abbr: &str,
  word: &str,
  sanitized: &str,
) -> Result<(), String> {
  if Path::new(&format!(
    "./static/audio/{}/{}.mp3",
    language_abbr, sanitized
  ))
  .exists()
  {
    return Ok(());
  }

  let url = google_translate_tts::url(word, language_abbr);
  reqwest::get(&url)
    .map_err(|err| err.to_string())
    .and_then(|source| {
      if !Path::new(&format!("./static/audio/{}", language_abbr)).exists() {
        create_dir(&format!("./static/audio/{}", language_abbr)).map_err(|err| err.to_string())?;
      }
      File::create(format!(
        "./static/audio/{}/{}.mp3",
        language_abbr, sanitized
      ))
      .map_err(|err| err.to_string())
      .map(|dest| (source, dest))
    })
    .and_then(|(mut source, mut dest)| copy(&mut source, &mut dest).map_err(|err| err.to_string()))
    .and_then(|_| Ok(()))
}
