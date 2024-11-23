use std::thread;

#[derive(Debug)]
pub struct BankAccount {
    pub id: String,
    pub balance: u128,
}

impl BankAccount {
    pub fn new(id: &str, initial_balance: u128) -> BankAccount {
        BankAccount {
            id: id.to_owned(),
            balance: initial_balance,
        }
    }
}

pub fn thread_builder(name: &str) -> thread::Builder {
    thread::Builder::new().name(name.to_owned())
}
