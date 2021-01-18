mod js {
  extern "C" {
    pub fn setup_canvas();
    pub fn clear_to_blue();
  }
}

#[no_mangle]
pub extern "C" fn start() {
  unsafe {
    js::setup_canvas();
    js::clear_to_blue();
  }
}
