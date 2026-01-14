#![doc = include_str!("../README.md")]
use proc_macro::{Delimiter, Group, Punct, Spacing, Span, TokenStream, TokenTree};

trait TokenTreeExt {
    fn is_brace(&self) -> bool;
    fn braced_group(&self) -> Option<&Group>;
    fn is(&self, c: char) -> bool;
    fn is_new_item_trait(&self) -> bool;
}

impl TokenTreeExt for TokenTree {
    fn is_brace(&self) -> bool {
        if let TokenTree::Group(g) = self && g.delimiter() == Delimiter::Brace {
            true
        } else {
            false
        }
    }

    fn braced_group(&self) -> Option<&Group> {
        if let TokenTree::Group(g) = self && g.delimiter() == Delimiter::Brace {
            Some(g)
        } else {
            None
        }
    }

    fn is(&self, c: char) -> bool {
        if let TokenTree::Punct(p) = self && p.as_char() == c {
            true
        } else {
            false
        }
    }

    fn is_new_item_trait(&self) -> bool {
        if let TokenTree::Ident(p) = self  {
            let p = p.to_string();
            p == "fn" || p == "const" || p == "type"
        } else {
            false
        }
    }
}

fn semi(span: Span) -> TokenTree {
    let mut semi = TokenTree::Punct(Punct::new(';', Spacing::Alone));
    semi.set_span(span);
    semi
}

fn braced(tokens: TokenStream, span: Span) -> TokenTree {
    let mut braced = TokenTree::Group(Group::new(Delimiter::Brace, tokens));
    braced.set_span(span);
    braced
}

/// Generate a trait with a blanket implementation.
///
/// # Rules
///
/// * Generated `trait` block will not contain any default implementations.
/// * Errors if any item do not contain a default implementation.
/// * Attributes on fields are copied to both instances.
///
/// # Syntax
///
/// ```
/// # use std::pin::Pin;
/// # use blanket_trait::blanket_trait;
/// trait Behavior {
///     fn name() -> &'static str;
///     async fn action(&self);
/// }
///
/// #[blanket_trait(impl<T: Behavior> ErasedBehavior for T where T: Send + Sync + Clone + 'static)]
/// pub trait ErasedBehavior {
///     fn name(&self) -> &str {
///         T::name()
///     }
///
///     fn action(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>{
///         Box::pin(T::action(self))
///     }
///
///     fn dyn_clone(&self) -> Box<dyn ErasedBehavior> {
///         Box::new(T::clone(self))
///     }
/// }
/// ```
///
/// # Example Output
///
/// ```
/// # use std::pin::Pin;
/// # use blanket_trait::blanket_trait;
/// # trait Behavior {
/// #     fn name() -> &'static str;
/// #     async fn action(&self);
/// # }
/// pub trait ErasedBehavior {
///     fn name(&self) -> &str;
///     fn action(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
///     fn dyn_clone(&self) -> Box<dyn ErasedBehavior>;
/// }
///
/// impl<T: Behavior> ErasedBehavior for T where T: Send + Sync + Clone + 'static {
///     fn name(&self) -> &str {
///         T::name()
///     }
///
///     fn action(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>{
///         Box::pin(T::action(self))
///     }
///
///     fn dyn_clone(&self) -> Box<dyn ErasedBehavior> {
///         Box::new(self.clone())
///     }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn blanket_trait(impl_header: TokenStream, tokens: TokenStream) -> TokenStream {
    let mut impl_block: Vec<_> = impl_header.into_iter().collect();
    let mut trait_block = Vec::new();
    for tt in tokens {
        let span = tt.span();
        if let Some(g) = tt.braced_group() {
            let mut impl_block_inner = Vec::new();
            let mut trait_block_inner = Vec::new();
            let mut is_default_member = false;
            for tt in g.stream() {
                if tt.is_brace() {
                    trait_block_inner.push(semi(tt.span()));
                    impl_block_inner.push(tt);
                } else if tt.is('=') {
                    is_default_member = true;
                    impl_block_inner.push(tt);
                } else if is_default_member && (tt.is(';') || tt.is_new_item_trait()) {
                    is_default_member = false;
                    trait_block_inner.push(tt.clone());
                    impl_block_inner.push(tt);
                } else if is_default_member {
                    impl_block_inner.push(tt);
                } else {
                    trait_block_inner.push(tt.clone());
                    impl_block_inner.push(tt);
                }
            }
            trait_block.push(braced(trait_block_inner.into_iter().collect(), span));
            impl_block.push(braced(impl_block_inner.into_iter().collect(), span));
        } else {
            trait_block.push(tt);
        }
    }
    trait_block.into_iter().chain(impl_block).collect()
}
