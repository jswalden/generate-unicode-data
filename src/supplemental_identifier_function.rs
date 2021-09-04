use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::btree_set;
use std::collections::BTreeSet;
use unicode_info::types::CodePointSet;

struct IntRange(u32, u32);

impl quote::ToTokens for IntRange {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let start = self.0;
        let end = self.1;

        let code = quote! {
            if #start <= code && code <= #end {
                return true;
            }
        };

        tokens.extend(code);
    }
}

struct IntRanges {
    resume: Option<u32>,
    nested_iter: btree_set::IntoIter<u32>,
}

impl IntRanges {
    fn new(code_points: BTreeSet<u32>) -> IntRanges {
        IntRanges {
            resume: None,
            nested_iter: code_points.into_iter(),
        }
    }
}

impl Iterator for IntRanges {
    type Item = IntRange;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = match self.resume {
                Some(n) => {
                    self.resume = None;
                    n
                }
                None => self.nested_iter.next()?,
            };

            let mut curr = start;
            loop {
                let next = match self.nested_iter.next() {
                    Some(n) => n,
                    None => {
                        return Some(IntRange(start, curr));
                    }
                };

                if curr + 1 != next {
                    self.resume = Some(next);
                    return Some(IntRange(start, curr));
                }

                curr = next;
            }
        }
    }
}

fn int_ranges(code_points: &CodePointSet) -> IntRanges {
    let mut sorted = BTreeSet::<u32>::new();
    sorted.extend(code_points.iter());
    IntRanges::new(sorted)
}

pub fn generate_supplemental_identifer_function(
    name: &str,
    set: &CodePointSet,
) -> proc_macro2::TokenStream {
    let name = Ident::new(name, Span::call_site());

    let ranges: Vec<_> = int_ranges(set).collect();

    quote! {
        pub fn #name(code: u32) -> bool {
            #( #ranges )*
            return false;
        }
    }
}
