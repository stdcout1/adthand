use clap::Parser;
use bitcode::Encode;
#[derive(Parser)]
#[command(version, name = "adthand")]
pub enum Adthand {
    Init,
    Kill
}
