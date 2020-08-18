use clap::App;
use clap::Arg;
use once_cell::sync::Lazy;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;
use std::vec::Vec;

pub struct Config {
    pub listen: SocketAddr,
    pub daemon: bool,
    pub site_control: SiteControl,
}

pub enum SiteControl {
    Disable,
    Allow(Vec<String>),
}

pub static CONFIG: Lazy<Config> = Lazy::new(parse_config);

fn command_config() -> App<'static, 'static> {
    App::new("SuperHacker's HTTP Proxy")
        .name(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("allow-all")
                .short("a")
                .long("allow-all")
                .help("disable site control")
                .takes_value(false)
                .multiple(false)
                .required(false),
        )
        .arg(
            Arg::with_name("allowed-site")
                .short("A")
                .long("allowed-site")
                .value_name("SITE.ALLOW")
                .help("site control, only accept ip/domains in SITE.ALLOW file")
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
            Arg::with_name("fail-log")
                .short("F")
                .long("fail-log")
                .value_name("PATH")
                .help("output failed request to PATH")
                .takes_value(true)
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
    let site_control = if matches.is_present("allow-all") {
        SiteControl::Disable
    } else {
        let list = match matches.value_of("allowed-site") {
            Some(file) => parse_sites(file.as_ref()).expect(&einfo("SITE.ALLOW")),
            None => vec![],
        };
        SiteControl::Allow(list)
    };
    Config {
        listen: listen,
        daemon: daemon,
        site_control: site_control,
    }
}

fn einfo(info: &str) -> String {
    ["Unable to parse ", info].concat()
}

fn parse_sites(conf: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut file = File::open(conf)?;
    let mut conf = String::new();
    file.read_to_string(&mut conf)?;
    let conf = conf
        .lines()
        .filter(|s| !s.starts_with("#") || s.trim().len() > 0)
        .map(|s| s.into())
        .collect();
    Ok(conf)
}
