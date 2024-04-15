def sumer():
    sum = 0
    while True:
        sum += yield sum


er = sumer()
next(er)
for i in range(10):
    print(er.send(i))
