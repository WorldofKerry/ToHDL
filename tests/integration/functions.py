def circle_lines(centre_x: int, centre_y: int, radius: int) -> tuple[int, int]:
    offset_y = 0
    offset_x = radius
    crit = 1 - radius
    while offset_y <= offset_x:
        yield (centre_x + offset_x, centre_y + offset_y)  # -- octant 1
        yield (centre_x + offset_y, centre_y + offset_x)  # -- octant 2
        yield (centre_x - offset_x, centre_y + offset_y)  # -- octant 4
        yield (centre_x - offset_y, centre_y + offset_x)  # -- octant 3
        yield (centre_x - offset_x, centre_y - offset_y)  # -- octant 5
        yield (centre_x - offset_y, centre_y - offset_x)  # -- octant 6
        yield (centre_x + offset_x, centre_y - offset_y)  # -- octant 8
        yield (centre_x + offset_y, centre_y - offset_x)  # -- octant 7
        offset_y = offset_y + 1
        if crit <= 0:
            crit = crit + 2 * offset_y + 1
        else:
            offset_x = offset_x - 1
            crit = crit + 2 * (offset_y - offset_x) + 1


def fib(n: int) -> int:
    """
    Fibonacci sequence
    """
    a, b = 0, 1
    for _ in p2vrange(0, n, 1):
        yield a
        a, b = b, a + b


def floor_div(n) -> tuple[int]:
    i = 1
    while i < n:
        j = 1
        while j < n:
            yield i // j
            j += 1
        i += 1


def happy_face(s_x, s_y, height):
    # Generate points for the outer circle
    x = 0
    y = height
    d = 3 - 2 * height
    yield (s_x + x, s_y + y)
    yield (s_x + x, s_y - y)
    yield (s_x - x, s_y + y)
    yield (s_x - x, s_y - y)
    yield (s_x + y, s_y + x)
    yield (s_x + y, s_y - x)
    yield (s_x - y, s_y + x)
    yield (s_x - y, s_y - x)
    while y >= x:
        x = x + 1
        if d > 0:
            y = y - 1
            d = d + 4 * (x - y) + 10
        else:
            d = d + 4 * x + 6
        # yield (x, y, d)
        yield (s_x + x, s_y + y)
        yield (s_x + x, s_y - y)
        yield (s_x - x, s_y + y)
        yield (s_x - x, s_y - y)
        yield (s_x + y, s_y + x)
        yield (s_x + y, s_y - x)
        yield (s_x - y, s_y + x)
        yield (s_x - y, s_y - x)

    # Generate points for the eyes
    rectangle_width = height // 3
    rectangle_height = height // 3

    # Left eye
    x = s_x + 10
    y = s_y + 5

    # Rectangle
    i, j = 0, 0
    while i < rectangle_width:
        while j < rectangle_height:
            yield (x + i, y + j)
            j += 1
        j = 0
        i += 1

    # Right eye
    x = s_x - 10
    y = s_y + 5

    # Rectangle
    i, j = 0, 0
    while i < rectangle_width:
        while j < rectangle_height:
            yield (x + i, y + j)
            j += 1
        j = 0
        i += 1


def multiplier_generator(multiplicand: int, multiplier: int) -> int:
    product = 0
    count = 0
    while count < multiplier:
        product += multiplicand
        count += 1
    yield product


def operators(x, y):
    yield 0, x
    yield 1, y

    # Arithmetic operators
    yield 2, x + y
    yield 3, x - y
    yield 4, x * y
    # yield x / y

    if y != 0:
        yield 5, x // y
        yield 6, x % y
    # yield x**y

    # Comparison operators
    yield 7, x == x
    yield 8, x == -x
    yield 9, x == y
    yield 10, x != y
    yield 11, x < y
    yield 12, x > y
    yield 13, x <= y
    yield 14, x >= y

    yield 88888888, 88888888  # delimiter

    # # Logical operators
    # yield x and y
    # yield x or y
    # yield not x

    # # Bitwise operators
    # yield x & y
    # yield x | y
    # yield x ^ y
    # yield ~x
    # yield x << y
    # yield x >> y

    # # Assignment operators
    # z = x
    # yield z
    # z += y
    # yield z
    # z -= y
    # yield z
    # z *= y
    # yield z
    # z /= y
    # yield z
    # z //= y
    # yield z
    # z %= y
    # yield z
    # z **= y
    # yield z
    # z &= y
    # yield z
    # z |= y
    # yield z
    # z ^= y
    # yield z
    # z <<= y
    # yield z
    # z >>= y
    # yield z

    # # Identity and membership operators
    # yield x is y
    # yield x is not y
    # yield x in [y, z]
    # yield x not in [y, z]


