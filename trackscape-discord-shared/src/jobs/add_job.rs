use crate::database::drop_logs_db::DropLogs;
use celery::prelude::*;
use celery::task::Signature;

#[celery::task]
pub fn run() -> TaskResult<i32> {
    println!("Hello from the add job!");
    Ok(4)
}
