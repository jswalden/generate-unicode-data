use crate::int_ranges;
use itertools::Itertools;
use quote::quote;
use std::iter::IntoIterator;
use unicode_info::{special_casing, types::CodePointSet};

fn in_any_range(ranges: Vec<int_ranges::IntRange>) -> Vec<proc_macro2::TokenStream> {
    assert!(ranges.len() > 1);

    let range_tests = ranges.into_iter().map(|int_ranges::IntRange(start, end)| {
        if start == end {
            quote! {
                code == #start
            }
        } else {
            quote! {
                #start <= code && code <= #end
            }
        }
    });

    // Iterator::intersperse_with is nightly-experimental.  Use fully-qualified
    // syntax to not conflict with a potential standard library addition.
    Itertools::intersperse_with(range_tests, || quote! { || }).collect()
}

fn generate_accept_range(code_points: &CodePointSet) -> proc_macro2::TokenStream {
    let child_ranges: Vec<_> = int_ranges::int_ranges(code_points).collect();

    // In principle we could pass this function the known range of `code` at the
    // location this code is generated, and we could eliminate some redundant
    // checks.
    //
    // But why bother?  This straight-line code is tailor-made for compiler
    // range analysis.  Let it do the heavy lifting.

    // If `code_points` is a contiguous list of code points, emit the simplest
    // possible range check.
    if child_ranges.len() == 1 {
        let int_ranges::IntRange(lower, upper) = child_ranges[0];
        return quote! {
            if (code <= #upper) {
                return #lower <= code;
            }
        };
    }

    // Otherwise exclude everything below the minimum, then admit everything
    // that passes that's below the maximum.
    let min_in_range = child_ranges[0].0;
    let max_in_range = child_ranges[child_ranges.len() - 1].1;

    let in_any_range = in_any_range(child_ranges);

    quote! {
        if code < #min_in_range {
            return false;
        }

        if (code <= #max_in_range) {
            return #( #in_any_range )*;
        }
    }
}

fn last_in_set(set: &CodePointSet) -> u32 {
    assert!(!set.is_empty());
    *set.iter().rev().next().expect("non-empty set")
}

fn generate_changes_when_upper_cased_special_casing_fun(
    unconditional_toupper: &special_casing::UnconditionalMapping,
) -> proc_macro2::TokenStream {
    assert!(
        !unconditional_toupper.is_empty(),
        "map shouldn't be empty, else why are we here?"
    );

    let unconditional_code_points = unconditional_toupper
        .keys()
        .map(|code| *code)
        .collect::<Vec<u32>>();

    let lowest = unconditional_code_points[0];
    let highest = unconditional_code_points[unconditional_code_points.len() - 1];

    // Partition the 64K set of BMP code points into 16 4K buckets, then check
    // for matches in each bucket.
    let mut range_tests = vec![];
    for (_, code_points) in unconditional_code_points
        .into_iter()
        .group_by(|code| {
            const CHUNK_SIZE: u32 = 0x1000;
            let code = *code;
            let start = code - (code % CHUNK_SIZE);
            (start, start + CHUNK_SIZE)
        })
        .into_iter()
    {
        let matches: CodePointSet = code_points.into_iter().collect();
        if matches.is_empty() {
            continue;
        }

        let code = if matches.len() <= 8 {
            // If `matches` contains only a very few code points, just directly
            // test for them.
            generate_accept_range(&matches)
        } else {
            let last = last_in_set(&matches);

            // Otherwise split into further sub-ranges of smaller size.`
            let mut inner_tests = vec![];
            for (_, inner_code_points) in matches
                .into_iter()
                .group_by(|code| {
                    const INNER_CHUNK_SIZE: u32 = 0x100;
                    let code = *code;
                    let start = code - (code % INNER_CHUNK_SIZE);
                    (start, start + INNER_CHUNK_SIZE)
                })
                .into_iter()
            {
                let inner_matches: CodePointSet = inner_code_points.into_iter().collect();
                if inner_matches.is_empty() {
                    continue;
                }

                inner_tests.push(generate_accept_range(&inner_matches));
            }

            quote! {
                if code <= #last {
                    #( #inner_tests )*
                }
            }
        };

        range_tests.push(code);
    }

    quote! {
        fn changes_when_upper_cased_special_casing(code: u16) -> bool {
            let code = code as u32;

            // Exclude all code points outside the smallest range encompassing
            // all special casing code points.  (Subsequent code depends upon
            // this to perform comparisons increasingly dependent on prior
            // comparisons having occurred.)
            if code < #lowest || code > #highest {
                return false;
            }

            #( #range_tests )*

            false
        }
    }
}

fn generate_length_upper_cased_special_casing_fun() -> proc_macro2::TokenStream {
    quote!()
}

fn generate_append_upper_cased_special_casing_fun() -> proc_macro2::TokenStream {
    quote!()
}

pub fn generate_special_casing_functions(
    scd: &special_casing::SpecialCasingData,
) -> proc_macro2::TokenStream {
    let changes_when_upper_cased_special_casing_fun =
        generate_changes_when_upper_cased_special_casing_fun(&scd.unconditional_toupper);

    let length_upper_cased_special_casing_fun = generate_length_upper_cased_special_casing_fun();

    let append_upper_cased_special_casing_fun = generate_append_upper_cased_special_casing_fun();

    quote! {
        #changes_when_upper_cased_special_casing_fun

        #length_upper_cased_special_casing_fun

        #append_upper_cased_special_casing_fun
    }
}
