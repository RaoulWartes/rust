use super::*;

// test trait_item
// trait T { fn new() -> Self; }
pub(super) fn trait_(p: &mut Parser, m: Marker) {
    p.bump(T![trait]);
    name_r(p, ITEM_RECOVERY_SET);

    // test trait_item_generic_params
    // trait X<U: Debug + Display> {}
    type_params::opt_generic_param_list(p);

    if p.eat(T![=]) {
        // test trait_alias
        // trait Z<U> = T<U>;
        type_params::bounds_without_colon(p);

        // test trait_alias_where_clause
        // trait Z<U> = T<U> where U: Copy;
        // trait Z<U> = where Self: T<U>;
        type_params::opt_where_clause(p);
        p.expect(T![;]);
        m.complete(p, TRAIT);
        return;
    }

    if p.at(T![:]) {
        // test trait_item_bounds
        // trait T: Hash + Clone {}
        type_params::bounds(p);
    }

    // test trait_item_where_clause
    // trait T where Self: Copy {}
    type_params::opt_where_clause(p);

    if p.at(T!['{']) {
        assoc_item_list(p);
    } else {
        p.error("expected `{`");
    }
    m.complete(p, TRAIT);
}

// test impl_def
// impl Foo {}
pub(super) fn impl_(p: &mut Parser, m: Marker) {
    assert!(p.at(T![impl]));
    p.bump(T![impl]);
    if choose_type_params_over_qpath(p) {
        type_params::opt_generic_param_list(p);
    }

    // test impl_def_const
    // impl const Send for X {}
    p.eat(T![const]);

    // FIXME: never type
    // impl ! {}

    // test impl_def_neg
    // impl !Send for X {}
    p.eat(T![!]);
    impl_type(p);
    if p.eat(T![for]) {
        impl_type(p);
    }
    type_params::opt_where_clause(p);
    if p.at(T!['{']) {
        assoc_item_list(p);
    } else {
        p.error("expected `{`");
    }
    m.complete(p, IMPL);
}

// test impl_item_list
// impl F {
//     type A = i32;
//     const B: i32 = 92;
//     fn foo() {}
//     fn bar(&self) {}
// }
pub(crate) fn assoc_item_list(p: &mut Parser) {
    assert!(p.at(T!['{']));
    let m = p.start();
    p.bump(T!['{']);
    // test impl_inner_attributes
    // enum F{}
    // impl F {
    //      //! This is a doc comment
    //      #![doc("This is also a doc comment")]
    // }
    attributes::inner_attrs(p);

    while !p.at(EOF) && !p.at(T!['}']) {
        if p.at(T!['{']) {
            error_block(p, "expected an item");
            continue;
        }
        item_or_macro(p, true);
    }
    p.expect(T!['}']);
    m.complete(p, ASSOC_ITEM_LIST);
}

// test impl_type_params
// impl<const N: u32> Bar<N> {}
fn choose_type_params_over_qpath(p: &Parser) -> bool {
    // There's an ambiguity between generic parameters and qualified paths in impls.
    // If we see `<` it may start both, so we have to inspect some following tokens.
    // The following combinations can only start generics,
    // but not qualified paths (with one exception):
    //     `<` `>` - empty generic parameters
    //     `<` `#` - generic parameters with attributes
    //     `<` `const` - const generic parameters
    //     `<` (LIFETIME_IDENT|IDENT) `>` - single generic parameter
    //     `<` (LIFETIME_IDENT|IDENT) `,` - first generic parameter in a list
    //     `<` (LIFETIME_IDENT|IDENT) `:` - generic parameter with bounds
    //     `<` (LIFETIME_IDENT|IDENT) `=` - generic parameter with a default
    // The only truly ambiguous case is
    //     `<` IDENT `>` `::` IDENT ...
    // we disambiguate it in favor of generics (`impl<T> ::absolute::Path<T> { ... }`)
    // because this is what almost always expected in practice, qualified paths in impls
    // (`impl <Type>::AssocTy { ... }`) aren't even allowed by type checker at the moment.
    if !p.at(T![<]) {
        return false;
    }
    if p.nth(1) == T![#] || p.nth(1) == T![>] || p.nth(1) == T![const] {
        return true;
    }
    (p.nth(1) == LIFETIME_IDENT || p.nth(1) == IDENT)
        && (p.nth(2) == T![>] || p.nth(2) == T![,] || p.nth(2) == T![:] || p.nth(2) == T![=])
}

// test_err impl_type
// impl Type {}
// impl Trait1 for T {}
// impl impl NotType {}
// impl Trait2 for impl NotType {}
pub(crate) fn impl_type(p: &mut Parser) {
    if p.at(T![impl]) {
        p.error("expected trait or type");
        return;
    }
    types::type_(p);
}
