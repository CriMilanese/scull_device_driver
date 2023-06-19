
#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/miscdevice.h>
#include <linux/mutex.h>
#include <linux/uaccess.h>

// BLOCK_SIZE already defined in fs.h
#define _BLOCK_SIZE 4096

MODULE_AUTHOR("Cristiano Milanese");
MODULE_LICENSE("GPL");

DEFINE_MUTEX(data_lock);
DEFINE_MUTEX(cursor_lock);

// square matrix holding the data in main memory
static char **matrix = NULL;
// the number of rows in the matrix, initially a square one
static size_t size = _BLOCK_SIZE;

// makes sure the block exists and fills it with data if it did not
static int find_block(int index){
    int ret = _BLOCK_SIZE;
    mutex_lock(&data_lock);
    if(index >= size) {
        char **res = (char **)kmalloc((2*size)*sizeof(char*), GFP_KERNEL);
        if(res == NULL){
            printk(KERN_ERR "ERROR: unable to allocate memory\n");
		    		ret = -ENOMEM;
		    		goto out;
		    }
        // initialize new pointers to NULL
		    //printk(KERN_INFO "allocating new rows\n");
        for(int i=index; i>=0; i--){
            res[index] = NULL;
        }
        //copy old memory to new memory
        for(int i=size-1; i>=0; i--){
            res[i] = matrix[i];
        }
        matrix = res;
        size *= 2;
    }
    if(matrix[index] == NULL) {
		    //printk(KERN_INFO "allocating new block in row\n");
    	  matrix[index] = (char *)kmalloc(_BLOCK_SIZE*sizeof(char), GFP_KERNEL);
    	  if(matrix[index] == NULL){
    	  		ret = -ENOMEM;
    	  		goto out;
    	  }
			  for(int i=0; i<_BLOCK_SIZE; i++){
			      matrix[index][i] = '\0';
			  }
    }
out:
    mutex_unlock(&data_lock);
    return ret;
}

/*
* in order to behave like a block device driver, we need to return the amount requested
* or less if the read operation could only partially succeed, zero is a proper returning
* value only for asynchronous requests, a block device file with a given size should 
* always return a positive value
*/
static ssize_t scull_read(struct file *fp, char __user *buf, size_t count, loff_t *f_pos) {
    size_t row, offset = 0;
    int bytes = 0, could_not_be_read = 0;

    if(!count) goto out;

    //printk(KERN_INFO "reading %d bytes from %d\n", count, *f_pos);
    row = *f_pos / _BLOCK_SIZE;
    offset = *f_pos % _BLOCK_SIZE;
    //printk(KERN_INFO "searching for index %d to read %d\n", row, count);
    bytes = find_block(row);
    // return error on allocation failure or BLOCK_SIZE
    if(bytes < 0) goto out;

    // handle misalignment
    if(offset > 0){
        bytes -= offset;
    }
    // handle partial read requests (i.e. less than BLOCK_SIZE)
    if(count < bytes) bytes = count;

    //printk(KERN_INFO "finally reading %d\n", bytes);
    mutex_lock(&data_lock);
    could_not_be_read = copy_to_user(buf, &matrix[row][offset], bytes);
    *f_pos += bytes - could_not_be_read;
    mutex_unlock(&data_lock);
out:
    //printk(KERN_INFO "could not read %d chars\n", could_not_be_read);
    return bytes - could_not_be_read;
}

static ssize_t scull_write(struct file *fp, const char __user *buf, size_t count, loff_t *f_pos) {
    int bytes = 0;
    int row, offset = 0;
    int could_not_be_written = 0;
    row = *f_pos / _BLOCK_SIZE;
    offset = *f_pos % _BLOCK_SIZE;
    //printk(KERN_INFO "want to write %ld bytes from %ld\n", count, row);
    bytes = find_block(row);

    // return error on allocation failure or BLOCK_SIZE if everything went well
    if(bytes < 0) {
      printk(KERN_ERR "some error occurres %d\n", bytes);
    	goto out;
    }

    // handle misalignment
    bytes -= offset;
    // handle partial writes (i.e. less than BLOCK_SIZE)
    if(count < bytes) bytes = count;

    mutex_lock(&data_lock);
    // could be a negative value
    //printk(KERN_INFO "writing this many bytes %d\n", bytes);
    could_not_be_written = copy_from_user(&matrix[row][offset], buf, bytes);
    //printk(KERN_INFO "we could not copy %d number of bytes\n", could_not_be_written);
    *f_pos += bytes - could_not_be_written;
    mutex_unlock(&data_lock);
out:
    return bytes - could_not_be_written;
}

static loff_t scull_llseek(struct file *fp, loff_t offset, int whence) {
    mutex_lock(&cursor_lock);
    switch(whence) {
        case SEEK_SET:
            fp->f_pos = offset; 
            break;
        case SEEK_END:
            fp->f_pos = (_BLOCK_SIZE * size) - offset;
            break;
        case SEEK_CUR:
            fp->f_pos += offset;
            break;
        default:
            break;
    }
    mutex_unlock(&cursor_lock);
    return fp->f_pos;
};

static int scull_open(struct inode *inode, struct file *filp)
{
    //printk(KERN_INFO "open operation\n");
    return 0;
}

static int scull_release(struct inode *inode, struct file *filp)
{
    //printk(KERN_INFO "close operation\n");
    return 0;
}

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

static int __init scull_init(void)
{
    // make sure to allocate memory before registering device
    int err = 0;

    matrix = kmalloc(sizeof(char*) * _BLOCK_SIZE, GFP_KERNEL);
		if(matrix == NULL) return -ENOMEM;
    for(int i=0; i < size; i++){
        matrix[i] = NULL;
    }
    err = misc_register(&scull_dev);
    if(err) pr_err("scull device registration failed\n");
    return err;
}

static void __exit scull_exit(void)
{
    for(int i=size-1; i>=0; i--){
        if(matrix[i]) kfree(matrix[i]);
    }
    kfree(matrix);
    misc_deregister(&scull_dev);
}

module_init(scull_init);
module_exit(scull_exit);
