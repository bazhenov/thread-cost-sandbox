use std::{
    hint::black_box,
    sync::{Arc, Barrier},
};

use bytesize::ByteSize;
use memory_stats::{memory_stats, MemoryStats};

fn main() {
    const THREADS: usize = 1000;

    // Warmup
    measure_memory_per_thread(THREADS, small_thread);

    println!("  {:>30}   {:>10} {:>10}", "", "VIRTUAL", "PHYSICAL");
    println!("  {:->30}   {:->10} {:->10}", "", "", "");

    let (usage_during, usage_after) = measure_memory_per_thread(THREADS, small_thread);
    report_usage("small_thread", usage_during);
    report_usage("small_thread (after)", usage_after);
    println!();

    let (usage_during, usage_after) =
        measure_memory_per_thread(THREADS, large_thread::<{ 1024 * 1024 }>);
    report_usage("large_thread (1M)", usage_during);
    report_usage("large_thread (1M) after", usage_after);
}

fn report_usage(name: &str, usage_per_thread: MemoryStats) {
    println!(
        "  {:>30} : {:>10} {:>10}",
        name,
        ByteSize::b(usage_per_thread.virtual_mem as u64),
        ByteSize::b(usage_per_thread.physical_mem as u64)
    );
}

fn measure_memory_per_thread(threads: usize, f: fn(Arc<Barrier>)) -> (MemoryStats, MemoryStats) {
    // Including outselves in waiting group to be able to block thread in running state
    let barrier = Arc::new(Barrier::new(threads + 1));
    let mut handles = vec![];

    let usage_before = memory_stats().unwrap();
    for _ in 0..threads {
        let barrier = Arc::clone(&barrier);
        let handle = std::thread::spawn(move || f(barrier));
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
    let usage_after = memory_stats().unwrap();

    (
        difference(usage_during, usage_before, threads),
        difference(usage_after, usage_before, threads),
    )
}

fn difference(a: MemoryStats, b: MemoryStats, threads: usize) -> MemoryStats {
    let physical_mem = (a.physical_mem.saturating_sub(b.physical_mem)) / threads;
    let virtual_mem = (a.virtual_mem.saturating_sub(b.virtual_mem)) / threads;
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
