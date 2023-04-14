#include <linux/init.h>
#include <linux/module.h>

#include <linux/kdev_t.h> // MAJOR, MKDEV
#include <linux/fs.h>     // register_chrdev_region, alloc_chrdev_region
#include <linux/cdev.h>   // cdev

MODULE_AUTHOR("Cristiano Milanese");
MODULE_LICENSE("GPL");

int my_null_major = 0;
int my_null_minor = 0;

struct cdev cdev;

ssize_t my_null_read(struct file *filp, char __user *buf, size_t count,
                     loff_t *f_pos)
{
    // dev_read returns the number of bytes read. 0 indicates the end
    printk(KERN_INFO "reading coffee grounds");
    return 0;
}

ssize_t my_null_write(struct file *filp, const char __user *buf, size_t count,
                      loff_t *f_pos)
{
    // dev_write returns the number of bytes read.
    // Just say to the user you read everything and return
    printk(KERN_INFO "writing to the wind");
    *f_pos += count;
    return count;
}

int my_null_open(struct inode *inode, struct file *filp)
{
    printk(KERN_INFO "open operation");
    return 0;
}

int my_null_release(struct inode *inode, struct file *filp)
{
    printk(KERN_INFO "close operation");
    return 0;
}

static struct file_operations my_null_fops = {
    .owner = THIS_MODULE,
    .read = my_null_read,
    .write = my_null_write,
    .open = my_null_open,
    .release = my_null_release,
};

static int my_null_setup_device(void)
{
    int err = 0, devno = MKDEV(my_null_major, my_null_minor);

    cdev_init(&cdev, &my_null_fops);
    cdev.owner = THIS_MODULE;
    err = cdev_add(&cdev, devno, 1);

    return err;
}

static int __init my_null_init(void)
{
    int result = 0;
    dev_t devno = 0;

    if (my_null_major)
    {
        devno = MKDEV(my_null_major, my_null_minor);
        result = register_chrdev_region(devno, 1, "my_null");
    }
    else
    {
        result = alloc_chrdev_region(&devno, my_null_minor, 1, "my_null");
        my_null_major = MAJOR(devno);
    }

    if (unlikely(result < 0))
    {
        printk(KERN_ERR "my_null: can't get major %d\n", my_null_major);
        goto out;
    }

    result = my_null_setup_device();
    if (unlikely(result < 0))
    {
        printk(KERN_WARNING "my_null: can't setup device\n");
        goto out;
    }

    printk(KERN_INFO "my_null: setup with major %d and minor %d\n", my_null_major, my_null_minor);

out:
    return result;
}

static void __exit my_null_exit(void)
{
    dev_t devno = MKDEV(my_null_major, my_null_minor);
    cdev_del(&cdev);
    unregister_chrdev_region(devno, 1);
}

module_init(my_null_init);
module_exit(my_null_exit);
