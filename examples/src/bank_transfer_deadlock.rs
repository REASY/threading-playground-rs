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
    println!("{:?} Acquiring mutex `from`: {:?}", id, from);
    let mut from_guard = from.lock().unwrap();
    println!("{:?} Acquired mutex `from`: {:?}", id, from_guard);

    thread::sleep(Duration::from_millis(100));

    println!("{:?} Acquiring mutex `to`: {:?}", id, to);
    let mut to_guard = to.lock().unwrap();
    println!("{:?} Acquired mutex `to`: {:?}", id, to_guard);
    if from_guard.balance < amount {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Account {} has insufficient balance", from_guard.id),
        ));
    }
    to_guard.balance += amount;
    from_guard.balance -= amount;
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
