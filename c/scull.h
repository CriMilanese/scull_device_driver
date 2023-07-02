#ifndef SCULL_H
#define SCULL_H

/*
* Scull inspired character device that behaves similarly to a block device driver, a user
* program can seek into the device and read and/or write from there. A real block device
* driver would then queue such request, to defer work to the scheduler.
* In order to behave like a block device driver, we need to return the amount requested
* or less if the read operation could only partially succeed, zero is a proper returning
* value only for asynchronous requests, a block device file with a given size should
* always return a positive value on synchronous requests
*/

#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/miscdevice.h>
#include <linux/mutex.h>
#include <linux/uaccess.h>

// BLOCK_SIZE already defined in fs.h (kernel dependent)
#define _BLOCK_SIZE 4096

MODULE_AUTHOR("Cristiano Milanese");
MODULE_DESCRIPTION("Simple Character Utility for Loading Localities");
MODULE_LICENSE("GPL");

DEFINE_MUTEX(data_lock);

// max allocated row (even if empty)
DEFINE_MUTEX(cursor_lock);

// square matrix holding the data in main memory
static char **matrix = NULL;
// keeping track of where we are in memory for relative indexingas used in llseek
static size_t cursor = 0;

// makes sure the block exists and fills it with data if it did not
static int find_block(int index);

// returns the amount of bytes successfully read, zero if end of file, negative for errors
static ssize_t scull_read(struct file *fp, char __user *buf, size_t count, loff_t *f_pos);

// zero or more for amount of bytes successfully written, negative for errors
static ssize_t scull_write(struct file *fp, const char __user *buf, size_t count, loff_t *f_pos);

// offset the next operation to bytes amount from enum definition
static loff_t scull_llseek(struct file *fp, loff_t offset, int whence);

// picks up a file descriptor for the current process for our device dile
static int scull_open(struct inode *inode, struct file *filp) { return 0; }

static int scull_release(struct inode *inode, struct file *filp) { return 0; }

static struct file_operations scull_fops = {
    .owner = THIS_MODULE,
    .read = scull_read,
    .write = scull_write,
    .open = scull_open,
    .llseek = scull_llseek,
    .release = scull_release,
};

static struct miscdevice scull_dev = {
    .minor = MISC_DYNAMIC_MINOR,
    .name = "scull",
    .fops = &scull_fops
};

// entry and exit point for kernel object
static int __init scull_init(void);
static void __exit scull_exit(void);

#endif
