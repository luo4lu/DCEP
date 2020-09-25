use clap::{App, Arg, ArgMatches};

pub fn get_command() -> ArgMatches<'static> {
    App::new("digital currency transaction system")
        .version("0.1.0")
        .author("luo4lu <luo4lu@163.com>")
        .about("config myself ip addr and port and database config")
        .arg(
            Arg::with_name("dcdt")
                .short("t")
                .long("dcdt")
                .help("set self DCD system IP addr and port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("database")
                .help("set database addr")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("username")
                .short("n")
                .long("username")
                .help("set database username")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("basename")
                .short("b")
                .long("basename")
                .help("set database base name")
                .takes_value(true),
        )
        .get_matches()
}
