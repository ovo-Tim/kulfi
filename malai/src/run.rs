use eyre::Context;
use eyre::ContextCompat;
use eyre::eyre;
use serde::Deserialize;
use serde::de;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt as stdfmt;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_malai_conf")]
    malai: MalaiConf,
    http: Option<HttpServices>,
    tcp: Option<TcpServices>,
    udp: Option<UdpServices>,
    tcp_udp: Option<TcpUdpServices>,
}

#[derive(Deserialize, Debug)]
struct MalaiConf {
    log: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct IdentityConf {
    identity: Option<String>,
    secret_file: Option<String>,
}

#[derive(Deserialize, Debug)]
struct HttpServices {
    #[allow(dead_code)]
    #[serde(flatten)]
    services: HashMap<String, HttpServiceConf>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct HttpServiceConf {
    #[serde(flatten)]
    identity_conf: IdentityConf, // Leave None to read from env, .malai.secret-key file or .malai.id52 file and system keyring
    #[serde(alias = "ports", deserialize_with = "deserialize_ports")]
    port: Vec<u16>,
    public: bool,
    active: bool,
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_bridge")]
    bridge: String,
}

#[derive(Deserialize, Debug)]
struct TcpServices {
    #[allow(dead_code)]
    #[serde(flatten)]
    services: HashMap<String, TcpServiceConf>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct TcpServiceConf {
    #[serde(flatten)]
    identity_conf: IdentityConf, // Leave None to read from env, .malai.secret-key file or .malai.id52 file and system keyring
    #[serde(alias = "ports", deserialize_with = "deserialize_ports")]
    port: Vec<u16>,
    public: bool,
    active: bool,
    #[serde(default = "default_host")]
    host: String,
}

#[derive(Deserialize, Debug)]
struct UdpServices {
    #[allow(dead_code)]
    #[serde(flatten)]
    services: HashMap<String, UdpServiceConf>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct UdpServiceConf {
    #[serde(flatten)]
    identity_conf: IdentityConf,
    #[serde(alias = "ports", deserialize_with = "deserialize_ports")]
    port: Vec<u16>,
    public: bool,
    active: bool,
    #[serde(default = "default_host")]
    host: String,
}

#[derive(Deserialize, Debug)]
struct TcpUdpServices {
    #[allow(dead_code)]
    #[serde(flatten)]
    services: HashMap<String, TcpUdpServiceConf>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct TcpUdpServiceConf {
    #[serde(flatten)]
    identity_conf: IdentityConf,
    #[serde(alias = "ports", deserialize_with = "deserialize_ports")]
    port: Vec<u16>,
    public: bool,
    active: bool,
    #[serde(default = "default_host")]
    host: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_bridge() -> String {
    match env::var("MALAI_HTTP_BRIDGE") {
        Ok(value) => value,
        Err(_) => String::new(), // No default bridge - users must provide their own
    }
}

fn default_malai_conf() -> MalaiConf {
    MalaiConf { log: None }
}

/// Deserializes either a single port (`port = 3000`) or a list of ports (`port = [3000, 3001]`).
fn deserialize_ports<'de, D>(deserializer: D) -> Result<Vec<u16>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct PortsVisitor;

    impl<'de> de::Visitor<'de> for PortsVisitor {
        type Value = Vec<u16>;

        fn expecting(&self, formatter: &mut stdfmt::Formatter) -> stdfmt::Result {
            formatter.write_str("a port number or a list of port numbers")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Vec<u16>, E>
        where
            E: de::Error,
        {
            if value > u16::MAX as u64 {
                return Err(E::custom(format!("port {} is out of range", value)));
            }
            Ok(vec![value as u16])
        }

        fn visit_i64<E>(self, value: i64) -> Result<Vec<u16>, E>
        where
            E: de::Error,
        {
            if value < 0 || value > u16::MAX as i64 {
                return Err(E::custom(format!("port {} is out of range", value)));
            }
            Ok(vec![value as u16])
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Vec<u16>, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut ports = Vec::new();
            while let Some(port) = seq.next_element::<u16>()? {
                ports.push(port);
            }
            if ports.is_empty() {
                return Err(de::Error::custom("port list cannot be empty"));
            }
            Ok(ports)
        }
    }

    deserializer.deserialize_any(PortsVisitor)
}

fn parse_config(path: &Path) -> eyre::Result<Config> {
    let conf_str = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file at {}", path.display()))?;

    let conf = toml::from_str(&conf_str).context("Failed to parse config file")?;
    Ok(conf)
}

fn set_up_logging(conf: &Config) -> eyre::Result<()> {
    match &conf.malai.log {
        Some(log_dir) => {
            let log_dir = Path::new(&log_dir);
            let file_appender = rolling::daily(
                log_dir.parent().unwrap_or(Path::new("./")),
                log_dir
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("malai.log")),
            );
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            LOG_GUARD.get_or_init(|| guard);
            // tracing_subscriber::fmt()
            //     .with_writer(non_blocking)
            //     .with_ansi(false)
            //     .init();
            let subscriber = fmt::Subscriber::builder().finish().with(
                fmt::Layer::new()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true),
            );

            tracing::subscriber::set_global_default(subscriber)?;
        }
        None => {
            tracing_subscriber::fmt::init();
        }
    }
    Ok(())
}

fn load_secret_from_file(path: &Path) -> eyre::Result<(String, kulfi_id52::SecretKey)> {
    let secret_key = fs::read_to_string(path)?.trim().to_string();
    kulfi_utils::secret::handle_secret(&secret_key)
}

fn check_used(used_id52: &mut HashSet<String>, id52: &str) -> eyre::Result<()> {
    if used_id52.contains(id52) {
        Err(eyre!("Identity already used."))
    } else {
        used_id52.insert(id52.to_string());
        Ok(())
    }
}

async fn load_identity(
    identity_conf: &IdentityConf,
    used_id52: &mut HashSet<String>,
) -> eyre::Result<(String, kulfi_id52::SecretKey)> {
    let (id52, secret_key) = if let Some(secret_path) = identity_conf.secret_file.as_ref() {
        load_secret_from_file(Path::new(&secret_path))?
    } else {
        let id52 = identity_conf
            .identity
            .as_ref()
            .context("No identity specified. Please specify an identity or a secret key file.")?;
        kulfi_utils::secret::handle_identity(id52.to_string()).context(format!(
            "Failed to load identity {} from system keyring.",
            id52
        ))?
    };
    check_used(used_id52, &id52)?;
    Ok((id52, secret_key))
}

async fn set_up_http_services(
    conf: &Config,
    used_id52: &mut HashSet<String>,
    graceful: kulfi_utils::Graceful,
) {
    if let Some(http_conf) = &conf.http {
        for (name, service_conf) in &http_conf.services {
            info!("Starting HTTP services: {}", name);
            // Check
            if !service_conf.active {
                continue;
            }
            if !service_conf.public {
                tracing::warn!(
                    "You have to set public to true for service {}. Skipping.",
                    name
                );
                continue;
            }

            let (id52, secret_key) =
                match load_identity(&service_conf.identity_conf, used_id52).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            "Failed to load identity for service {}: {} Skipping.",
                            name, e
                        );
                        continue;
                    }
                };

