// SPDX-License-Identifier: GPL-2.0

//! Rust scull device sample.

use kernel::{
    Module,
    miscdev,
    prelude::*,
    file::{File, Operations, SeekFrom},
    sync::{Arc, ArcBorrow, smutex::Mutex},
    io_buffer::{IoBufferReader, IoBufferWriter}
};


module! {
    type: ScullDeviceModule,
    name: "rust_scull",
    author: "Cristiano Milanese",
    description: "Safe Character Utility for Loading Localities",
    license: "GPL",
}

const BLOCK_SIZE : usize = 4096;

struct ScullDevice {
  data:  Mutex<Vec<Vec<u8>>>,
  cursor: Mutex<usize>
}

impl ScullDevice {
    fn try_new() -> Result<Self> {
      let set = Vec::<Vec<u8>>::try_with_capacity(BLOCK_SIZE)?;
      Ok(Self {
      	data: Mutex::new(set),
      	cursor: Mutex::new(0)
      })
    }

    // returns the current maximum amount of bytes readable
    fn capacity(&self) -> usize {
				let vec = self.data.lock();
				let mut byte_size = 0;
				for _ in 0..vec.len() {
						byte_size += BLOCK_SIZE;
				}
				byte_size
    }

    fn find_block(
    		&self,
    		row: usize
    ) -> Result<usize> {
        let mut vec = self.data.lock();
        if row >= vec.len() {
            // add one to compensate for index vs size
            let fill = row.saturating_sub(vec.len()) + 1;
		        for _i in 0..fill {
		            match vec.try_push(Vec::<u8>::new()) {
		            		Ok(_) => continue,
		            		Err(_) => {
		            		  pr_err!("out of memory for new row {}\n", vec.len());
		            		  return Err(ENOMEM)
		            		}
		            }
		        }
        }
        if vec[row].len() != BLOCK_SIZE {
        		match vec[row].try_resize(BLOCK_SIZE, 0) {
        				Ok(..) => Ok(BLOCK_SIZE),
        				Err(..) => {
	            		  pr_err!("out of memory while allocating {} bytes for block {}\n", BLOCK_SIZE, row);
        						Err(ENOMEM)
        				}
        		}
        } else {
        		return Ok(BLOCK_SIZE);
        }
    }
}

#[vtable]
impl Operations for ScullDevice {

    type OpenData = Arc<ScullDevice>;
    type Data = Arc<ScullDevice>;

    fn open(
    	this: & Self::Data,
    	_file: &File
    	) -> Result<Self::Data> {
        Ok(this.clone())
    }

    fn read(
        this: ArcBorrow<'_, ScullDevice>,
        _file: &File,
        user_buff: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        if user_buff.is_empty() { return Ok(0) }
        let total_offset;
        // this portion is not present in the c code because
        // there we modify the file pointer directly
        {
        		let curr_pos = this.cursor.lock();
        		let cast : u64 = (*curr_pos).try_into().unwrap();
		        total_offset = _offset.checked_add(cast).unwrap();
		    }
        let block_index = total_offset.checked_div(BLOCK_SIZE as u64).unwrap();
        let _rest = total_offset.checked_rem(BLOCK_SIZE as u64).unwrap();
        let row : usize = block_index.try_into()?;
        let block_offset : usize = _rest.try_into()?;
        match this.find_block(row) {
        		Ok(bytes) => {
	        	  	let tot = user_buff.len().checked_add(block_offset).unwrap();
	        	  	let mut end = bytes;
	        	  	if tot < bytes { end = tot; }
	        	  	let vec = this.data.lock();
	              user_buff.write_slice(& vec[row][block_offset..end])?;
	              return Ok(end.saturating_sub(block_offset));
	          },
	          Err(err) => Err(err)
        }
    }

    fn write(
        this: ArcBorrow<'_, ScullDevice>,
        _file: &File,
        user_buff: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        if user_buff.is_empty() { return Ok(0) }
				let total_offset;
        {
        		let curr_pos = this.cursor.lock();
        		let cast : u64 = (*curr_pos).try_into().unwrap();
		        total_offset = _offset.checked_add(cast).unwrap();
		    }

        let block_index = total_offset / BLOCK_SIZE as u64;
        let _rest = total_offset % BLOCK_SIZE as u64;
        let row : usize = block_index.try_into()?;
        let offset : usize = _rest.try_into()?;
        match this.find_block(row) {
						Ok(bytes) => {
		            let mut vec = this.data.lock();
	        	  	let tot = user_buff.len().checked_add(offset).unwrap();
	        	  	let mut end = bytes;
	        	  	if tot < bytes { end = tot }
		           	user_buff.read_slice(&mut vec[row][offset..end])?;
		           	return Ok(end.saturating_sub(offset))
		        },
		        Err(err) => Err(err)
        }
    }
    
    fn seek(
    		this: ArcBorrow<'_, ScullDevice>,
    		_fd: &File,
    		_offset: SeekFrom
    ) -> Result<u64> {
				match _offset {
						SeekFrom::Start(of) => {
        				let mut guard = this.cursor.lock();
							  *guard = of.try_into()?;
							  return Ok(of);
						}  
						SeekFrom::End(of) => {
								let from_begin : usize = this.capacity();
								let ret : usize = from_begin.saturating_add_signed(of.try_into()?);	
								let mut guard = this.cursor.lock();
         				*guard = ret;
								return Ok(ret.try_into()?);
						}
						SeekFrom::Current(of) =>  {
        				let mut guard = this.cursor.lock();
								let ret = (*guard).saturating_add_signed(of.try_into()?);
								*guard = ret;
								return Ok(ret.try_into()?);
						}
				}
    }
    
    fn release(_this: Arc<ScullDevice>, _: &File) {}
}

struct ScullDeviceModule {
    _dev: Pin<Box<miscdev::Registration<ScullDevice>>>,
}

impl Module for ScullDeviceModule {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        let dev = Arc::try_new(ScullDevice::try_new()?)?;
        let reg = miscdev::Registration::<ScullDevice>::new_pinned(fmt!("{name}"), dev)?;
				pr_debug!("mounting {} in devfs and registering its device driver\n", fmt!("{name}"));
        Ok(ScullDeviceModule {
            _dev: reg,
        })
    }
}
