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
    int done;
    int output_0;
};
struct FibOutputs fib_next(struct FibMemory *fib, struct FibInputs inputs);
struct FibMemory fib_init(struct FibInputs input);