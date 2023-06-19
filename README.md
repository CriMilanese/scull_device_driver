# rust scull device driver
a simple rust implementation of the scull device found on "Linux Device Drivers" [book]{https://lwn.net/Kernel/LDD3} chapter 3.

### tree
`./c`
contains the c implementation of a custom scull device driver

`./rust`
the rust counterpart

`**/jf.fio`
the FIO job file associated with the performance tests we ran on the related device file

### requires
It is highly recommended to use a Virtual Machine with a hypervisor of your choice, the following
operations are often irreversible and an unsound configuration will corrupt data in the best scenarios.

The kernel module must be built using a kernel build system that include the Rust compiler,
at the time of writing no distribution delivers rust support, hence a custom kernel must be
built from scratch.

#### gather the tools
In order to do that, start by cloning the [Rust-for-Linux]{https://github.com/Rust-for-Linux/linux}
project, then gather the Rust compiler and its dependencies, it is easier to use the [rustup]{https://www.rust-lang.org/tools/install} tool.
Rust needs clang to correctly create the bindings with the legacy C types.
You will therefore need to install the [llvm]{https://llvm.org/docs/GettingStarted.html} toolchain as well.
To make sure that the rust compiler version is matching the one required by the linux kernel version you checked out, move to the repository root folder and run:
`rustup override set $(scripts/min-version-tools.sh rustc)`
Finally, a makefile target is there to help make sure your kernel is building with Rust support
available, as Rust needs clang to compile, the same requirement applies to the kernel, we can enforce this using:
`make LLVM=1 rustavailable`

#### configure the kernel
The Linux kernel docs reports a [quick start]{https://docs.kernel.org/rust/quick-start.html} guide to help you get to this point.
Next, we need to configure the kernel configuration to enable Rust before compiling:
`make LLVM=1 defconfig rust.config`
This combination of configuration files have been tested on Fedora 37 host OS only, so far.
If the kernel configuration requires manual intervention, the *menuconfig* target will display
the current configuration, allowing you to make changes where needed, including enabling modules
to be loaded at runtime and some rust *samples*.
`make menuconfig`
You can finally build the kernel with `make LLVM=1 -j$(nproc)` where nproc is the number of processor available, so to shorten build time.

#### install custom kernel
Make sure to install the kernel development library related to the kernel version we want to build this module against.

rhel `dnf install kernel-devel-$(uname -r)` \
debian `apt install linux-headers-$(uname -r)`

Before installing the new header files in the host tree, we need to modify the kernel name, 
add your preferred suffix to the kernel name on the repository root Makefile, EXTRAVERSION
will be appended to the kernel version and hash. Not committing the changes will lead to an
extra "dirty" string being appended at the end of the kernel name.

install extra modules and main headers:
`make LLVM=1 modules_install`
`make LLVM=1 install`

update grub to list the new kernel when booting:
`sudo grub2-mkconfig -o /boot/grub2/grub.cfg`

NB: if grub wait timeout is 0, you will not have the time to select the kernel to boot from.
make sure the homonymous attribute is configured in your system, to an adequate value.

### usage

You booted into a linux kernel that has Rust support, now we can use the modules in this repository
to create the scull device driver, navigate to the rust subfolder and run `make`.
The kernel object file (.ko) will show up in this folder and you can load it in your 
system with `make load`, or similarly *unload* it.

The device driver is registered as a miscellaneous device, which means it will automatically
be assigned a major number of 10 and the first minor number available. Once you load the
module(s), you will find a special device file in the _/dev_ directory named after the 
original *scull* driver and its details will refer to such numbers.

You are now free to play with this file as if it was a normal file, with some exceptions.

#### workarounds and their implications
This project was originally intended to be a block device driver written in Rust, exploiting
the language safety guarantees to write kernel code. The lack of C bindings to the original 
kernel block layer and its structures moved the objective towards a mock-up, namely a character
device driver that uses a bi-dimensional array to simulate reading and writing to non-contiguous
areas of memory. Furthermore, to test its performance using [FIO]{https://fio.readthedocs.io/en/latest/} (a standard testing tool
for IO operations), our device driver *never returns zero* on read or write, because if our
file was actually mapped to physical memory, that memory would have been always readable or
writable, at least partially.
Considering the above, some user-space programs will expect a return value of zero, for example
the `cat` utility, which concats files and prints them to _stdout_ until it receives a 0 in
return for its read requests. Asynchronous read/write calls will also accept such a value,
yet they are beyond the scope of this device driver.

#### a word on memory management
Upon load, the module will be part of the kernel address space, with its benefits and 
limitations.
Any read/write operation will request memory from RAM, in fact requesting pages from 
the virtual address space, risking that they might be swapped out, in case the OS sees 
the necessity. Rust memory safety guarantuees that in the event of a kernel panic
due to inexistent pages, all memory allocated is dropped, as their owners go out of scope.
