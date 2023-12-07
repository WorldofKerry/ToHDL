CC = gcc
CFLAGS = -Wall -Werror

all: program

program: main.c generator.c generator.h
	$(CC) $(CFLAGS) $^ -o $@

clean:
	rm -f program
