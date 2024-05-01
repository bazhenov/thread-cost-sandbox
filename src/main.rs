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

    let usage = measure_memory_per_thread(THREADS, small_thread);
    report_usage("small_thread", usage);

    let usage = measure_memory_per_thread(THREADS, large_thread::<{ 1024 * 1024 }>);
    report_usage("large_thread (1M)", usage);
}

fn report_usage(name: &str, usage_per_thread: MemoryStats) {
    println!("=== Memory usage per thread : {} ===", name);
    println!(
        "  Physical: {:>10}",
        ByteSize::b(usage_per_thread.physical_mem as u64)
    );
    println!(
        "   Virtual: {:>10}",
        ByteSize::b(usage_per_thread.virtual_mem as u64)
    );
}

fn measure_memory_per_thread(threads: usize, f: fn(Arc<Barrier>)) -> MemoryStats {
    // Inclusing outselves in waiting group to be able to block thread in running state
    let barrier = Arc::new(Barrier::new(threads + 1));
    let mut handles = vec![];

    let usage_before = memory_stats().unwrap();
    for _ in 0..threads {
        let barrier = Arc::clone(&barrier);
        let handle = std::thread::spawn(move || f(barrier));
        handles.push(handle);
    }
    barrier.wait();
    let usage_after = memory_stats().unwrap();

    barrier.wait();
    for handle in handles.into_iter() {
        handle.join().unwrap();
    }

    let physical_mem = (usage_after.physical_mem - usage_before.physical_mem) / threads;
    let virtual_mem = (usage_after.virtual_mem - usage_before.virtual_mem) / threads;
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
    let _: [u8; S] = black_box([0; S]);
    barrier.wait();
    barrier.wait();
}
