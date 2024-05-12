use clap::Parser;
use cargo_metadata::camino::Utf8PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct ResourceArgs {
    /// The tool name from cargo invocation (which is ignored)
    pub tool_name: Option<String>,

    /// The optional package to operate on
    #[arg(short, long, value_name = "FILE")]
    pub package: Option<Utf8PathBuf>,
}
