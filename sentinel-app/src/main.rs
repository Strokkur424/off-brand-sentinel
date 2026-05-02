use std::env;

#[tokio::main]
async fn main() {
  let mut args = env::args();
  let _ = args.next();

  let working_dir = args.next();
  sentinel_bot::run(working_dir).await;

  println!("Execution finished.")
}
