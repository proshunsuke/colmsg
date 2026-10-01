use clap::{App as ClapApp, AppSettings, Arg, SubCommand};

pub fn build_app() -> ClapApp<'static, 'static> {
    ClapApp::new(crate_name!())
        .version(crate_version!())
        .global_setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .before_help("Recommended authentication (Cookie):\n  colmsg login hinatazaka\n  colmsg -g hinatazaka\n\nRefresh-token authentication is deprecated.")
        .about(
            "A CLI tool for '櫻坂46メッセージ', '日向坂46メッセージ', '乃木坂46メッセージ', '齋藤飛鳥メッセージ', '白石麻衣メッセージ', and 'yodel' app.\n\n\
             Use '--help' instead of '-h' to see a more detailed version of the help text.",
        )
        .long_about("A CLI tool for saving messages of '櫻坂46メッセージ', '日向坂46メッセージ', '乃木坂46メッセージ', '齋藤飛鳥メッセージ', '白石麻衣メッセージ', and 'yodel' app locally. Log in with `colmsg login SERVICE` to use recommended Cookie authentication.")
        .subcommand(SubCommand::with_name("login")
            .about("Register recommended Cookie authentication through the official website; does not download messages.")
            .arg(Arg::with_name("service").required(true).possible_values(crate::auth::SERVICES))
            .arg(Arg::with_name("browser").long("browser").takes_value(true).help("Chromium browser executable path (or COLMSG_BROWSER).")))
        .subcommand(SubCommand::with_name("auth")
            .about("Authentication information")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(SubCommand::with_name("status").about("Show locally stored authentication metadata, without secrets.")))
        .arg(
            Arg::with_name("group")
                .long("group")
                .short("g")
                .multiple(true)
                .possible_values(&["sakurazaka", "hinatazaka", "nogizaka", "asukasaito", "maishiraishi", "yodel"])
                .help("Save messages of specific group.")
                .long_help("Save messages of specific group.
If not specified, save messages from all services with configured authentication.")
                .takes_value(true),

        )
        .arg(
            Arg::with_name("jobs")
                .long("jobs")
                .short("j")
                .default_value("4")
                .validator(|value| match value.parse::<usize>() {
                    Ok(jobs) if jobs > 0 => Ok(()),
                    _ => Err("must be a positive integer".to_owned()),
                })
                .help("Concurrent member saves per service (default: 4).")
                .long_help("Set the number of members to save concurrently within each selected service. Selected services are run concurrently. Defaults to 4."),
        )
        .arg(
            Arg::with_name("name")
                .long("name")
                .short("n")
                .help("Save messages of specific members (菅井友香, 佐々木久美, 秋元真夏..)")
                .long_help("Save messages of specific members (菅井友香, 佐々木久美, 秋元真夏..)
Name must be a valid full name of kanji.
If not specified, save messages of all members.
e.g. -n 菅井友香 -n 佐々木久美 -n 秋元真夏.")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("from")
                .long("from")
                .short("F")
                .help("Save messages after the specific date.")
                .long_help("Save messages after the specific date.
Date format is %Y/%m/%d %H:%M:%S
e.g. -F '2020/01/01 00:00:00'")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("kind")
                .long("kind")
                .short("k")
                .multiple(true)
                .possible_values(&["text", "picture", "video", "voice", "link"])
                .help("Save specific kind of messages.")
                .long_help("Save specific kind of messages.
If not specified, save all kinds of messages.
e.g. -k text -k image")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dir")
                .long("dir")
                .short("d")
                .help("Set the download directory.")
                .long_help("Set the download directory.
Use '--download-dir' to confirm the default directory.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("s_refresh_token")
                .long("s_refresh_token")
                .hidden_short_help(true)
                .help("Deprecated: set the sakurazaka refresh token. Use colmsg login sakurazaka.")
                .long_help("Deprecated legacy authentication: set the sakurazaka refresh token. Recommended: colmsg login sakurazaka (Cookie authentication). Legacy use displays a warning.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("h_refresh_token")
                .long("h_refresh_token")
                .hidden_short_help(true)
                .help("Deprecated: set the hinatazaka refresh token. Use colmsg login hinatazaka.")
                .long_help("Deprecated legacy authentication: set the hinatazaka refresh token. Recommended: colmsg login hinatazaka (Cookie authentication). Legacy use displays a warning.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("n_refresh_token")
                .long("n_refresh_token")
                .hidden_short_help(true)
                .help("Deprecated: set the nogizaka refresh token. Use colmsg login nogizaka.")
                .long_help("Deprecated legacy authentication: set the nogizaka refresh token. Recommended: colmsg login nogizaka (Cookie authentication). Legacy use displays a warning.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("a_refresh_token")
                .long("a_refresh_token")
                .hidden_short_help(true)
                .help("Deprecated: set the asukasaito refresh token. Use colmsg login asukasaito.")
                .long_help("Deprecated legacy authentication: set the asukasaito refresh token. Recommended: colmsg login asukasaito (Cookie authentication). Legacy use displays a warning.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("m_refresh_token")
                .long("m_refresh_token")
                .hidden_short_help(true)
                .help("Deprecated: set the maishiraishi refresh token. Use colmsg login maishiraishi.")
                .long_help("Deprecated legacy authentication: set the maishiraishi refresh token. Recommended: colmsg login maishiraishi (Cookie authentication). Legacy use displays a warning.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("y_refresh_token")
                .long("y_refresh_token")
                .hidden_short_help(true)
                .help("Deprecated: set the yodel refresh token. Use colmsg login yodel.")
                .long_help("Deprecated legacy authentication: set the yodel refresh token. Recommended: colmsg login yodel (Cookie authentication). Legacy use displays a warning.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("delete")
                .long("delete")
                .help("Delete all saved messages.")
                .long_help("Delete all saved messages.
If you execute command with this option, all saved messages are deleted from your disk.
Please use be careful."),
        )
        .arg(
            Arg::with_name("config-path")
                .long("config-path")
                .help("Show the path to the configuration file used by colmsg.")
        )
        .arg(
            Arg::with_name("download-dir")
                .long("download-dir")
                .help("Show the effective download directory.")
                .long_help("Show the effective download directory, using '--dir' from the configuration file or command line when set.")
        )
        .help_message("Print this help message.")
        .version_message("Show version information.")
}
