#ifndef __TIMER_H__
#define __TIMER_H__

/**
 * Normally these functions will be defined in time.h,
 * but they need to be defined in a separate header file(here is timer.h)
 * because they form a circular dependency with the structures they rely on,
 * which can cause forward dependency bugs and affect the code generation(See
 * https://github.com/arceos-org/arceos/pull/288).
 *
 * When they are defined in time.h, the `sigevent` structure depends on `signal.h`,
 * which depends on `pthread.h` because of the `pthread_attr_t` field. But the
 * `pthread.h` depends on `time.h` because of the `clock_t` field.
 *
 * So we need to define them in a separate header file.
 */

#include <signal.h>

struct sigevent;
int timer_create(clockid_t, struct sigevent *__restrict, timer_t *__restrict);
int timer_delete(timer_t);
int timer_settime(timer_t, int, const struct itimerspec *__restrict, struct itimerspec *__restrict);

#endif // __TIMER_H__