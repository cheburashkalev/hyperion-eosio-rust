use std::error::Error;
mod index;
mod configs;
mod elastic_hyperion;
#[tokio::main(flavor = "multi_thread", worker_threads = 32)]
async fn main() -> Result<(), Box<dyn Error>> {
    index::start_index_block_result_v0().await;
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