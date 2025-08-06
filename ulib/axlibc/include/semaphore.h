
typedef struct {
    volatile int __val[4 * sizeof(long) / sizeof(int)];
} sem_t;

int sem_destroy(sem_t *sem);
int sem_init(sem_t *sem, int pshared, unsigned value);
sem_t *sem_open(const char *name, int oflag, int mode, unsigned value);
int sem_post(sem_t *sem);
int sem_wait(sem_t *sem);