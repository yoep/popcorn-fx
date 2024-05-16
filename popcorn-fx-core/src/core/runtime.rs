use std::future::Future;

/// Blocks the current thread to execute the provided future synchronously.
///
/// This function blocks the current thread to execute the provided future synchronously,
/// ensuring that any asynchronous tasks within the future are executed within a Tokio runtime.
///
/// # Arguments
///
/// * `closure` - The future to be executed synchronously.
///
/// # Returns
///
/// The output of the provided future.
pub fn block_in_place<F: Future>(closure: F) -> F::Output {
    tokio::task::block_in_place(|| {
        let runtime =
            tokio::runtime::Runtime::new().expect("expected a runtime to have been created");
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
