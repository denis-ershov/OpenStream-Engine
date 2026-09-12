use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser, Debug)]
#[command(
    name = "osrule",
    about = "OpenStream 2.0 CLI: Rule linter, Ed25519 signer, and catalog indexer",
    version = "2.0.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Проверить синтаксис и SecOps-ограничения правил .osrule.yaml
    Lint {
        /// Пути к файлам или каталогам с правилами
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
    /// Сгенерировать пару ключей Ed25519 для подписания правил
    Keygen {
        /// Префикс для сохранения файлов .key и .pub (необязательно)
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Подписать манифест правила закрытым ключом Ed25519
    Sign {
        /// Путь к файлу .osrule.yaml
        file: PathBuf,
        /// Закрытый ключ (hex-строка или путь к .key файлу)
        #[arg(short, long)]
        key: String,
        /// Путь для сохранения подписи (по умолчанию <file>.osrule.sig)
        #[arg(short, long)]
        sig: Option<PathBuf>,
    },
    /// Проверить цифровую подпись манифеста правила
    Verify {
        /// Путь к файлу .osrule.yaml
        file: PathBuf,
        /// Открытый ключ (hex-строка или путь к .pub файлу)
        #[arg(short, long)]
        pubkey: String,
        /// Путь к файлу подписи (по умолчанию <file>.osrule.sig)
        #[arg(short, long)]
        sig: Option<PathBuf>,
    },
    /// Собрать публичный JSON-индекс каталога правил
    Index {
        /// Каталог с файлами .osrule.yaml
        #[arg(default_value = "rules")]
        dir: PathBuf,
        /// Путь для выгрузки файла индекса
        #[arg(short, long, default_value = "rules-index.json")]
        out: PathBuf,
    },
    /// Синхронизировать и проверить целостность каталога правил
    Update {
        /// Каталог с файлами .osrule.yaml
        #[arg(short, long, default_value = "rules")]
        dir: PathBuf,
        /// Проверять SHA256 контрольные суммы
        #[arg(long, default_value_t = true)]
        verify_hashes: bool,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lint { paths } => commands::lint::execute_lint(&paths),
        Commands::Keygen { out } => commands::keygen::execute_keygen(out),
        Commands::Sign { file, key, sig } => commands::sign::execute_sign(file, key, sig),
        Commands::Verify { file, pubkey, sig } => commands::verify::execute_verify(file, pubkey, sig),
        Commands::Index { dir, out } => commands::index::execute_index(dir, out),
        Commands::Update { dir, verify_hashes } => commands::update::execute_update(dir, verify_hashes),
    }
}
