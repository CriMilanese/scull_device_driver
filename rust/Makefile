TARGET_MODULE:=rust_scull
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
