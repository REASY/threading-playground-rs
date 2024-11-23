mod util;

use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::util::{thread_builder, BankAccount};

fn transfer(
    amount: u128,
    from: &Mutex<BankAccount>,
    to: &Mutex<BankAccount>,
) -> Result<(), std::io::Error> {
    let id = thread::current();
    // Consistent locking order based on memory addresses of mutexes
    let from_ptr = std::ptr::addr_of!(*from);
    let to_ptr = std::ptr::addr_of!(*to);
    // Derive first and second mutexes, and a flag whether `first` != `from` => is_swapped == true
    let (first, second, is_swapped) = if from_ptr.le(&to_ptr) {
        (from, to, false)
    } else {
        (to, from, true)
    };
    println!("{:?} Acquiring mutex `first`: {:?}", id, first);
    let mut first_guard = first.lock().unwrap();
    println!("{:?} Acquired mutex `first`: {:?}", id, first_guard);

    thread::sleep(Duration::from_millis(100));

    println!("{:?} Acquiring mutex `second`: {:?}", id, second);
    let mut second_guard = second.lock().unwrap();
    println!("{:?} Acquired mutex `second`: {:?}", id, second_guard);
    if !is_swapped && first_guard.balance < amount {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Account {} has insufficient balance", first_guard.id),
        ));
    } else if is_swapped && second_guard.balance < amount {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Account {} has insufficient balance", second_guard.id),
        ));
    }
    if !is_swapped {
        second_guard.balance += amount;
        first_guard.balance -= amount;
    } else {
        first_guard.balance += amount;
        second_guard.balance -= amount;
    }
    Ok(())
}

fn main() {
    let m1: Arc<Mutex<BankAccount>> = Arc::new(Mutex::new(BankAccount::new("A", 1000)));
    let m2: Arc<Mutex<BankAccount>> = Arc::new(Mutex::new(BankAccount::new("B", 2000)));
    let m1_cloned = m1.clone();
    let m2_cloned = m2.clone();
    let t1 = thread_builder("T1")
        .spawn(move || {
            let tid = thread::current();
            println!("{:?} Started", tid);
            transfer(100, &m1_cloned, &m2_cloned)
        })
        .unwrap();
    println!("Started thread T1");

    let m1_cloned = m1.clone();
    let m2_cloned = m2.clone();
    let t2 = thread_builder("T2")
        .spawn(move || {
            let tid = thread::current();
            println!("{:?} Started", tid);
            transfer(300, &m2_cloned, &m1_cloned)
        })
        .unwrap();
    println!("Started thread T2");

    t1.join()
        .unwrap()
        .expect("Failed to wait the thread to finish");
    t2.join()
        .unwrap()
        .expect("Failed to wait the thread to finish");

    println!("Result:");
    println!("m1: {:?}", m1);
    println!("m2: {:?}", m2);
}
