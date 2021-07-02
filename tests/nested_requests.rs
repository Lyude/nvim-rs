use nvim_rs::{
  compat::tokio::Compat, create::tokio as create, neovim::Neovim, Handler,
};

use async_trait::async_trait;

use rmpv::Value;

use std::sync::Arc;

use tokio::{
  self,
  process::{ChildStdin, Command},
  spawn,
};

use futures::lock::Mutex;
use futures::select;
use futures::future::FutureExt;

const NVIMPATH: &str = "neovim/build/bin/nvim";

#[derive(Clone)]
struct NeovimHandler {
  froodle: Arc<Mutex<String>>,
}

#[async_trait]
impl Handler for NeovimHandler {
  type Writer = Compat<ChildStdin>;

  async fn handle_request(
    &self,
    name: String,
    args: Vec<Value>,
    neovim: Neovim<Compat<ChildStdin>>,
  ) -> Result<Value, Value> {
    match name.as_ref() {
      "dummy" => Ok(Value::from("o")),
      "req" => {
        let v = args[0].as_str().unwrap();

        let neovim = neovim.clone();
        match v {
          "y" => {
            let mut x: String = neovim
              .get_vvar("progname")
              .await
              .unwrap()
              .as_str()
              .unwrap()
              .into();
            x.push_str(" - ");
            x.push_str(
              neovim.get_var("oogle").await.unwrap().as_str().unwrap(),
            );
            x.push_str(" - ");
            x.push_str(
              neovim
                .eval("rpcrequest(1,'dummy')")
                .await
                .unwrap()
                .as_str()
                .unwrap(),
            );
            x.push_str(" - ");
            x.push_str(
              neovim
                .eval("rpcrequest(1,'req', 'z')")
                .await
                .unwrap()
                .as_str()
                .unwrap(),
            );
            Ok(Value::from(x))
          }
          "z" => {
            let x: String = neovim
              .get_vvar("progname")
              .await
              .unwrap()
              .as_str()
              .unwrap()
              .into();
            Ok(Value::from(x))
          }
          &_ => Err(Value::from("wrong argument to req")),
        }
      }
      &_ => Err(Value::from("wrong method name for request")),
    }
  }

  async fn handle_notify(
    &self,
    name: String,
    args: Vec<Value>,
    _neovim: Neovim<Compat<ChildStdin>>,
  ) {
    eprintln!("Handling notification {}", name);
    match name.as_ref() {
      "set_froodle" => {
        let s =args[0].as_str().unwrap().to_string();
        *self.froodle.lock().await = s
      }
      _ => {}
    };
  }
}

#[tokio::test(flavor = "current_thread")]
async fn nested_requests() {
  let rs = r#"exe ":fun M(timer) 
      call rpcnotify(1, 'set_froodle', rpcrequest(1, 'req', 'y'))
    endfun""#;
  let rs2 = r#"exe ":fun N(timer) 
      call chanclose(1)
    endfun""#;

  let froodle = Arc::new(Mutex::new(String::new()));
  let handler = NeovimHandler {
    froodle: froodle.clone(),
  };

  let (nvim, io_handler, _child) = create::new_child_cmd(
    Command::new(NVIMPATH).args(&[
      "-u",
      "NONE",
      "--embed",
      "--headless",
      "-c",
      rs,
      "-c",
      ":let timer = timer_start(500, 'M')",
      "-c",
      rs2,
      "-c",
      ":let timer = timer_start(1500, 'N')",
    ]),
    handler,
  )
  .await
  .unwrap();

  let nv = nvim.clone();
  spawn(async move { nv.set_var("oogle", Value::from("doodle")).await });

  // The 2nd timer closes the channel, which will be returned as an error from
  // the io handler. We only fail the test if we got another error
  if let Err(err) = io_handler.await.unwrap() {
    if !err.is_channel_closed() {
      panic!("{}", err);
    }
  }

  assert_eq!("nvim - doodle - o - nvim", *froodle.lock().await);
}
