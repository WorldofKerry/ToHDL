def hrange(base: int, step: int, limit: int) -> int:
    while base < limit:
        yield base
        base += step
