use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    days_before: u32,

    days_after: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    process::load_foods(args.days_before, args.days_after).await;
}
