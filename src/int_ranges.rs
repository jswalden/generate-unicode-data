use quote::quote;
use std::collections::btree_set;
use unicode_info::types::CodePointSet;

/// A non-empty integer range from start to end, inclusive.  May consist of only
/// a single code point.
pub struct IntRange(pub u32, pub u32);

impl quote::ToTokens for IntRange {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let IntRange(start, end) = self;
        let code = quote! {
            if #start <= code && code <= #end {
                return true;
            }
        };

        tokens.extend(code);
    }
}

/// An iterator over non-overlapping integer ranges, exposed in sorted order.
pub struct IntRanges<'a> {
    resume: Option<u32>,
    nested_iter: btree_set::Iter<'a, u32>,
}

impl<'a> IntRanges<'a> {
    fn new(code_points: &CodePointSet) -> IntRanges {
        IntRanges {
            resume: None,
            nested_iter: code_points.iter(),
        }
    }
}

impl<'a> Iterator for IntRanges<'a> {
    type Item = IntRange;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = match self.resume {
                Some(n) => {
                    self.resume = None;
                    n
                }
                None => *(self.nested_iter.next()?),
            };

            let mut curr = start;
            loop {
                let next = match self.nested_iter.next() {
                    Some(n) => *n,
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

/// Given a set of code points, return an `IntRanges` that sequentially exposes
/// all code points in the set as a series of `IntRange` ranges.
pub fn int_ranges(code_points: &CodePointSet) -> IntRanges {
    IntRanges::new(code_points)
}
