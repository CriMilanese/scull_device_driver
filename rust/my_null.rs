//! loads a custom implementation of the null device driver as a module

use kernel::prelude::*;
use kernel::{file, chrdev};

module! {
  type: IsChrDev,
  name: "isnull",
  license: "GPL",
}

struct IsNull;

#[vtable]
impl file::Operations for IsNull {
  fn open(_shared: &(), _file: &file::File) -> Result<Self::Data> {
    pr_info!("opening this device file\n");
    Ok(())
  }
}

struct IsChrDev {
  // Pin a pointer to a Box (aka allocated memory) as big as a Registration for char device
  _dev: Pin<Box<chrdev::Registration<1>>>,
}

impl kernel::Module for IsChrDev {
  fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
    pr_info!("loading IsChrDev module\n");
    // question mark at the end allows us to dodge error handling
    let reg = chrdev::Registration::new_pinned(_name, 0, _module)?;
    Ok(IsChrDev { _dev: reg })
  }
}

impl Drop for IsChrDev {
    fn drop(&mut self) {
        pr_info!("Unloading IsChrDev module\n");
    }
}