def rectangle_filled(s_x, s_y, height, width):
    i0 = 0
    while i0 < width:
        i1 = 0
        while i1 < height:
            yield (s_x + i1, s_y + i0)
            i1 = i1 + 1
        i0 = i0 + 1


def rectangle_lines(s_x, s_y, height, width):
    i0 = 0
    while i0 < width:
        yield (s_x, s_y + i0)
        yield (s_x + height - 1, s_y + i0)
        i0 += 1

    i1 = 0
    while i1 < height:
        yield (s_x + i1, s_y)
        yield (s_x + i1, s_y + width - 1)
        i1 += 1


def division(dividend, divisor, precision):
    iter = 0
    if dividend < 0:
        dividend = -dividend
    while dividend > 0 and iter <= precision:
        digit = 0
        while (digit + 1) * divisor <= dividend:
            digit += 1
        yield digit
        dividend -= digit * divisor
        if dividend // divisor == 0:
            dividend *= 10
        iter += 1


def olympic_logo_naive(mid_x, mid_y, radius):
    spread = radius - 2
    gen = circle_lines(mid_x, mid_y + spread, radius)
    for x, y in gen:
        yield x, y, 50
    gen = circle_lines(mid_x + spread * 2, mid_y + spread, radius)
    for x, y in gen:
        yield x, y, 180
    gen = circle_lines(mid_x - spread * 2, mid_y + spread, radius)
    for x, y in gen:
        yield x, y, 500
    gen = circle_lines(mid_x + spread, mid_y - spread, radius)
    for x, y in gen:
        yield x, y, 400
    gen = circle_lines(mid_x - spread, mid_y - spread, radius)
    for x, y in gen:
        yield x, y, 300


def p2vrange(start: int, stop: int, step: int) -> int:
    """
    Simplified version of Python's built-in range function
    """
    while start < stop:
        yield start
        start += step


def dupe(base: int, limit: int, step: int) -> int:
    """
    Dupe hrange
    """
    inst = p2vrange(base, limit, step)
    for out in inst:
        yield out
        yield out


def double_for(limit: int) -> tuple[int, int]:
    """
    Double for loop
    """
    x_gen = p2vrange(0, limit, 1)
    for x in x_gen:
        y_gen = p2vrange(0, limit, 1)
        for y in y_gen:
            yield x, y


def olympic_logo_mids(mid_x: int, mid_y: int, spread: int) -> tuple[int, int, int]:
    """
    Yields the middle coordinates and the color
    for the 5 circles in the olympics logo
    """
    yield mid_x, mid_y + spread, 50
    yield mid_x + spread * 2, mid_y + spread, 180
    yield mid_x - spread * 2, mid_y + spread, 500
    yield mid_x + spread, mid_y - spread, 400
    yield mid_x - spread, mid_y - spread, 300


def olympic_logo(mid_x, mid_y, radius):
    """
    Draws the olympic logo
    """
    spread = radius - 2
    middles_and_colors = olympic_logo_mids(mid_x, mid_y, spread)
    for x, y, color in middles_and_colors:
        coords = circle_lines(x, y, radius)
        for x, y in coords:
            yield x, y, color


def keyword_test():
    """
    Testing for break, continue, return
    """
    i = 0
    while i < 10:
        if i % 2 == 0:
            i += 1
            continue
        yield i
        i += 3

    i = 0
    while i < 10:
        if i == 7:
            break
        yield i
        i += 1

    i = 0
    while i < 10:
        if i == 5:
            return
        yield i
        i += 1


def quad_multiply(left, right):
    """
    Given left and right,
    yields
    left * right
    left * -right
    -left * right
    -left * -right
    """
    inst = multiplier_generator(left, right)
    for val in inst:
        yield val
    inst = multiplier_generator(left, -right)
    for val in inst:
        yield val
    inst = multiplier_generator(-left, right)
    for val in inst:
        yield val
    for val in multiplier_generator(-left, -right):
        yield val


def multiplier(multiplicand: int, multiplier: int) -> int:
    product = 0
    while multiplier > 0:
        product += multiplicand
        multiplier -= 1
    return product


def fib_product(n):
    """
    Yields the product of the first n fibonacci numbers
    """
    for num in fib(n):
        prod = multiplier(num, num)
        yield prod


