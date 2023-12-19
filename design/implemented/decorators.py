import atexit
import inspect
import logging
from functools import wraps
from types import FunctionType

all_functions = {}


def decorator_with_args(func: FunctionType):
    """
    a decorator decorator, allowing the decorator to be used as:
    @decorator(with, arguments, and=kwargs)
    or
    @decorator

    Note: can't distinguish between a function as a parameter vs
    a function to-be decorated
    """

    @wraps(func)
    def new_dec(*args, **kwargs):
        if len(args) == 1 and len(kwargs) == 0 and isinstance(args[0], FunctionType):
            # actual decorated function
            return func(args[0])
        else:
            # decorator arguments
            return lambda realf: func(realf, *args, **kwargs)

    return new_dec


@decorator_with_args
def verilogify(func: FunctionType, module_path: str = "", testbench_path: str = ""):
    """
    :param module_path: path to write verilog module to, defaults to function_name.sv
    :param testbench_path: path to write verilog testbench to, defaults to function_name_tb.sv

    """
    print("Created verilogify")

    @wraps(func)
    def generator_wrapper(*args, **kwargs):
        print(f"Called with {str(*args)} {str(**kwargs)}")

        all_functions[func] = {"func": func, "source": inspect.getsource(func)}

        @wraps(func)
        def __generator(*args, **kwargs):
            """
            Required to executate function up until first yield
            """
            for result in func(*args, **kwargs):
                yield result

        return __generator(*args, **kwargs)

    @wraps(func)
    def function_wrapper(*args, **kwargs):
        logging.error("Non-generator functions currently not supported")
        return func(*args, **kwargs)

    return generator_wrapper if inspect.isgeneratorfunction(func) else function_wrapper


@verilogify
def my_counter(n):
    i = 0
    while i < n:
        yield (i)
        i += 1


@verilogify(module_path="ayy lmao")
def my_counter2(n):
    i = 0
    while i < n:
        yield (i)
        i += 1


for value in my_counter(5):
    print(value)
my_counter(10)
my_counter(15)

for key, value in all_functions.items():
    print(type(key), type(value))


def exit_handler():
    print(all_functions)
    for key, value in all_functions.items():
        print(type(key), type(value))


atexit.register(exit_handler)
