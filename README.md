# blanket_trait

Attribute macro that generates a trait with an inherent blanket implementation.

## Example

```rust
trait Behavior {
    fn name() -> &'static str;
    async fn action(&self);
}

#[blanket_trait(impl<T: Behavior> ErasedBehavior for T where T: Send + Sync + Clone + 'static)]
pub trait ErasedBehavior {
    fn name(&self) -> &str {
        T::name()
    }

    fn action(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>{
        Box::pin(T::action(self))
    }

    fn dyn_clone(&self) -> Box<dyn ErasedBehavior> {
        Box::new(T::clone(self))
    }
}
```

Generates:

```rust
pub trait ErasedBehavior {
    fn name(&self) -> &str;
    fn action(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn dyn_clone(&self) -> Box<dyn ErasedBehavior>;
}

impl<T: Behavior> ErasedBehavior for T where T: Send + Sync + Clone + 'static {
    fn name(&self) -> &str {
        T::name()
    }

    fn action(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>{
        Box::pin(T::action(self))
    }

    fn dyn_clone(&self) -> Box<dyn ErasedBehavior> {
        Box::new(self.clone())
    }
}
```

## License

License under either of

Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)
at your option.

## Contribution

Contributions are welcome!

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
