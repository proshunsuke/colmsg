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
                .help("save messages of specific group.")
                .long_help("save messages of specific group.
if not specified, save messages both of groups")
                .takes_value(true),

        )
        .arg(
            Arg::with_name("name")
                .long("name")
                .short("n")
                .help("save messages of specific members (菅井友香,佐々木久美,..)")
                .long_help("save messages of specific members (菅井友香,佐々木久美,..)
name must be a valid full name of kanji.
if not specified, save messages of all members.
e.g. -n 菅井友香 -n 佐々木久美.")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("from")
                .long("from")
                .short("F")
                .help("save messages after a specific date.")
                .long_help("save messages after a specific date.
date format is %Y/%m/%d %H:%M:%S
e.g. -F '2020/01/01/ 00:00:00'")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("to")
                .long("to")
                .short("T")
                .help("save messages before a specific date.")
                .long_help("save messages before a specific date.
date format is %Y/%m/%d %H:%M:%S
e.g. -T '2020/01/01/ 00:00:00'")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("kind")
                .long("kind")
                .short("k")
                .multiple(true)
                .possible_values(&["text", "image", "movie", "voice"])
                .help("save specific kind of messages.")
                .long_help("save specific kind of messages.
if not specified, save all kinds of messages.
e.g. -k text -k image")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dir")
                .long("dir")
                .short("d")
                .help("set a project directory.")
                .long_help("set a project directory.
default directory is ~/Documents/colmsg.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("username")
                .long("username")
                .short("u")
                .required(true)
                .help("set a username.")
                .long_help("set a username. username is required.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("token")
                .long("token")
                .short("t")
                .required(true)
                .help("set a token.")
                .long_help("set a token. token is required.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("delete")
                .long("delete")
                .help("delete all saved messages.")
                .long_help("delete all saved messages.
if you execute command with this option, all saved messages are deleted from your disk.
please use be careful."),
        )
}
