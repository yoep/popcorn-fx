use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Blocks the current thread to execute the provided future synchronously.
///
/// This function blocks the current thread to execute the provided future synchronously,
/// ensuring that any asynchronous tasks within the future are executed within a Tokio runtime.
///
/// # Arguments
///
/// * `closure` - The future to be executed synchronously.
#[deprecated(note = "Use block_in_place_runtime instead")]
pub fn block_in_place<F: Future>(closure: F) -> F::Output {
    tokio::task::block_in_place(|| {
        let runtime = Runtime::new().expect("expected a runtime to have been created");
        runtime.block_on(closure)
    })
}

/// Blocks the current thread to execute the provided future synchronously.
///
/// This function blocks the current thread to execute the provided future synchronously,
/// ensuring that any asynchronous tasks within the future are executed within the provided runtime.
pub fn block_in_place_runtime<F: Future>(closure: F, runtime: &Arc<Runtime>) -> F::Output {
    tokio::task::block_in_place(|| runtime.block_on(closure))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use tokio::time;

    #[test]
    fn test_block_in_place_no_runtime() {
        let result = block_in_place(async {
            time::sleep(Duration::from_millis(10)).await;
            "lorem"
        });

        assert_eq!("lorem", result);
    }

    #[test]
    fn test_block_in_place_on_runtime() {
        let (tx, rx) = channel();
        let runtime = Runtime::new().unwrap();

        runtime.spawn(async move {
            let result = block_in_place(async {
                time::sleep(Duration::from_millis(10)).await;
                "ipsum"
            });

            tx.send(result).unwrap();
        });

        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!("ipsum", result);
    }

    #[test]
    fn test_block_in_place_runtime_not_nested() {
        let expected = "FooBar";
        let runtime = Arc::new(Runtime::new().unwrap());

        let result = block_in_place_runtime(
            async {
                time::sleep(Duration::from_millis(10)).await;
                expected
            },
            &runtime,
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn test_block_in_place_runtime_nested() {
        let expected = "LoremIpsumDolor";
        let nested = Arc::new(Runtime::new().unwrap());
        let runtime = Arc::new(Runtime::new().unwrap());
        let (tx, rx) = channel();

        let runtime_inner = runtime.clone();
        nested.spawn(async move {
            let result = block_in_place_runtime(
                async {
                    time::sleep(Duration::from_millis(10)).await;
                    expected
                },
                &runtime_inner,
            );
            tx.send(result).unwrap();
        });

        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(expected, result);
    }
}
