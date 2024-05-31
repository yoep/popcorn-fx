/// A C-compatible enum representing either a successful result or an error.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum ResultC<T, E> {
    /// Represents a successful result containing a value of type `T`.
    Ok(T),
    /// Represents an error containing a value of type `E`.
    Err(E),
}

impl<T, E> From<Result<T, E>> for ResultC<T, E> {
    /// Converts a Rust `Result` into a `ResultC`.
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(e) => ResultC::Ok(e),
            Err(e) => ResultC::Err(e),
        }
    }
}

impl<T, E> From<ResultC<T, E>> for Result<T, E> {
    /// Converts a `ResultC` into a Rust `Result`.
    fn from(value: ResultC<T, E>) -> Self {
        match value {
            ResultC::Ok(e) => Ok(e),
            ResultC::Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use popcorn_fx_core::core::torrents::TorrentError;
    use popcorn_fx_core::testing::init_logger;

    #[test]
    fn test_result_c_from() {
        init_logger();
        let result = Ok(1);

        let result_c: ResultC<i32, TorrentError> = ResultC::from(result);

        assert_eq!(result_c, ResultC::Ok(1));
    }

    #[test]
    fn test_result_from() {
        init_logger();
        let result_c = ResultC::Ok(1);

        let result: Result<i32, TorrentError> = Result::from(result_c);

        assert_eq!(result, Ok(1));
    }
}
