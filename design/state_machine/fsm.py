from enum import Enum


def card_reader(type_2_starter=420):
    States = Enum("States", ["WAIT", "TYPE2", "ECHO"])

    state = States.WAIT
    while True:
        if state == States.WAIT:
            input = yield
            if not input:
                continue
            if input[0] == type_2_starter:
                state = States.TYPE2
                buffer = input[1:]
            else:
                state = States.ECHO
                buffer = input

        elif state == States.TYPE2:
            input = yield
            if not input:
                continue
            buffer += input
            state = States.ECHO

        elif state == States.ECHO:
            yield buffer
            state = States.WAIT


if __name__ == "__main__":
    fsm = card_reader()
    next(fsm)

    inputs = [
        [],
        [1, 2, 3, 4],
        [],
        [],
        [420, 7, 8, 9],
        [],
        [10, 11, 12, 13],
        [],
        [1, 2, 3, 4],
    ]
    for input in inputs:
        print(fsm.send(input))
