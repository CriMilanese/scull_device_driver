// SPDX-License-Identifier: GPL-2.0

//! Rust scull device sample.

use kernel::{
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
    description: "Rust Simple Character Utility for Loading Localities",
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

    fn find_block(
    		&self,
    		row: usize
    ) -> Result<usize> {
        let mut vec = self.data.lock();
				//allocate the index
        if row >= vec.len() {
            pr_info!("adding\n");
            // add one to compensate for index vs size
            let fill = row.saturating_sub(vec.len()).checked_add(1).unwrap();
		        for _ in 0..fill {
		            pr_info!("new vector\n");
		            match vec.try_push(Vec::<u8>::new()) {
		            		Ok(_) => continue,
		            		Err(_) => return Err(ENOMEM)
		            }
		        }
        }
        pr_info!("trying to enter the row {}, current length is {}\n", row, vec.len());
        match vec[row].try_resize(BLOCK_SIZE, 0) {
        		Ok(..) => Ok(BLOCK_SIZE),
        		Err(..) => Err(ENOMEM)
        }
/*
        if rows.len() > row_index && rows[row_index].len() > 0 {
            pr_info!("it is possible to read {} bytes\n", rows[row_index].len());
        	  return Some(rows[row_index].len());
        } */
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
        pr_info!("opened scull device file\n");
        Ok(this.clone())
    }

    fn read(
        this: ArcBorrow<'_, ScullDevice>,
        _file: &File,
        user_buff: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        // find the correct block given the offset
        let total_offset;
        {
        		let curr_pos = this.cursor.lock();
        		let cast : u64 = (*curr_pos).try_into().unwrap();
		        total_offset = _offset.checked_add(cast).unwrap();
		    }
        pr_info!("totaling offset {}, just added some {} into it\n", total_offset, _offset);
        let block_index = total_offset / BLOCK_SIZE as u64;
        let _rest = total_offset % BLOCK_SIZE as u64;
        let row : usize = block_index.try_into()?;
        let block_offset : usize = _rest.try_into()?;
        pr_info!("writing to row {}, some {} into it\n", row, block_offset);
        pr_info!("the user requested {} bytes\n", user_buff.len());
        match this.find_block(row) {
        		Ok(bytes) => {
	        	  	let vec = this.data.lock();
	        	  	// missing the case in which the user buffer is smaller than the block size
	        	  	// bytes will show too big of a slice, the user_buff only has space for a few
	        	  	let tot = user_buff.len().checked_add(block_offset).unwrap();
	        	  	let mut end = bytes;
	        	  	if tot < bytes { end = tot; }
	        	  	//let end = offset.checked_add(min).unwrap();
	              user_buff.write_slice(& vec[row][block_offset..end])?;
	        		  pr_info!("reading - successfully read {} bytes from device file\n", end.saturating_sub(block_offset));
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
				let total_offset;
        {
        		let curr_pos = this.cursor.lock();
        		let cast : u64 = (*curr_pos).try_into().unwrap();
		        total_offset = _offset.checked_add(cast).unwrap();
		    }

        pr_info!("totaling offset {}, just added some {} into it\n", total_offset, _offset);
        // find the correct block given the offset
        let block_index = total_offset / BLOCK_SIZE as u64;
        if user_buff.len() == 0 { return Ok(0) }
        let _rest = total_offset % BLOCK_SIZE as u64;
        let row : usize = block_index.try_into()?;
        let offset : usize = _rest.try_into()?;
        pr_info!("writing to row {}, some {} into it\n", row, offset);
        pr_info!("the user requested {} bytes\n", user_buff.len());
        match this.find_block(row) {
						Ok(bytes) => {
								// can only be BLOCK_SIZE
	        	  	let tot = user_buff.len().checked_add(offset).unwrap();
	        	  	let mut end = bytes;
	        	  	if tot < bytes { end = tot }
		            let mut vec = this.data.lock();
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
		     		pr_info!("seeking from start with offset {}\n", of);
            let mut guard = this.cursor.lock();
            *guard = of.try_into()?;
            return Ok(of);
		     }  
		     SeekFrom::End(of) => {
		     		pr_info!("seeking from end with offset {}\n", of);
		     		let from_begin : usize = BLOCK_SIZE * BLOCK_SIZE;
		     		let mut guard = this.cursor.lock();
		     		let ret : usize = from_begin.saturating_add_signed(of.try_into()?);	
     				*guard = ret;
		     		//let from_begin = BLOCK_SIZE * BLOCK_SIZE - of;
     				return Ok(ret.try_into()?);
		     }
		     SeekFrom::Current(of) =>  {
		     		pr_info!("seeking from curr with offset {}\n", of);
    				let mut guard = this.cursor.lock();
    				let ret = (*guard).saturating_add_signed(of.try_into()?);
    				*guard = ret;
    				return Ok(ret.try_into()?);
		     }
       }
    }
    
    fn release(_this: Arc<ScullDevice>, _: &File) {
        pr_info!("closing scull device file\n");
    }
}

struct ScullDeviceModule {
    _dev: Pin<Box<miscdev::Registration<ScullDevice>>>,
}

impl kernel::Module for ScullDeviceModule {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("loading scull device driver\n");
        let dev = Arc::try_new(ScullDevice::try_new()?)?;					
        let reg = miscdev::Registration::<ScullDevice>::new_pinned(fmt!("{name}"), dev)?;
        Ok(ScullDeviceModule {
            _dev: reg,
        })
    }
}

impl Drop for ScullDeviceModule {
    fn drop(&mut self) {
        pr_info!("Unloading scull device driver\n");
    }
}
