CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username VARCHAR (50) UNIQUE NOT NULL,
  password VARCHAR (255) NOT NULL,
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL
);

CREATE TABLE languages (
  id SERIAL PRIMARY KEY,
  name VARCHAR (255) NOT NULL,
  abbreviation VARCHAR (50) UNIQUE NOT NULL
);

CREATE TABLE decks (
  id SERIAL PRIMARY KEY,
  name VARCHAR (255) NOT NULL,
  owner INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  language INT NOT NULL REFERENCES languages(id) ON DELETE CASCADE
);

CREATE TABLE backs (
  id SERIAL PRIMARY KEY,
  text TEXT NOT NULL,
  language INT NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
  audio TEXT,
  image TEXT
);

CREATE TABLE cards (
  id SERIAL PRIMARY KEY,
  created_at BIGINT NOT NULL,
  front TEXT NOT NULL,
  back INT NOT NULL REFERENCES backs(id) ON DELETE CASCADE,
  deck INT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
  link TEXT
);

CREATE TABLE scores (
  id SERIAL PRIMARY KEY,
  created_at BIGINT NOT NULL,
  card INT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  value SMALLINT NOT NULL
);

CREATE TABLE sets (
  id SERIAL PRIMARY KEY,
  created_at BIGINT NOT NULL,
  name TEXT NOT NULL,
  deck INT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
  owner INT NOT NULL REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE set_cards (
  id SERIAL PRIMARY KEY,
  card_id INT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  set_id INT NOT NULL REFERENCES sets(id) ON DELETE CASCADE
);

INSERT INTO languages (abbreviation, name)
  VALUES
    ('af', 'Afrikaans'),
    ('ar', 'Arabic'),
    ('bn', 'Bengali'),
    ('bs', 'Bosnian'),
    ('ca', 'Catalan'),
    ('cs', 'Czech'),
    ('cy', 'Welsh'),
    ('da', 'Danish'),
    ('de', 'German'),
    ('el', 'Greek'),
    ('en-au', 'English (Australia)'),
    ('en-ca', 'English (Canada)'),
    ('en-gb', 'English (UK)'),
    ('en-gh', 'English (Ghana)'),
    ('en-ie', 'English (Ireland)'),
    ('en-in', 'English (India)'),
    ('en-ng', 'English (Nigeria)'),
    ('en-nz', 'English (New Zealand)'),
    ('en-ph', 'English (Philippines)'),
    ('en-tz', 'English (Tanzania)'),
    ('en-uk', 'English (UK)'),
    ('en-us', 'English (US)'),
    ('en-za', 'English (South Africa)'),
    ('en', 'English'),
    ('eo', 'Esperanto'),
    ('es-es', 'Spanish (Spain)'),
    ('es-us', 'Spanish (United States)'),
    ('es', 'Spanish'),
    ('et', 'Estonian'),
    ('fi', 'Finnish'),
    ('fr-ca', 'French (Canada)'),
    ('fr-fr', 'French (France)'),
    ('fr', 'French'),
    ('gu', 'Gujarati'),
    ('hi', 'Hindi'),
    ('hr', 'Croatian'),
    ('hu', 'Hungarian'),
    ('hy', 'Armenian'),
    ('id', 'Indonesian'),
    ('is', 'Icelandic'),
    ('it', 'Italian'),
    ('ja', 'Japanese'),
    ('jw', 'Javanese'),
    ('km', 'Khmer'),
    ('kn', 'Kannada'),
    ('ko', 'Korean'),
    ('la', 'Latin'),
    ('lv', 'Latvian'),
    ('mk', 'Macedonian'),
    ('ml', 'Malayalam'),
    ('mr', 'Marathi'),
    ('my', 'Myanmar (Burmese)'),
    ('ne', 'Nepali'),
    ('nl', 'Dutch'),
    ('no', 'Norwegian'),
    ('pl', 'Polish'),
    ('pt-br', 'Portuguese (Brazil)'),
    ('pt-pt', 'Portuguese (Portugal)'),
    ('pt', 'Portuguese'),
    ('ro', 'Romanian'),
    ('ru', 'Russian'),
    ('si', 'Sinhala'),
    ('sk', 'Slovak'),
    ('sq', 'Albanian'),
    ('sr', 'Serbian'),
    ('su', 'Sundanese'),
    ('sv', 'Swedish'),
    ('sw', 'Swahili'),
    ('ta', 'Tamil'),
    ('te', 'Telugu'),
    ('th', 'Thai'),
    ('tl', 'Filipino'),
    ('tr', 'Turkish'),
    ('uk', 'Ukrainian'),
    ('ur', 'Urdu'),
    ('vi', 'Vietnamese'),
    ('zh-cn', 'Chinese (Mandarin/China)'),
    ('zh-tw', 'Chinese (Mandarin/Taiwan)');
