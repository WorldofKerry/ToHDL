#include <stdio.h>
#include "generator.h"

int main()
{
    int sum = 0;
    struct FibMemory fib = fib_init((struct FibInputs){.in_0 = 50});
    for (;;)
    {
        struct FibOutputs fib_out = fib_next(&fib, (struct FibInputs){});
        if (fib_out.valid)
        {
            sum += fib_out.output_0;
        }
        else if (fib_out.done)
        {
            break;
        }
    }
    printf("sum %d\n", sum);
}