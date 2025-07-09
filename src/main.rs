use std::error::Error;
mod index;
mod configs;
mod elastic_hyperion;
use clap::{Command, Arg, ArgAction};
#[tokio::main(flavor = "multi_thread", worker_threads = 32)]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("HYPERION-EOSIO-RUST")
        .version("0.01")
        .author("Andrei Levchenko")
        .about("Fast Indexer and API for Hyperion EOSIO")
        .arg(Arg::new("start_block")
            .help("Block number to start from")
            .value_parser(clap::value_parser!(u32).range(1..))
            .short('b')
            .long("start-block"))
        .arg(Arg::new("verbose")
            .help("Enable verbose output")
            .short('v')
            .long("verbose"));
    command.print_help()?;
    let matches = command.get_matches();
    let start_block: u32 = *matches.get_one::<u32>("start_block")
        .unwrap_or(&1);
    index::start_index_block_result_v0(start_block).await;
    Ok(())
}
#[macro_export]
macro_rules! measure_time {
    ($name:expr, $code:block) => {
        let start = Instant::now();
        $code
        let _duration = start.elapsed();
        //println!("{}: {:?}", $name, duration);
    };
}