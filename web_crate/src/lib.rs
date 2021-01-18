mod js {
  extern "C" {
    pub fn setupCanvas();
    pub fn clearToBlue();
  }
}

#[no_mangle]
pub extern "C" fn start() {
  unsafe {
    js::setupCanvas();
    js::clearToBlue();
  }
}
