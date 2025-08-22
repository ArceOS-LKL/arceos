#ifndef _SYS_UIO_H
#define _SYS_UIO_H

#include <stddef.h>

struct iovec {
    void *iov_base; /* Pointer to data.  */
    size_t iov_len; /* Length of data.  */
};
ssize_t readv(int, const struct iovec *, int);
ssize_t writev(int, const struct iovec *, int);

#if defined(_GNU_SOURCE) || defined(_BSD_SOURCE)
ssize_t preadv(int, const struct iovec *, int, off_t);
ssize_t pwritev(int, const struct iovec *, int, off_t);
#if defined(_LARGEFILE64_SOURCE)
#define preadv64  preadv
#define pwritev64 pwritev
#define off64_t   off_t
#endif
#endif

#endif
