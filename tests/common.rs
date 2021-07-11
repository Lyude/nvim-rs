use std::path::Path;
use std::env;

const NVIMPATH: &str = "neovim/build/bin/nvim";

pub fn nvim_path() -> &Path {
  let (path, have_env) = match env::var("NVIM_RS_TEST_BIN") {
    Ok(path) => (path, true),
    Err(_) => (NVIMPATH, false),
  };

  let path = Path::new(path);
  if !path.exists() {
    if have_env {
      panic!("nvim bin from NVIM_RS_TEST_BIN \"{}\" does not exist", path)
    } else {
      panic!(
        "\"{}\" not found, maybe you need to build it or set \
        NVIM_RS_TEST_BIN?",
        NVIMPATH
      );
    }
  }
  path
}
