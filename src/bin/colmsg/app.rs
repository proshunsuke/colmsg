use std::fs;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use clap::ArgMatches;
use wild;

use colmsg::{
    controller::Service,
    dirs::PROJECT_DIRS,
    errors::*,
    http::client::{AClient, HClient, MClient, NClient, SClient, SHNClient, YClient},
    Config, Kind,
};

use crate::{
    auth, clap_app, config::get_access_token_from_file, config::get_args_from_config_file,
};

pub struct App {
    pub matches: ArgMatches<'static>,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(App {
            matches: Self::matches()?,
        })
    }

    pub fn browser_auth(&self, service: Service) -> bool {
        auth::has_browser_login(service)
    }

    pub fn download_dir(&self) -> PathBuf {
        self.matches
            .value_of("dir")
            .map(PathBuf::from)
            .unwrap_or_else(|| PROJECT_DIRS.download_dir().to_path_buf())
    }

    fn matches() -> Result<ArgMatches<'static>> {
        let mut cli_args = wild::args_os();
        let executable = cli_args.next().unwrap();
        let cli_args = cli_args.collect::<Vec<_>>();
        let command = cli_args.first().and_then(|s| s.to_str());
        let mut args = if matches!(command, Some("login" | "auth")) {
            vec![]
        } else {
            get_args_from_config_file()?
        };

        args.insert(0, executable);
        args.extend(cli_args);

        Ok(clap_app::build_app().get_matches_from(args))
    }

    pub fn sakurazaka_config(
        &self,
        refresh_token: &str,
        force: bool,
    ) -> Result<Config<'_, SClient>> {
        let client = SClient::new();
        self.config(
            "s_refresh_token",
            refresh_token,
            client,
            Service::Sakurazaka,
            force,
        )
    }

    pub fn hinatazaka_config(
        &self,
        refresh_token: &str,
        force: bool,
    ) -> Result<Config<'_, HClient>> {
        let client = HClient::new();
        self.config(
            "h_refresh_token",
            refresh_token,
            client,
            Service::Hinatazaka,
            force,
        )
    }

    pub fn nogizaka_config(&self, refresh_token: &str, force: bool) -> Result<Config<'_, NClient>> {
        let client = NClient::new();
        self.config(
            "n_refresh_token",
            refresh_token,
            client,
            Service::Nogizaka,
            force,
        )
    }

    pub fn asukasaito_config(
        &self,
        refresh_token: &str,
        force: bool,
    ) -> Result<Config<'_, AClient>> {
        let client = AClient::new();
        self.config(
            "a_refresh_token",
            refresh_token,
            client,
            Service::Asukasaito,
            force,
        )
    }

    pub fn maishiraishi_config(
        &self,
        refresh_token: &str,
        force: bool,
    ) -> Result<Config<'_, MClient>> {
        let client = MClient::new();
        self.config(
            "m_refresh_token",
            refresh_token,
            client,
            Service::Maishiraishi,
            force,
        )
    }

    pub fn yodel_config(&self, refresh_token: &str, force: bool) -> Result<Config<'_, YClient>> {
        let client = YClient::new();
        self.config(
            "y_refresh_token",
            refresh_token,
            client,
            Service::Yodel,
            force,
        )
    }

    fn config<S: AsRef<str>, C: SHNClient>(
        &self,
        refresh_token_str: S,
        refresh_token: &str,
        client: C,
        service: Service,
        force: bool,
    ) -> Result<Config<'_, C>> {
        let name = match self.matches.values_of("name") {
            Some(names) => names.map(|name| name.trim()).collect::<Vec<_>>(),
            None => vec![],
        };

        let from = self
            .matches
            .value_of("from")
            .map(|from| NaiveDateTime::parse_from_str(from, "%Y/%m/%d %H:%M:%S"));
        let from = match from {
            Some(Ok(t)) => Some(t),
            Some(Err(e)) => return Err(e.into()),
            None => None,
        };

        let kind = match self.matches.values_of("kind") {
            Some(k) => {
                k.map(|v| {
                    match v {
                        "text" => Kind::Text,
                        "picture" => Kind::Picture,
                        "video" => Kind::Video,
                        "voice" => Kind::Voice,
                        "link" => Kind::Link,
                        _ => Kind::Link, // _ はあり得ないはずだが怒られるのでとりあえずLinkにする
                    }
                })
                .collect::<Vec<_>>()
            }
            None => vec![
                Kind::Text,
                Kind::Picture,
                Kind::Video,
                Kind::Voice,
                Kind::Link,
            ],
        };

        let dir = self.download_dir();
        if !dir.is_dir() {
            if let Err(e) = fs::create_dir_all(&dir) {
                return Err(e.into());
            }
        }

        let token_file = refresh_token_str
            .as_ref()
            .replace("refresh_token", "access_token");
        let access_token = if self.browser_auth(service) {
            auth::token(service, force)?
        } else {
            get_access_token_from_file(&refresh_token.to_owned(), client.clone(), &token_file)?
        };
        let client = if self.browser_auth(service) {
            let (base_url, headers) = auth::endpoint(service)?;
            client.with_web_endpoint(base_url, headers)
        } else {
            client
        };
        Ok(Config {
            name,
            from,
            kind,
            dir,
            client: client.clone(),
            access_token,
        })
    }
}