            for &port in &service_conf.port {
                let host = service_conf.host.clone();
                let bridge = service_conf.bridge.clone();
                let graceful_clone = graceful.clone();
                let id52 = id52.clone();
                let secret_key = secret_key.clone();

                graceful.spawn(async move {
                    malai::expose_http(host, port, bridge, id52, secret_key, graceful_clone).await
                });
            }
        }
    }
}

async fn set_up_tcp_services(
    conf: &Config,
    used_id52: &mut HashSet<String>,
    graceful: kulfi_utils::Graceful,
) {
    if let Some(tcp_conf) = &conf.tcp {
        for (name, service_conf) in &tcp_conf.services {
            info!("Starting TCP services: {}", name);
            // Check
            if !service_conf.active {
                continue;
            }
            if !service_conf.public {
                tracing::warn!(
                    "You have to set public to true for service {}. Skipping.",
                    name
                );
                continue;
            }

            let (id52, secret_key) =
                match load_identity(&service_conf.identity_conf, used_id52).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            "Failed to load identity for service {}: {} Skipping.",
                            name, e
                        );
                        continue;
                    }
                };

            for &port in &service_conf.port {
                let host = service_conf.host.clone();
                let graceful_clone = graceful.clone();
                let id52 = id52.clone();
                let secret_key = secret_key.clone();

                graceful.spawn(async move {
                    malai::expose_tcp(host, port, id52, secret_key, graceful_clone).await
                });
            }
        }
    }
}

