#include <stdio.h>
#include "generator.h"

int main(void)
{
    int sum = 0;
    for (struct FibMemory fib = fib_init(50);
         fib.state != FIB_STATE_DONE;)
    {
        struct FibOutputs fib_out = fib_next(&fib, (struct FibInputs){0});
        if (fib_out.valid)
        {
            sum += fib_out.output_0;
        }
    }
    printf("sum %d\n", sum);
}