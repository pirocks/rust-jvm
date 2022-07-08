use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;



/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync>>
    where
        T: FromStr,
        T::Err: Error + Send + Sync + 'static,
        U: FromStr,
        U::Err: Error + Send + Sync + 'static,
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
    #[clap(short, long, help = "the classpath")]
    classpath: Vec<PathBuf>,
    #[clap(short, long, help = "the jar to find a manifest in and run", conflicts_with = "main", required_unless = "main")]
    jar: Option<PathBuf>,
    #[clap(short, long, help = "the main class", conflicts_with = "jar", required_unless = "jar")]
    main: Option<String>,
    #[clap(short, long, help = "properties", parse(try_from_str = parse_key_val), number_of_values = 1)]
    properties: Vec<(String, String)>,
}