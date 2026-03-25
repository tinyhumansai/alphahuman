use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<serde_json::Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Parser)]
#[command(name = "alphahuman-cli")]
#[command(about = "CLI for the AlphaHuman core RPC server")]
struct Cli {
    /// Core RPC endpoint URL
    #[arg(long, global = true)]
    rpc_url: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check core health
    Ping,
    /// Print core version
    Version,
    /// Get health snapshot
    Health,
    /// Get runtime flags
    RuntimeFlags,

    /// Config operations
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// Service operations
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },

    /// Doctor operations
    Doctor {
        #[command(subcommand)]
        command: DoctorCommand,
    },

    /// Integrations operations
    Integrations {
        #[command(subcommand)]
        command: IntegrationsCommand,
    },

    /// Send one-shot agent message
    AgentChat(AgentChatArgs),

    /// Hardware operations
    Hardware {
        #[command(subcommand)]
        command: HardwareCommand,
    },

    /// Encrypt a secret
    Encrypt {
        plaintext: String,
    },

    /// Decrypt a secret
    Decrypt {
        ciphertext: String,
    },

    /// Toggle browser allow-all runtime flag
    BrowserAllowAll {
        #[arg(long)]
        enabled: bool,
    },

    /// Refresh model catalog
    ModelsRefresh {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// Migrate OpenClaw memory
    MigrateOpenclaw {
        #[arg(long)]
        source_workspace: Option<String>,
        #[arg(long, default_value_t = true)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Get full config snapshot
    Get,
}

#[derive(Debug, Subcommand)]
enum ServiceCommand {
    Install,
    Start,
    Stop,
    Status,
    Uninstall,
}

#[derive(Debug, Subcommand)]
enum DoctorCommand {
    /// Run doctor checks
    Report,
    /// Probe model catalog
    Models {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long, default_value_t = true)]
        use_cache: bool,
    },
}

#[derive(Debug, Subcommand)]
enum IntegrationsCommand {
    /// List integrations
    List,
    /// Get one integration info
    Info {
        #[arg(long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum HardwareCommand {
    /// Discover connected hardware
    Discover,
    /// Introspect one device path
    Introspect {
        #[arg(long)]
        path: String,
    },
}

#[derive(Debug, Args)]
struct AgentChatArgs {
    message: String,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    temperature: Option<f64>,
}

fn endpoint(cli: &Cli) -> String {
    if let Some(url) = &cli.rpc_url {
        return url.clone();
    }
    std::env::var("ALPHAHUMAN_CORE_RPC_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:7788/rpc".to_string())
}

fn call(url: &str, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    let req = RpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method: method.to_string(),
        params,
    };

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(url)
        .json(&req)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    let payload: RpcResponse = resp.json().map_err(|e| format!("invalid response: {e}"))?;

    if let Some(err) = payload.error {
        return Err(format!(
            "rpc error {}: {}{}",
            err.code,
            err.message,
            err.data.map(|d| format!(" ({d})")).unwrap_or_default()
        ));
    }

    Ok(payload.result.unwrap_or(serde_json::Value::Null))
}

fn execute(cli: Cli) -> Result<serde_json::Value, String> {
    let url = endpoint(&cli);

    match cli.command {
        Command::Ping => call(&url, "core.ping", serde_json::json!({})),
        Command::Version => call(&url, "core.version", serde_json::json!({})),
        Command::Health => call(&url, "alphahuman.health_snapshot", serde_json::json!({})),
        Command::RuntimeFlags => call(&url, "alphahuman.get_runtime_flags", serde_json::json!({})),
        Command::Config { command } => match command {
            ConfigCommand::Get => call(&url, "alphahuman.get_config", serde_json::json!({})),
        },
        Command::Service { command } => match command {
            ServiceCommand::Install => {
                call(&url, "alphahuman.service_install", serde_json::json!({}))
            }
            ServiceCommand::Start => call(&url, "alphahuman.service_start", serde_json::json!({})),
            ServiceCommand::Stop => call(&url, "alphahuman.service_stop", serde_json::json!({})),
            ServiceCommand::Status => {
                call(&url, "alphahuman.service_status", serde_json::json!({}))
            }
            ServiceCommand::Uninstall => {
                call(&url, "alphahuman.service_uninstall", serde_json::json!({}))
            }
        },
        Command::Doctor { command } => match command {
            DoctorCommand::Report => call(&url, "alphahuman.doctor_report", serde_json::json!({})),
            DoctorCommand::Models {
                provider,
                use_cache,
            } => call(
                &url,
                "alphahuman.doctor_models",
                serde_json::json!({
                    "provider_override": provider,
                    "use_cache": use_cache,
                }),
            ),
        },
        Command::Integrations { command } => match command {
            IntegrationsCommand::List => {
                call(&url, "alphahuman.list_integrations", serde_json::json!({}))
            }
            IntegrationsCommand::Info { name } => call(
                &url,
                "alphahuman.get_integration_info",
                serde_json::json!({ "name": name }),
            ),
        },
        Command::AgentChat(args) => call(
            &url,
            "alphahuman.agent_chat",
            serde_json::json!({
                "message": args.message,
                "provider_override": args.provider,
                "model_override": args.model,
                "temperature": args.temperature,
            }),
        ),
        Command::Hardware { command } => match command {
            HardwareCommand::Discover => {
                call(&url, "alphahuman.hardware_discover", serde_json::json!({}))
            }
            HardwareCommand::Introspect { path } => call(
                &url,
                "alphahuman.hardware_introspect",
                serde_json::json!({ "path": path }),
            ),
        },
        Command::Encrypt { plaintext } => call(
            &url,
            "alphahuman.encrypt_secret",
            serde_json::json!({ "plaintext": plaintext }),
        ),
        Command::Decrypt { ciphertext } => call(
            &url,
            "alphahuman.decrypt_secret",
            serde_json::json!({ "ciphertext": ciphertext }),
        ),
        Command::BrowserAllowAll { enabled } => call(
            &url,
            "alphahuman.set_browser_allow_all",
            serde_json::json!({ "enabled": enabled }),
        ),
        Command::ModelsRefresh { provider, force } => call(
            &url,
            "alphahuman.models_refresh",
            serde_json::json!({
                "provider_override": provider,
                "force": force,
            }),
        ),
        Command::MigrateOpenclaw {
            source_workspace,
            dry_run,
        } => call(
            &url,
            "alphahuman.migrate_openclaw",
            serde_json::json!({
                "source_workspace": source_workspace,
                "dry_run": dry_run,
            }),
        ),
    }
}

fn main() {
    let cli = Cli::parse();
    match execute(cli) {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| "null".to_string())
            );
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
