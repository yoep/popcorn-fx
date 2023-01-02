use std::fmt::Display;

/// An observer invocation which can be triggered for each registered [Observer].
pub type ObserverInvocation<T> = fn(&mut T);

/// The observable can be watched for state changes.
pub trait Observable<'a, T: Observer> {
    /// Register a new [Observer] to the [Observable].
    fn register(&mut self, observer: &'a mut T);

    /// Remove an existing [Observer] from the [Observable].
    fn unregister(&mut self, observer: &'a T);
}

/// The observer which watches an [Observable] for state changes.
/// Each [Observer] must include the [PartialEq] for it to be able to unregister itself from the
/// [Observable].
pub trait Observer: PartialEq + Display + Send + Sync {}

/// The basic observable can be used as a wrapper within another struct for providing the core
/// functionality of an [Observable].
pub struct BasicObservable<'a, T:Observer>{
    /// The registered observers.
    observers: Vec<&'a mut T>,
}

impl<'a, T: Observer> BasicObservable<'a, T> {
    /// Create a new [BasicObservable] instance.
    pub fn new() -> Self {
        Self {
            observers: vec!()
        }
    }

    /// Invoke each registered observer.
    pub fn invoke_observers(self, invoke: ObserverInvocation<T>) {
        for observer in self.observers {
            (invoke)(observer);
        }
    }
}

impl<'a, T: Observer> Observable<'a, T> for BasicObservable<'a, T> {
    fn register(&mut self, observer: &'a mut T) {
        self.observers.push(observer);
    }

    fn unregister(&mut self, observer: &'a T) {
        if let Some(index) = self.observers.iter().position(|x| *x == observer) {
            self.observers.remove(index);
        }
    }
}

#[cfg(test)]
mod test {
    use std::fmt::{Display, Formatter};

    use crate::observer::{BasicObservable, Observable, Observer};

    pub trait TestObservable: Observer {
        fn invoked();
    }

    #[derive(PartialEq)]
    struct TestObservableImpl {
        invoked: bool,
    }

    impl TestObservableImpl {
        fn new() -> Self {
            Self {
                invoked: false,
            }
        }
    }

    impl Observer for TestObservableImpl {}

    impl Display for TestObservableImpl {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestObservable")
        }
    }

    impl TestObservable for TestObservableImpl {
        fn invoked() {}
    }

    #[test]
    fn test_register_should_add_the_observer() {
        let mut observer = TestObservableImpl::new();
        let mut observable: BasicObservable<TestObservableImpl> = BasicObservable::new();

        observable.register(&mut observer);
        observable.invoke_observers(|mut x| {
            x.invoked = true;
        });

        assert_eq!(true, observer.invoked)
    }
}