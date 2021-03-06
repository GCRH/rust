// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// To work around #46855
// compile-flags: -Z mir-opt-level=0
// Regression test for the inhabitedness of unions with uninhabited variants, issue #46845

use std::mem;

#[derive(Copy, Clone)]
enum Never { }

// A single uninhabited variant shouldn't make the whole union uninhabited.
union Foo {
    a: u64,
    _b: Never
}

// If all the variants are uninhabited, however, the union should be uninhabited.
union Bar {
    _a: (Never, u64),
    _b: (u64, Never)
}

fn main() {
    assert_eq!(mem::size_of::<Foo>(), 8);
    assert_eq!(mem::size_of::<Bar>(), 0);

    let f = [Foo { a: 42 }, Foo { a: 10 }];
    println!("{}", unsafe { f[0].a });
    assert_eq!(unsafe { f[1].a }, 10);
}
