#include "scull.h"

static int find_block(int index){
    int ret = _BLOCK_SIZE;

    mutex_lock(&data_lock);
    if(index >= cursor) {
        // try to allocate new size and memcpy the old content
        matrix = krealloc(matrix, (index+1)*sizeof(char*), GFP_KERNEL);
        if(matrix == NULL){
            printk(KERN_ERR "ERROR: unable to allocate memory\n");
		    		ret = -ENOMEM;
		    		goto out;
		    }
        for(int i = index; i >= cursor; i--){
            matrix[i] = NULL;
        }
        cursor = index + 1;
    }
    if(matrix[index] == NULL) {
    	  matrix[index] = krealloc(matrix[index], _BLOCK_SIZE*sizeof(char), GFP_KERNEL);
    	  if(matrix[index] == NULL){
            printk(KERN_ERR "ERROR: unable to allocate memory for this block\n");
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

static ssize_t scull_read(struct file *fp, char __user *buf, size_t count, loff_t *f_pos) {
    size_t row, offset = 0;
    int bytes = 0, could_not_be_read = 0;

    if(!count) goto out;

    row = *f_pos / _BLOCK_SIZE;
    offset = *f_pos % _BLOCK_SIZE;

    bytes = find_block(row);

    if(bytes < 0) goto out;

    // handle misalignment
    bytes -= offset;
    // handle partial read requests (i.e. less than _BLOCK_SIZE)
    if(count < bytes) bytes = count;

    mutex_lock(&data_lock);
    could_not_be_read = copy_to_user(buf, &matrix[row][offset], bytes);
    *f_pos += bytes - could_not_be_read;
    mutex_unlock(&data_lock);
out:
    return bytes - could_not_be_read;
}

static ssize_t scull_write(struct file *fp, const char __user *buf, size_t count, loff_t *f_pos) {
    int bytes = 0;
    int row, offset = 0;
    int could_not_be_written = 0;
    row = *f_pos / _BLOCK_SIZE;
    offset = *f_pos % _BLOCK_SIZE;

    if(!count) goto out;

    bytes = find_block(row);

    if(bytes < 0) {
      printk(KERN_ERR "some error occurres %d\n", bytes);
    	goto out;
    }

    // handle misalignment
    bytes -= offset;
    // handle partial writes
    if(count < bytes) bytes = count;

    mutex_lock(&data_lock);
    could_not_be_written = copy_from_user(&matrix[row][offset], buf, bytes);
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
            fp->f_pos = (_BLOCK_SIZE * cursor) - offset;
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

static int __init scull_init(void)
{
    int err = 0;

    mutex_lock(&data_lock);
    matrix = krealloc(matrix, sizeof(char*) * _BLOCK_SIZE, GFP_KERNEL);
		if(matrix == NULL) return -ENOMEM;
    for(int i=_BLOCK_SIZE; i>=0; i--){
        matrix[i] = NULL;
    }
    cursor = _BLOCK_SIZE;
    mutex_unlock(&data_lock);
    err = misc_register(&scull_dev);
    if(err) pr_err("scull device registration failed\n");
    return err;
}

static void __exit scull_exit(void)
{
    mutex_lock(&data_lock);
    for(int i=cursor-1; i>=0; i--){
        if(matrix[i]) kfree(matrix[i]);
    }
    kfree(matrix);
    mutex_unlock(&data_lock);
    misc_deregister(&scull_dev);
}

module_init(scull_init);
module_exit(scull_exit);
