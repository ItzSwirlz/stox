use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::{fs::DirBuilderExt, prelude::OpenOptionsExt},
    path::{Path, PathBuf},
};

const SAVED_STOCKS_FILENAME: &str = "saved-stocks.toml";
const SAVED_STOCKS_VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
struct SavedStocks {
    version: u8,
    symbols: Vec<String>,
}

fn persistence_disabled() -> bool {
    #[cfg(not(unix))]
    return true;

    match env::var("STOX_NO_PERSISTENCE") {
        Ok(env_var) => env_var == "1",
        Err(_) => false,
    }
}

fn get_persistence_path() -> Result<PathBuf, anyhow::Error> {
    let mut data_home = env::var("XDG_DATA_HOME");
    if data_home.is_err() {
        let home = env::var("HOME")?;
        match Path::new(&home).join(".local/share").to_str() {
            Some(path) => data_home = Ok(path.to_string()),
            None => return Err(anyhow::anyhow!("could not get data home")),
        }
    }

    let data_home = data_home.unwrap();

    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(data_home.clone())?;

    let path = Path::new(&data_home).join("stox");

    if let Err(err) = DirBuilder::new().mode(0o755).create(&path) {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(err.into());
        }
    }

    Ok(path)
}

pub fn write_saved_stocks(symbols: Vec<String>) -> Result<(), anyhow::Error> {
    if persistence_disabled() {
        return Ok(());
    }

    let path = get_persistence_path()?.join(SAVED_STOCKS_FILENAME);

    let saved_stocks = SavedStocks {
        version: SAVED_STOCKS_VERSION,
        symbols,
    };

    let toml_data = toml::to_string(&saved_stocks)?;

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .mode(0o600)
        .open(path)?;

    file.write_all(toml_data.as_bytes())?;

    Ok(())
}

pub fn read_saved_stocks() -> Result<Vec<String>, anyhow::Error> {
    if persistence_disabled() {
        return Ok(vec![]);
    }

    let path = get_persistence_path()?.join(SAVED_STOCKS_FILENAME);

    match fs::read_to_string(path) {
        Ok(toml_data) => {
            let saved_stocks: SavedStocks = toml::from_str(&toml_data)?;
            if saved_stocks.version != SAVED_STOCKS_VERSION {
                return Err(anyhow::anyhow!("unknown file version"));
            }

            Ok(saved_stocks.symbols)
        }
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                write_saved_stocks(vec![])?;

                Ok(vec![])
            } else {
                Err(err.into())
            }
        }
    }
}
