CC = gcc
CFLAGS = -Wall -Wextra -O2 -I../include
LDFLAGS = -L../../target/release -lqsae_c

TARGET = example

all: $(TARGET)

$(TARGET): example.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

clean:
	rm -f $(TARGET)

.PHONY: all clean
