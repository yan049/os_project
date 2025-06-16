#![no_std]
#![no_main]

#[macro_use]
extern crate user;

use user::{fork, getpid, wait};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    
    let mut x: i32 = 42;
    println!("x is {}", x);
    println!("pid {}: parent start forking ...", getpid());
    let pid = fork();
    if pid == 0 {
        //  child process
        println!("pid {}: forked child see x {}",getpid() , x);
        x = 100;
        println!("pid {}: child change x to {}",getpid(), x);
	100
    } else {
        // parent process
	let mut exit_code: i32 = 0;
        println!("pid {}: ready waiting child ...", getpid());
	assert_eq!(pid, wait(&mut exit_code));
        println!("pid {}: parent see x {}", getpid(),x);
        x = 200;
        println!("pid {}: parent changes x to {}", getpid(), x);
	0
    }
}
