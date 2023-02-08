use crate::core::CoreCallback;

/// The media callback type which handles callbacks for media changes.
/// This is a generic type that can be reused within the [crate::core::media] package.
pub type MediaCallback<E> = CoreCallback<E>;
