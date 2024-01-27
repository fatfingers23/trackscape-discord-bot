use celery::prelude::*;

#[celery::task]
pub fn run() -> TaskResult<i32> {
    println!("Hello from the add job!");
    Ok(4)
}
