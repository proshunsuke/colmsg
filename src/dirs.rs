use std::path::{Path, PathBuf};
use crate::dirs_rs;

#[cfg(target_os = "macos")]
use std::env;

pub struct ColmsgProjectDirs {
    config_dir: PathBuf,
    download_dir: PathBuf
}

impl ColmsgProjectDirs {
    fn new() -> Option<ColmsgProjectDirs> {
        #[cfg(target_os = "macos")]
            let config_dir_op = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| dirs_rs::home_dir().map(|d| d.join(".config")));

        #[cfg(not(target_os = "macos"))]
        let config_dir_op = dirs_rs::config_dir();
        let config_dir = config_dir_op.map(|d| d.join("colmsg"))?;

        let download_dir_op = dirs_rs::download_dir();
        let download_dir = download_dir_op.map(|d| d.join("colmsg"))?;

        Some(ColmsgProjectDirs {
            config_dir,
            download_dir
        })
    }
    pub fn config_dir(&self) -> &Path { &self.config_dir }
    pub fn download_dir(&self) -> &Path { &self.download_dir }
}

lazy_static! {
    pub static ref PROJECT_DIRS: ColmsgProjectDirs =
        ColmsgProjectDirs::new().expect("Could not get home directory");
}
