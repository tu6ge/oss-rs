use std::task::{Context, Poll};

trait Stream {
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<()>;
}

struct Struct;

impl Stream for Struct {
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        println!("aaa");
        Poll::Pending
    }
}

#[tokio::main]
async fn main() {}
