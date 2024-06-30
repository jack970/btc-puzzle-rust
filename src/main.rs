use num_bigint::{BigInt, ToBigInt};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc,
    },
    time::Instant,
};

mod utils;

extern crate num_cpus;

fn main() {
    let num_threads: usize = num_cpus::get() / 2;
    println!("Threads detectadas: {}", num_threads);

    let (min, max, wallet) = utils::escolher_carteira();
    let chunk_size = (&max - &min) / num_threads;

    let (tx, rx) = mpsc::channel(); // Canal para enviar chaves privadas

    let found = Arc::new(AtomicBool::new(false));

    let start_time = Instant::now();

    let hash_count = Arc::new(AtomicUsize::new(0));
    (0..num_threads).into_par_iter().for_each(|i| {
        println!("dividindo {}", i);
        let start = &min + (&chunk_size * i);
        let end = if i == num_threads - 1 {
            max.clone()
        } else {
            &start + &chunk_size - 1
        };

        let mut current = start.clone();
        while current <= end && !found.load(Ordering::Relaxed) {
            tx.send(current.clone()).unwrap();
            current += 1.to_bigint().unwrap();
            hash_count.fetch_add(1, Ordering::Relaxed);
        }
    });

    while let Ok(private_key) = rx.recv() {
        let hash160 = utils::criar_chave_publica160(private_key.clone());

        if wallet == hash160 {
            println!("Chave Privada: {}", private_key.to_string());
            found.store(true, Ordering::Relaxed);
            break; // Saia do loop ao encontrar a chave
        }
    }

    println!("Começa a percorrer vector chaves");

    let elapsed = start_time.elapsed();

    println!(r#"chaves geradas: {}"#, hash_count.load(Ordering::Relaxed));
    println!("Tempo: {}", elapsed.as_secs_f64());
}
