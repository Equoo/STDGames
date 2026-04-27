CC      = gcc
CFLAGS  = -Wall -Wextra -O2 -fPIC -D_GNU_SOURCE -U_FORTIFY_SOURCE
LDFLAGS = -shared -ldl -lpthread
TARGET  = libhttpfs_preload.so

SRCS = src/shim.c src/cache.c src/fetch.c src/gc.c
OBJS = $(SRCS:.c=.o)
INC  = -Iinclude

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(LDFLAGS) -o $@ $^

%.o: %.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

clean:
	rm -f $(OBJS) $(TARGET)

install: $(TARGET)
	cp $(TARGET) $(HOME)/.local/lib/
