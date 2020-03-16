use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use shell_words;

use colmsg::dirs::PROJECT_DIRS;

pub fn config_file() -> PathBuf {
    env::var("COLMSG_CONFIG_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|config_path| config_path.is_file())
        .unwrap_or_else(|| PROJECT_DIRS.config_dir().join("config"))
}

pub fn get_args_from_config_file() -> Result<Vec<OsString>, shell_words::ParseError> {
    Ok(fs::read_to_string(config_file())
        .ok()
        .map(|content| get_args_from_str(&content))
        .transpose()?
        .unwrap_or_else(|| vec![]))
}

fn get_args_from_str(content: &str) -> Result<Vec<OsString>, shell_words::ParseError> {
    let args_par_line = content
        .split('\n')
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(|line| shell_words::split(line))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(args_par_line
        .iter()
        .flatten()
        .map(|line| line.into())
        .collect())
}