def multi_funcs(a, b):
    """
    Testing multiple function calls and tested function calls
    """
    temp = multiplier(a, b)
    yield temp
    temp = multiplier(a + 10, b)
    yield temp
    for i in p2vrange(0, 2, 1):
        yield i
    for i in p2vrange(0, 2, 1):
        yield i
    for i in p2vrange(0, 2, 1):
        yield i
        for i in p2vrange(0, 2, 1):
            yield i


def floating_point_add(a_sign, a_exponent, a_mantissa, b_sign, b_exponent, b_mantissa):
    """
    ieee754 binary32
    """
    # Make sure a has larger by magnitude
    if a_exponent < b_exponent:
        temp_sign = a_sign
        a_sign = b_sign
        b_sign = temp_sign

        temp_exponent = a_exponent
        a_exponent = b_exponent
        b_exponent = temp_exponent

        temp_mantissa = a_mantissa
        a_mantissa = b_mantissa
        b_mantissa = temp_mantissa

    elif a_exponent == b_exponent:
        if a_mantissa < b_mantissa:
            temp_sign = a_sign
            a_sign = b_sign
            b_sign = temp_sign

            temp_exponent = a_exponent
            a_exponent = b_exponent
            b_exponent = temp_exponent

            temp_mantissa = a_mantissa
            a_mantissa = b_mantissa
            b_mantissa = temp_mantissa

    yield a_mantissa
    yield a_exponent
    yield b_mantissa
    yield b_exponent

    c_sign = a_sign

    # Add implicit one
    a_mantissa |= 1 << 23
    b_mantissa |= 1 << 23

    yield c_sign
    yield c_sign
    yield a_mantissa

    # Adjust the smaller mantissa so exponents are same
    exponent_difference = a_exponent - b_exponent
    b_mantissa >>= exponent_difference

    subtract = a_sign ^ b_sign

    if subtract:
        c_mantissa = a_mantissa - b_mantissa
    else:
        c_mantissa = a_mantissa + b_mantissa

    c_exponent = a_exponent

    # Normalize
    msb_index = 0
    temp = c_mantissa

    while temp:

        # Logical shift right (Python only does arithmetic)
        if temp >= 0:
            temp >>= 1
        else:
            temp = (temp + (1 << 24)) >> 1

        msb_index += 1

    yield msb_index

    # Shift left until implicit bit is MSB
    # Decrease exponent to match
    left_shift_amount = 24 - msb_index  # Can be negative
    if left_shift_amount >= 0:
        c_mantissa <<= left_shift_amount
        c_exponent -= left_shift_amount
    else:
        c_mantissa >>= -left_shift_amount
        c_exponent += -left_shift_amount

    c_mantissa &= (1 << 23) - 1

    yield c_sign
    yield c_exponent
    yield c_mantissa


def seven_seg(n: int) -> int:
    """
    Decimal digit to 7 segment display
    0 means on, 1 means off
    """
    if n == 0:
        return 192
    elif n == 1:
        return 249
    elif n == 2:
        return 164
    elif n == 3:
        return 176
    elif n == 4:
        return 153
    elif n == 5:
        return 146
    elif n == 6:
        return 130
    elif n == 7:
        return 248
    elif n == 8:
        return 128
    elif n == 9:
        return 144
    else:
        return 0


def mod_10(n: int) -> int:
    """
    Computes n % 10
    """
    mod = 0
    quo = -1
    count = 0
    while count < n:
        quo += 1
        count += 10
    mod = n - quo * 10
    if mod == 10:
        return 0
    else:
        return mod


def div_10(n: int) -> int:
    """
    Computes floor(n / 10)
    """
    mod = 0
    quo = -1
    count = 0
    while count < n:
        quo += 1
        count += 10
    mod = n - quo * 10
    if mod == 10:
        return quo + 1
    return quo


def binary_to_7_seg(n: int) -> int:
    """
    Converts binary number to 7-segment display
    """
    ret = 0

    count = 0
    while count < 4:

        mod = mod_10(n)
        n = div_10(n)

        hex0 = seven_seg(mod)

        temp = (hex0 << (count * 7)) & 127
        ret = ret | temp

        count += 1

    return ret


def fib_to_7_seg(n):
    a = 0
    b = 1

    count = 0
    while count < n:
        h = binary_to_7_seg(a)
        yield h

        temp = b
        b = a + b
        a = temp
        # a, b = b, a + b

        count += 1


def callee(a: int, b: int) -> int:
    c = a + b
    return 5


def caller(a: int, n: int):
    count = 0
    while count < n:
        c = callee(a, count)
        yield c
        count += 1
