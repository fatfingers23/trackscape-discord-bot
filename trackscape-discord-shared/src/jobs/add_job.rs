use anyhow::Result;
use celery::prelude::*;

#[celery::task]
pub fn run() -> TaskResult<i32> {
    println!("Hello from the job!");
    Ok(4)
}
