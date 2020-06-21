use clap::{App as ClapApp, Arg, AppSettings};

pub fn build_app() -> ClapApp<'static, 'static> {
    ClapApp::new(crate_name!())
        .version(crate_version!())
        .global_setting(AppSettings::ColoredHelp)
        .about(
            "A CLI tool for '欅坂/日向坂メッセージアプリ'.\n\n\
             Use '--help' instead of '-h' to see a more detailed version of the help text.",
        )
        .long_about("A CLI tool for saving messages of '欅坂/日向坂メッセージアプリ' locally.")
        .arg(
            Arg::with_name("group")
                .long("group")
                .short("g")
                .possible_values(&["keyakizaka", "hinatazaka"])
                .help("Save messages of specific group.")
                .long_help("Save messages of specific group.
If not specified, save messages both of groups")
                .takes_value(true),

        )
        .arg(
            Arg::with_name("name")
                .long("name")
                .short("n")
                .help("Save messages of specific members (菅井友香,佐々木久美,..)")
                .long_help("Save messages of specific members (菅井友香,佐々木久美,..)
Name must be a valid full name of kanji.
If not specified, save messages of all members.
e.g. -n 菅井友香 -n 佐々木久美.")
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
            Arg::with_name("refresh_token")
                .long("refresh_token")
                .short("t")
                .required(true)
                .help("Set the refresh token.")
                .long_help("Set the refresh token. refresh token is required.")
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
