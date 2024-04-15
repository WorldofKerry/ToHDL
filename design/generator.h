#ifndef FIB_H
#define FIB_H

struct FibMemory
{
    int state;
    int mem_0;
    int mem_1;
    int mem_2;
    int mem_3;
};
struct FibInputs
{
    int in_0;
};
struct FibOutputs
{
    int valid;
    int output_0;
};
struct FibOutputs fib_next(struct FibMemory *fib, struct FibInputs inputs);
struct FibMemory fib_init(int n);
static int const FIB_STATE_DONE = -1;

#endif
