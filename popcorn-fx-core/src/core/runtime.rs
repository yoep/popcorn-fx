use std::future::Future;

/// Run the given future on the current thread or spawn a new runtime if needed.
pub fn block_in_place<F: Future>(closure: F) -> F::Output {
    tokio::task::block_in_place(|| {
        let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");
        runtime.block_on(closure)
    })
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_block_in_place_no_runtime() {
        let result = block_in_place(async {
            thread::sleep(Duration::from_millis(10));
            "lorem"
        });

        assert_eq!("lorem", result);
    }

    #[test]
    fn test_block_in_place_on_runtime() {
        let (tx, rx) = channel();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.spawn(async move {
            let result = block_in_place(async {
                thread::sleep(Duration::from_millis(10));
                "ipsum"
            });

            tx.send(result).unwrap();
        });

        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!("ipsum", result);
    }
}