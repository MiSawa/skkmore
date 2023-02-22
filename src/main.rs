use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on.
    #[arg(short, long, default_value_t = 1178)]
    port: u16,
}

fn main() {
    let args = Args::parse();
    dbg!(args);
}