async fn set_up_udp_services(
    conf: &Config,
    used_id52: &mut HashSet<String>,
    graceful: kulfi_utils::Graceful,
) {
    if let Some(udp_conf) = &conf.udp {
        for (name, service_conf) in &udp_conf.services {
            info!("Starting UDP services: {}", name);
            if !service_conf.active {
                continue;
            }
            if !service_conf.public {
                tracing::warn!(
                    "You have to set public to true for service {}. Skipping.",
                    name
                );
                continue;
            }

            let (id52, secret_key) =
                match load_identity(&service_conf.identity_conf, used_id52).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            "Failed to load identity for service {}: {} Skipping.",
                            name, e
                        );
                        continue;
                    }
                };

            for &port in &service_conf.port {
                let host = service_conf.host.clone();
                let graceful_clone = graceful.clone();
                let id52 = id52.clone();
                let secret_key = secret_key.clone();

                graceful.spawn(async move {
                    malai::expose_udp(host, port, id52, secret_key, graceful_clone).await
                });
            }
        }
    }
}

async fn set_up_tcp_udp_services(
    conf: &Config,
    used_id52: &mut HashSet<String>,
    graceful: kulfi_utils::Graceful,
) {
    if let Some(tcp_udp_conf) = &conf.tcp_udp {
        for (name, service_conf) in &tcp_udp_conf.services {
            info!("Starting TCP+UDP services: {}", name);
            if !service_conf.active {
                continue;
            }
            if !service_conf.public {
                tracing::warn!(
                    "You have to set public to true for service {}. Skipping.",
                    name
                );
                continue;
            }

            let (id52, secret_key) =
                match load_identity(&service_conf.identity_conf, used_id52).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            "Failed to load identity for service {}: {} Skipping.",
                            name, e
                        );
                        continue;
                    }
                };

            for &port in &service_conf.port {
                let host = service_conf.host.clone();
                let graceful_clone = graceful.clone();
                let id52 = id52.clone();
                let secret_key = secret_key.clone();

                graceful.spawn(async move {
                    malai::expose_tcp_udp(host, port, id52, secret_key, graceful_clone).await
                });
            }
        }
    }
}

pub async fn run(conf_path: &Path, graceful: kulfi_utils::Graceful) {
    let conf = match parse_config(conf_path) {
        Ok(conf) => conf,
        Err(e) => {
            error!("Failed to parse config: {}", e);
            return;
        }
    };

    match set_up_logging(&conf) {
        Ok(guard) => guard,
        Err(e) => {
            error!("Failed to set up logging: {}. Skipping.", e);
        }
    };

    let mut used_id52: HashSet<String> = HashSet::new();

    set_up_http_services(&conf, &mut used_id52, graceful.clone()).await;
    set_up_tcp_services(&conf, &mut used_id52, graceful.clone()).await;
    set_up_udp_services(&conf, &mut used_id52, graceful.clone()).await;
    set_up_tcp_udp_services(&conf, &mut used_id52, graceful.clone()).await;
}

#[test]
fn parse_config_test() {
    let conf = parse_config(Path::new("tests/http_example_conf.toml")).unwrap();
    println!("{:?}", conf);
    assert!(conf.http.is_some());
    let http = conf.http.as_ref().expect("HTTP services should be present");
    assert!(http.services.get("service1").is_some());
    assert!(http.services.get("service2").is_some());
    assert!(
        http.services
            .get("service2")
            .unwrap()
            .identity_conf
            .secret_file
            .is_some()
    );

    assert!(conf.tcp.is_some());
    let tcp = conf.tcp.as_ref().expect("TCP services should be present");
    assert!(tcp.services.get("service3").is_some());
    assert_eq!(tcp.services.get("service3").unwrap().port, vec![3002]);

    // Multi-port service
    let service5 = tcp
        .services
        .get("service5")
        .expect("service5 should be present");
    assert_eq!(service5.port, vec![4000, 4001, 4002]);

    assert!(conf.udp.is_some());
    let udp = conf.udp.as_ref().expect("UDP services should be present");
    assert!(udp.services.get("service4").is_some());
}
