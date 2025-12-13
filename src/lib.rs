#![doc = include_str!("../README.md")]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Generics, ImplItem, ImplItemConst, ImplItemFn, ImplItemType, ItemImpl, ItemTrait,
    Path, Token, Type, Visibility, WhereClause, parse::Parse, parse_macro_input, spanned::Spanned,
};

struct ItemImplHeader {
    pub attrs: Vec<Attribute>,
    pub unsafety: Option<Token![unsafe]>,
    pub impl_token: Token![impl],
    pub generics: Generics,
    pub path: Path,
    pub for_: Token![for],
    pub self_ty: Box<Type>,
}

impl Parse for ItemImplHeader {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut header = ItemImplHeader {
            attrs: input.call(Attribute::parse_outer)?,
            unsafety: input.parse()?,
            impl_token: input.parse()?,
            // This never parses the where clause
            generics: input.parse()?,
            path: input.parse()?,
            for_: input.parse()?,
            self_ty: input.parse()?,
        };
        let where_clause: Option<WhereClause> = input.parse()?;
        header.generics.where_clause = where_clause;
        Ok(header)
    }
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
pub fn blanket_trait(first: TokenStream, tokens: TokenStream) -> TokenStream {
    let header = parse_macro_input!(first as ItemImplHeader);
    let mut trait_block = parse_macro_input!(tokens as ItemTrait);

    let mut items = Vec::new();

    for item in &mut trait_block.items {
        match item {
            syn::TraitItem::Macro(_) => (),
            syn::TraitItem::Verbatim(_) => (),
            syn::TraitItem::Fn(f) => {
                let Some(block) = f.default.take() else {
                    return syn::Error::new(f.span(), "Expected function body")
                        .into_compile_error()
                        .into();
                };
                items.push(ImplItem::Fn(ImplItemFn {
                    attrs: f.attrs.clone(),
                    vis: Visibility::Inherited,
                    defaultness: None,
                    sig: f.sig.clone(),
                    block,
                }));
            }
            syn::TraitItem::Type(t) => {
                let Some((eq_token, ty)) = t.default.take() else {
                    return syn::Error::new(t.span(), "Expected default value.")
                        .into_compile_error()
                        .into();
                };
                items.push(ImplItem::Type(ImplItemType {
                    attrs: t.attrs.clone(),
                    vis: Visibility::Inherited,
                    defaultness: None,
                    type_token: t.type_token,
                    ident: t.ident.clone(),
                    generics: t.generics.clone(),
                    eq_token,
                    ty,
                    semi_token: t.semi_token,
                }));
            }
            syn::TraitItem::Const(c) => {
                let Some((eq_token, expr)) = c.default.take() else {
                    return syn::Error::new(c.span(), "Expected default value.")
                        .into_compile_error()
                        .into();
                };
                items.push(ImplItem::Const(ImplItemConst {
                    attrs: c.attrs.clone(),
                    vis: Visibility::Inherited,
                    defaultness: None,
                    const_token: c.const_token,
                    ident: c.ident.clone(),
                    generics: c.generics.clone(),
                    colon_token: c.colon_token,
                    ty: c.ty.clone(),
                    eq_token,
                    expr,
                    semi_token: c.semi_token,
                }));
            }
            _ => (),
        }
    }

    let out_impl = ItemImpl {
        attrs: header.attrs,
        defaultness: None,
        unsafety: header.unsafety,
        impl_token: header.impl_token,
        generics: header.generics,
        trait_: Some((None, header.path, header.for_)),
        self_ty: header.self_ty,
        brace_token: trait_block.brace_token,
        items,
    };

    quote! {
        #trait_block

        #out_impl
    }
    .into()
}
