use std::env;

#[tokio::main]
async fn main() {
  let mut args = env::args();
  let _ = args.next();

  let working_dir = args.next();
  if let Err(err) = sentinel_bot::run(working_dir).await {
    println!("An error occurred: {err}")
  }

  println!("Execution finished.")
}
