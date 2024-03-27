#![no_std]
mod sync;
mod utils;

use sync::spinlock::SpinLock;

fn main() {
    let lock = SpinLock::new(0);
}
