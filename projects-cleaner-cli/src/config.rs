#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct Config {
    pub extensions: Vec<String>,
}

impl Config {
    pub fn new(add: Vec<String>, remove: Vec<String>) -> Self {
        let path = std::path::Path::new("./config.toml");
        let mut conf = match std::fs::File::options().read(true).open(path) {
            Ok(file) => match serde_json::de::from_reader(file) {
                Ok(deserialized) => deserialized,
                Err(err) => {
                    eprintln!("deserialise error: {}", err);
                    Config { extensions: vec![] }
                }
            },
            Err(_) => Config { extensions: vec![] },
        };
        for extension in add {
            if !extension.starts_with(".") {
                eprintln!("extension '{}' doesn't start with a '.'", extension);
            } else if conf.extensions.contains(&extension) {
                eprintln!("extension '{}' is already in the config", extension);
            } else {
                conf.extensions.push(extension);
            }
        }
        for extension in remove {
            if !extension.starts_with(".") {
                eprintln!("extension '{}' doesn't start with a '.'", extension);
            } else if conf.extensions.contains(&extension) {
                conf.extensions.retain(|ext| ext == &extension);
            } else {
                eprintln!("extension '{}' not found in the config", extension);
            }
        }
        conf
    }

    pub fn save(self) -> Result<(), std::io::Error> {
        let path = std::path::Path::new("./config.toml");
        let file = std::fs::File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        match serde_json::ser::to_writer(&file, &self) {
            Ok(_) => Ok(()),
            Err(err) => Err(std::io::Error::other(format!("Serialising error: {}", err))),
        }
    }
}
