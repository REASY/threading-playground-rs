mod util;

use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use crate::util::{thread_builder, BankAccount};

use lock_order::{impl_lock_after, lock::LockFor, relation::LockAfter, Locked, Unlocked};

#[derive(Debug)]
struct HoldsLocks {
    from: Mutex<BankAccount>,
    to: Mutex<BankAccount>,
}
// Define enum type per mutex we want to hold.
// This is pure for type system, no runtime cost!
enum LockFrom {}
enum LockTo {}

// Connect how LockFrom interacts with `HoldsLocks.from` mutex
impl LockFor<LockFrom> for HoldsLocks {
    type Data = BankAccount;
    type Guard<'l> = MutexGuard<'l, BankAccount> where Self: 'l;
    fn lock(&self) -> Self::Guard<'_> {
        self.from.lock().unwrap()
    }
}

// Connect how LockTo interacts with `HoldsLocks.to` mutex
impl LockFor<LockTo> for HoldsLocks {
    type Data = BankAccount;
    type Guard<'l> = MutexGuard<'l, BankAccount> where Self: 'l;
    fn lock(&self) -> Self::Guard<'_> {
        self.to.lock().unwrap()
    }
}

// LockFrom is the top of the lock hierarchy.
impl LockAfter<Unlocked> for LockFrom {}
// LockFrom can be acquired before LockTo.
impl_lock_after!(LockFrom => LockTo);

fn transfer(
    amount: u128,
    from: &mut BankAccount,
    to: &mut BankAccount,
) -> Result<(), std::io::Error> {
    if from.balance < amount {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Account {} has insufficient balance", from.id),
        ));
    }
    to.balance += amount;
    from.balance -= amount;
    Ok(())
}

fn main() {
    let state = Arc::new(HoldsLocks {
        from: Mutex::new(BankAccount::new("A", 1000)),
        to: Mutex::new(BankAccount::new("B", 2000)),
    });
    let state_copy = state.clone();
    let t1 = thread_builder("T1")
        .spawn(move || {
            let tid = thread::current();
            let mut locked: Locked<&HoldsLocks, Unlocked> = Locked::new(state_copy.as_ref());
            println!("{:?} Started", tid);
            println!("{:?} Acquiring mutex for LockFrom", tid);
            let (mut a, mut locked_a): (MutexGuard<BankAccount>, Locked<&HoldsLocks, LockFrom>) =
                locked.lock_and::<LockFrom>();
            println!("{:?} Acquired mutex for LockFrom: {:?}", tid, a);

            println!("{:?} Acquiring mutex for LockTo", tid);
            let mut b: MutexGuard<BankAccount> = locked_a.lock::<LockTo>();
            println!("{:?} Acquired mutex for LockTo: {:?}", tid, b);

            transfer(100, &mut a, &mut b)
        })
        .unwrap();
    println!("Started thread T1");

    let state_copy = state.clone();
    let t2 = thread_builder("T2")
        .spawn(move || {
            let tid = thread::current();
            let mut locked = Locked::new(state_copy.as_ref());
            println!("{:?} Started", tid);
            let (mut a, mut locked_a) = locked.lock_and::<LockFrom>();
            let mut b = locked_a.lock::<LockTo>();
            transfer(300, &mut b, &mut a)
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
    println!("from: {:?}", state.from);
    println!("to: {:?}", state.to);
}
