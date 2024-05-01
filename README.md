This repository contains test application for exploring stack allocation policies on different OSes

# Treads are costly because of the stack

There is an opinion that OS stacks size is main contributor why threads are expensive.
In this demo I show that OS never allocates space in physical memory until application are allocating memory on a stack. Only virtual memory is allocated that is not backed by physical RAM.

## How to run

```console
$ cargo run --release
                                      VIRTUAL   PHYSICAL
  ------------------------------   ---------- ----------
                    small_thread :     2.3 MB    16.6 KB
            small_thread (after) :     1.0 KB      262 B

               large_thread (1M) :     2.3 MB     1.1 MB
         large_thread (1M) after :     9.4 KB      229 B
```
