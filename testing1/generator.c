/*
def even_fib():
    i = 0
    a = 0
    b = 1
    while a < n:
        if a % 2:
            yield a
        temp = a + b
        a = b
        b = temp
        i = i + 1
*/

#include <stdio.h>

struct FibMemory
{
    int state;
    int mem_0;
    int mem_1;
    int mem_2;
    int mem_3;
};

struct FibInit
{
    int n;
};

struct FibInputs
{
};

struct FibOutputs
{
    int valid;
    int done;
    int output_0;
};

struct FibOutputs fib_state_0(struct FibMemory *fib, struct FibInputs inputs)
{
    struct FibOutputs outputs = {.valid = 0, .done = 0};
    struct FibMemory original = *fib;
    if ((0 < fib->mem_0))
    {
        if ((0 % 2))
        {
            outputs.valid = 1;
            outputs.output_0 = 0;
            fib->mem_0 = 1;
            fib->mem_1 = (0 + 1);
            fib->mem_2 = (0 + 1);
            fib->mem_3 = original.mem_0;
            fib->state = 1;
        }
        else
        {
            fib->mem_0 = 1;
            fib->mem_1 = (0 + 1);
            fib->mem_2 = (0 + 1);
            fib->mem_3 = original.mem_0;
            fib->state = 1;
        }
    }
    else
    {
        fib->state = -1;
    }
    return outputs;
}

struct FibOutputs fib_state_1(struct FibMemory *fib, struct FibInputs inputs)
{
    struct FibOutputs outputs = {.valid = 0, .done = 0};
    struct FibMemory original = *fib;
    if ((original.mem_0 < original.mem_3))
    {
        if ((original.mem_0 % 2))
        {
            outputs.valid = 1;
            outputs.output_0 = original.mem_0;
            fib->mem_0 = original.mem_1;
            fib->mem_1 = (original.mem_0 + original.mem_1);
            fib->mem_2 = (original.mem_2 + 1);
            fib->mem_3 = original.mem_3;
            fib->state = 1;
        }
        else
        {
            fib->mem_0 = original.mem_1;
            fib->mem_1 = (original.mem_0 + original.mem_1);
            fib->mem_2 = (original.mem_2 + 1);
            fib->mem_3 = original.mem_3;
            fib->state = 1;
        }
    }
    else
    {
        fib->state = -1;
    }
    return outputs;
}

static const struct FibOutputs (*FIB_STATE_DISPATCH[2])(struct FibMemory *fib, struct FibInputs inputs) = {fib_state_0,
                                                                                                           fib_state_1};

struct FibMemory fib_init(struct FibInit input)
{
    return (struct FibMemory){
        .state = 0,
        .mem_0 = input.n};
}

struct FibOutputs fib_next(struct FibMemory *fib, struct FibInputs inputs)
{
    if (fib->state == -1)
    {
        // If done state
        return (struct FibOutputs){.done = 1, .valid = 0};
    }
    return FIB_STATE_DISPATCH[fib->state](fib, inputs);
}

int main()
{
    int sum = 0;
    struct FibMemory fib = fib_init((struct FibInit){.n = 50});
    struct FibOutputs fib_out = fib_next(&fib, (struct FibInputs){});
    while (1)
    {
        if (fib_out.valid)
        {
            // For loop body
            sum += fib_out.output_0;
        }
        else if (fib_out.done)
        {
            break;
        }
        fib_out = fib_next(&fib, (struct FibInputs){});
    }
    printf("sum %d\n", sum);
}