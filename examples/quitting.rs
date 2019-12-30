//! Quitting. See src/examples/quitting.rs for documentation
use async_trait::async_trait;

use nvim_rs::{
  create,
  runtime::{ChildStdin, Command},
  Handler,
};

use std::error::Error;

const NVIMPATH: &str = "neovim/build/bin/nvim";
struct NeovimHandler {}

#[async_trait]
impl Handler for NeovimHandler {
  type Writer = ChildStdin;
}

#[tokio::main]
async fn main() {
  let handler = NeovimHandler {};

  let res = create::run_child_cmd(
    Command::new(NVIMPATH)
      .args(&["-u", "NONE", "--embed", "--headless"])
      .env("NVIM_LOG_FILE", "nvimlog"),
    handler,
    |nvim| {
      async move {
        let chan = nvim.get_api_info().await.unwrap()[0].as_i64().unwrap();
        let close = format!("call chanclose({})", chan);
        Ok(nvim.command(&close).await?)
      }
    },
  )
  .await;

  if let Err(e) = res {
    eprintln!("Error in last command: {}", e);
    eprintln!("Caused by : {:?}", e.as_ref().source());

    if e.is_channel_closed() {
      eprintln!("Channel closed, quitting!");
    } else {
      eprintln!("Channel was not closed, no idea what happened!");
    }
  }
}
