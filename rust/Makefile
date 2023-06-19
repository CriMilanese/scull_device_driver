TARGET_MODULE:=rust-driver-module
# to build with kernel building system
ifneq ($(KERNELRELEASE),)
	$(TARGET_MODULE)-objs := rust_scull.o
	obj-m := $(TARGET_MODULE).o

# to build against host kernel build system
else
	BUILDSYSTEM_DIR:=/lib/modules/`uname -r`/build
	PWD:=$(shell pwd)

all:
	$(MAKE) LLVM=1 -C $(BUILDSYSTEM_DIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(BUILDSYSTEM_DIR) M=$(PWD) clean

load:
	insmod ./$(TARGET_MODULE).ko

unload:
	rmmod ./$(TARGET_MODULE).ko

endif
