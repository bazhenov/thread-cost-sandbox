use bytesize::ByteSize;
use memory_stats::{memory_stats, MemoryStats};
use std::{
    hint::black_box,
    sync::{Arc, Barrier},
    thread,
};

fn main() {
    const THREADS: usize = 1000;

    // Warmup
    measure_memory(THREADS, small_thread);

    println!(
        "  {:>30}   {:>15} {:>15} {:>15}",
        "", "VIRT", "VIRT (thread)", "PHYS (thread)"
    );
    println!("  {:->30}   {:->15} {:->15} {:->15}", "", "", "", "");

    let usage = measure_memory(THREADS, small_thread);
    report_usage("small_thread", usage, THREADS);

    let usage = measure_memory(THREADS, large_thread::<{ 1024 * 1024 }>);
    report_usage("large_thread (1M)", usage, THREADS);
}

fn report_usage(name: &str, usage: MemoryStats, threads: usize) {
    println!(
        "  {:>30} : {:>15} {:>15} {:>15}",
        name,
        ByteSize::b(usage.virtual_mem as u64),
        ByteSize::b((usage.virtual_mem / threads) as u64),
        ByteSize::b((usage.physical_mem / threads) as u64)
    );
}

fn measure_memory(threads: usize, f: fn(Arc<Barrier>)) -> MemoryStats {
    // Including outselves in waiting group to be able to block thread in running state
    let barrier = Arc::new(Barrier::new(threads + 1));
    let mut handles = vec![];

    let usage_before = memory_stats().unwrap();
    for _ in 0..threads {
        let barrier = Arc::clone(&barrier);
        let handle = thread::Builder::new()
            .stack_size(1024 * 1024 * 1024) // 1GB thread stack
            .spawn(move || f(barrier))
            .expect("Unable to spawn thread");
        handles.push(handle);
    }

    // For measurements to be precise we need for tested function to wait on barrier twice.
    // This way we can measure memory usage at all threads being at precise the same place in code.
    barrier.wait();
    let usage_during = memory_stats().unwrap();
    barrier.wait();

    for handle in handles.into_iter() {
        handle.join().unwrap();
    }

    difference(usage_during, usage_before)
}

fn difference(a: MemoryStats, b: MemoryStats) -> MemoryStats {
    let physical_mem = a.physical_mem.saturating_sub(b.physical_mem);
    let virtual_mem = a.virtual_mem.saturating_sub(b.virtual_mem);
    MemoryStats {
        physical_mem,
        virtual_mem,
    }
}

fn small_thread(barrier: Arc<Barrier>) {
    barrier.wait();
    barrier.wait();
}
fn large_thread<const S: usize>(barrier: Arc<Barrier>) {
    large_allocate::<S>();
    barrier.wait();
    barrier.wait();
}

#[inline(never)]
fn large_allocate<const S: usize>() {
    black_box([0u8; S]);
}
