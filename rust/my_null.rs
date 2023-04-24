// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

use kernel::prelude::*;
use kernel::{
    file::{File, Operations},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    sync::{Arc, ArcBorrow},
};

module! {
    type: NullDeviceModule,
    name: "isnull",
    author: "Cristiano Milanese",
    description: "Rust null device sample",
    license: "GPL",
}

struct NullDevice {
  number: usize,
  contents: Vec<u8>,
}

#[vtable]
impl Operations for NullDevice {

    type OpenData = Arc<NullDevice>;
    type Data = Arc<NullDevice>;

    fn open(data: &Self::OpenData, _file: &File) -> Result<Self::Data> {
        pr_info!("opened device with minor number {}\n", data.number);
        Ok(data.clone())
    }

    fn read(
        _this: ArcBorrow<'_, NullDevice>,
        _: &File,
        _data: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        Ok(0)
    }

    fn write(
        _this: ArcBorrow<'_, NullDevice>,
        _: &File,
        data: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        Ok(data.len())
    }
}

struct NullDeviceModule {
    _dev: Pin<Box<miscdev::Registration<NullDevice>>>,
}

impl kernel::Module for NullDeviceModule {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {

        let dev = Arc::try_new(NullDevice {
		number: 0,
		contents: Vec::new(),
	})?;
        pr_info!("loading null device driver\n");
        let reg = miscdev::Registration::<NullDevice>::new_pinned(fmt!("{name}"), dev)?;
        Ok(NullDeviceModule {
            _dev: reg,
        })
    }
}

impl Drop for NullDeviceModule {
    fn drop(&mut self) {
        pr_info!("Unloading null device driver\n");
    }
}
