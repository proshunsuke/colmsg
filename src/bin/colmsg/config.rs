use std::env;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use shell_words;

use colmsg::dirs::PROJECT_DIRS;
use colmsg::{errors::*, http};

pub fn config_file() -> PathBuf {
    env::var("COLMSG_CONFIG_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|config_path| config_path.is_file())
        .unwrap_or_else(|| PROJECT_DIRS.config_dir().join("config"))
}

pub fn get_args_from_config_file() -> Result<Vec<OsString>> {
    Ok(fs::read_to_string(config_file())
        .ok()
        .map(|content| get_args_from_str(&content))
        .transpose()?
        .unwrap_or_else(|| vec![]))
}

fn get_args_from_str(content: &str) -> Result<Vec<OsString>> {
    let args_par_line = content
        .split('\n')
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(|line| shell_words::split(line))
        .collect::<Vec<_>>();
    Ok(args_par_line
        .iter()
        .flatten()
        .flatten()
        .map(|line| line.into())
        .collect::<Vec<_>>())
}

pub fn get_access_token_from_file(refresh_token: &String) -> Result<String> {
    let dir = PROJECT_DIRS.config_dir().to_path_buf();
    if !dir.is_dir() { fs::create_dir_all(&dir)? };
    let file = dir.join("access_token");
    if file.is_file() { return Ok(fs::read_to_string(file)?); }
    let update_token_res = http::update_token::request(refresh_token)?;
    let mut f = File::create(file)?;
    f.write_all(update_token_res.access_token.as_ref())?;
    Ok(update_token_res.access_token)
}

pub fn delete_access_token_file() -> Result<()> {
    let dir = PROJECT_DIRS.config_dir().to_path_buf();
    let file = dir.join("access_token");
    if !file.is_file() { return Ok(()); }
    Ok(fs::remove_file(file)?)
}
