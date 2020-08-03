use clap::App;
use clap::Arg;
use once_cell::sync::Lazy;
use std::net::SocketAddr;

pub struct Config {
    pub listen: SocketAddr,
    pub daemon: bool,
}

pub static CONFIG: Lazy<Config> = Lazy::new(parse_config);

fn command_config() -> App<'static, 'static> {
    App::new("SuperHacker's HTTP Proxy")
        .name(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .value_name("IPADDR:PORT")
                .help("listen on IPADDR:PORT, default to 0.0.0.0:32767, IPv6 supported")
                .takes_value(true)
                .multiple(false)
                .required(false),
        )
        .arg(
            Arg::with_name("daemon")
                .short("d")
                .long("daemon")
                .help("start in daemon mode")
                .takes_value(false)
                .multiple(false)
                .required(false),
        )
}

fn parse_config() -> Config {
    let matches = command_config().get_matches();
    let listen = matches
        .value_of("listen")
        .unwrap_or("0.0.0.0:32767")
        .parse()
        .expect(&einfo("IPADDR:PORT"));
    let daemon = matches.is_present("daemon");
    Config {
        listen: listen,
        daemon: daemon,
    }
}

fn einfo(info: &str) -> String {
    ["Unable to parse ", info].concat()
}
