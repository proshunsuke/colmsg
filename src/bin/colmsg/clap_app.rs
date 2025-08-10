use clap::{App as ClapApp, Arg, AppSettings};

pub fn build_app() -> ClapApp<'static, 'static> {
    ClapApp::new(crate_name!())
        .version(crate_version!())
        .global_setting(AppSettings::ColoredHelp)
        .about(
            "A CLI tool for '櫻坂46メッセージ', '日向坂46メッセージ', '乃木坂46メッセージ', '齋藤飛鳥メッセージ', '白石麻衣メッセージ', and 'yodel' app.\n\n\
             Use '--help' instead of '-h' to see a more detailed version of the help text.",
        )
        .long_about("A CLI tool for saving messages of '櫻坂46メッセージ', '日向坂46メッセージ', '乃木坂46メッセージ', '齋藤飛鳥メッセージ', '白石麻衣メッセージ', and 'yodel' app locally.")
        .arg(
            Arg::with_name("group")
                .long("group")
                .short("g")
                .multiple(true)
                .possible_values(&["sakurazaka", "hinatazaka", "nogizaka", "asukasaito", "maishiraishi", "yodel"])
                .help("Save messages of specific group.")
                .long_help("Save messages of specific group.
If not specified, save messages both of groups")
                .takes_value(true),

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
                .possible_values(&["text", "picture", "video", "voice"])
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
                .help("Set the sakurazaka refresh token.")
                .long_help("Set the sakurazaka refresh token.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("h_refresh_token")
                .long("h_refresh_token")
                .help("Set the hinatazaka refresh token.")
                .long_help("Set the hinatazaka refresh token.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("n_refresh_token")
                .long("n_refresh_token")
                .help("Set the nogizaka refresh token.")
                .long_help("Set the nogizaka refresh token.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("a_refresh_token")
                .long("a_refresh_token")
                .help("Set the asukasaito refresh token.")
                .long_help("Set the asukasaito refresh token.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("m_refresh_token")
                .long("m_refresh_token")
                .help("Set the maishiraishi refresh token.")
                .long_help("Set the maishiraishi refresh token.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("y_refresh_token")
                .long("y_refresh_token")
                .help("Set the yodel refresh token.")
                .long_help("Set the yodel refresh token.")
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
            Arg::with_name("config-dir")
                .long("config-dir")
                .help("Show colmsg's default configuration directory.")
        )
        .arg(
            Arg::with_name("download-dir")
                .long("download-dir")
                .help("Show colmsg's default download directory.")
        )
        .help_message("Print this help message.")
        .version_message("Show version information.")
}
