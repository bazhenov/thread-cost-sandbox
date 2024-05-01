This repository contains test application for exploring stack allocation policies on different OSes

# Treads are costly because of the stack

There is an opinion that OS stacks size is main contributor why threads are expensive.
In this demo I show that OS never allocates space in physical memory until application are allocating memory on a stack. Only virtual memory is allocated that is not backed by physical RAM.

## How to run

```console
$ cargo run --release
                                              VIRT   VIRT (thread)   PHYS (thread)
  ------------------------------   --------------- --------------- ---------------
                    small_thread :       1073.9 GB          1.1 GB         16.7 KB
               large_thread (1M) :       1073.9 GB          1.1 GB          1.1 MB
```
