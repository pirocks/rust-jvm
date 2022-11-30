use std::error::Error;
use std::path::PathBuf;

use clap::Parser;

/// Parse a single key-value pair
fn parse_key_val(s: &str) -> Result<(String, String), Box<dyn Error + Send + Sync>>
{
    //todo support escaping
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}


#[derive(Parser, Debug, Clone)]
#[clap(version)]
pub struct JVMArgs {
    #[clap(long, help = "the java home for this jvm")]
    pub java_home: PathBuf,
    #[clap(long, multiple = true, help = "the classpath")]
    pub classpath: Vec<PathBuf>,
    /*#[clap(long, help = "the jar to find a manifest in and run", conflicts_with = "main", required_unless = "main")]
    jar: Option<PathBuf>,*/
    //, conflicts_with = "jar", required_unless = "jar"
    #[clap(long, help = "the main class")]
    pub main: String,
    #[clap(long, help = "properties", parse(try_from_str = parse_key_val), number_of_values = 1)]
    pub properties: Vec<(String, String)>,
    #[clap(long, help = "args for java program")]
    pub args: Vec<String>,
    #[clap(long, help = "enable assertions")]
    pub enable_assertions: bool,
    #[clap(long, help = "Enable exception debug logging")]
    pub debug_exceptions: bool,
    #[clap(long, help = "Store anonymous classes")]
    pub store_anon_class: bool
}