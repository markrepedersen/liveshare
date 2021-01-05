use {clap::Clap, serde::Deserialize, std::fs::read_to_string, toml::from_str};

#[derive(Clap)]
#[clap(version = "1.0", author = "Mark P. <markrepedersen@gmail.com>")]
struct Opts {
    /// Specifies the config file to use.
    /// - By default, this will look for "config.toml" in the project directory.
    /// - Any subsequent arguments will take precedence over the configuration file if it exists.
    #[clap(short, long)]
    config: Option<String>,

    /// Specifies the IP address for this node.
    #[clap(short, long)]
    addr: Option<String>,

    /// Specifies the client IP/port combinations.
    /// - Each combination must be of the form "<addr>:<port>".
    #[clap(short, long)]
    clients: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct Client {
    pub host: String,
    pub port: u16,
}

impl Client {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub fn parse(v: &String) -> Self {
        let split: Vec<&str> = v.split(":").collect();

        if split.len() < 2 {
            panic!("Error parsing config file.");
        }

        Client {
            host: split[0].to_string(),
            port: split[1]
                .parse()
                .expect("Error parsing port in config file."),
        }
    }
}

/// Represents the contents of a client's config file. Information within will include the following:
/// - A list of any other clients that this client knows about
#[derive(Deserialize)]
pub struct Config {
    pub addr: Client,
}

impl Config {
    fn parse_args(opts: Opts) -> Result<Config, Box<dyn std::error::Error>> {
        let addr = Client::parse(
            &opts
                .addr
                .expect("<addr> argument must be specified if no config file is given."),
        );

        Ok(Config { addr })
    }

    fn parse_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = read_to_string(path)?;
        Ok(from_str::<Config>(&contents).unwrap())
    }

    /// Parses the contents of a config file.
    pub fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        let opts: Opts = Opts::parse();
        match opts.config {
            Some(path) => Self::parse_file(&path),
            None => Self::parse_args(opts),
        }
    }
}
