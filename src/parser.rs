use clap::Parser;
/// Consumer to adthand dameon
#[derive(Parser)]
#[command(version, about, name = "adthand")]
pub enum Adthand {
    Init,
    Ping,
    Kill,
    Next {
        #[arg(short)]
        relative: bool
    },
    Waybar,
    All
}
