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
    for (_, code_points) in Itertools::group_by(unconditional_code_points.into_iter(), |code| {
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
            for (_, inner_code_points) in Itertools::group_by(matches.into_iter(), |code| {
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
        /// Given a code point, return `true` iff its uppercased form consists
        /// of multiple code points.
        ///
        /// Most uppercased code points consist only of a single code point:
        /// 'a' -> 'A', ':' -> ':' (i.e. no transformation), etc.  A relative
        /// few expand to more than one code point.  Perhaps most notoriously in
        /// the Western world U+00DF LATIN SMALL LETTER SHARP S, "ß", uppercases
        /// to "SS".  This function returns true for such code points:
        ///
        /// ```
        /// assert!(!changes_when_upper_cased_special_casing('a' as u16));
        /// assert!(!changes_when_upper_cased_special_casing(':' as u16));
        ///
        /// assert!(changes_when_upper_cased_special_casing('ß' as u16));
        /// ```
        #[no_mangle]
        pub extern "C" fn changes_when_upper_cased_special_casing(code: u16) -> bool {
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

fn generate_length_upper_case_special_casing_fun(
    unconditional_toupper: &special_casing::UnconditionalMapping,
) -> proc_macro2::TokenStream {
    // We could, C++-style, generate a zillion `code => len,` cases.  But we
    // have very few distinct `len`, and Rust provides more concise, readable
    // `code1 | code2 | ... => len` syntax.  Reorder the mappings to group all
    // replacements of identical length together, group by replacement length,
    // then generate one match-arm per replacement length.
    let mut unconditional_toupper: Vec<(&u32, &Vec<u32>)> =
        unconditional_toupper.into_iter().collect();
    unconditional_toupper
        .sort_by(|left, right| (left.1.len(), left.0).cmp(&(right.1.len(), right.0)));

    let cases: Vec<proc_macro2::TokenStream> = unconditional_toupper
        .into_iter()
        .group_by(|(_code, replacements)| replacements.len())
        .into_iter()
        .map(|(replacement_len, codes)| {
            let codes: Vec<_> = codes
                .into_iter()
                .map(|(code, _)| {
                    let code = *code as u16;
                    quote! {
                        #code
                    }
                })
                .collect();
            quote! {
                #( #codes )|* => #replacement_len,
            }
        })
        .collect();

    quote! {
        /// Given a code point for which
        /// `changes_when_upper_cased_special_casing` returns true,
        /// return the number of code points that constitute its uppercased
        /// form.
        ///
        /// ```
        /// assert!(length_upper_case_special_casing('ß' as u16), 2); // SS
        /// ```
        ///
        /// Behavior is undefined if this function is called with a code point
        /// that doesn't pass this gauntlet, ergo does not have special
        /// uppercasing behavior.
        #[no_mangle]
        pub extern "C" fn length_upper_case_special_casing(code: u16) -> usize {
            match code {
                #( #cases )*
                _ => panic!("bad input"),
            }
        }
    }
}

fn generate_append_upper_case_special_casing_fun(
    unconditional_toupper: &special_casing::UnconditionalMapping,
) -> proc_macro2::TokenStream {
    let cases: Vec<proc_macro2::TokenStream> = unconditional_toupper
        .into_iter()
        .map(|(code, replacements)| {
            let code = *code as u16;
            let replacements_len = replacements.len();
            let replacements = replacements
                .into_iter()
                .map(|code| *code as u16)
                .collect::<Vec<u16>>();

            quote! {
                #code => {
                    ptr.copy_from_nonoverlapping(
                        [ #( #replacements ),* ].as_ptr(),
                        #replacements_len
                    );
                    index.write(#replacements_len as usize + index.read())
                },
            }
        })
        .collect();

    quote! {
        /// Given a code point for which
        /// `changes_when_upper_cased_special_casing` returns true,  write the
        /// code points that constitute its uppercased form to
        /// `elements[*index]`, incrementing `*index` by the number of code
        /// points written.
        ///
        /// It is presumed that properly-owned memory exists at these addresses
        /// -- typically by calling `length_upper_case_special_casing` and using
        /// the value it returns to provide such memory.
        ///
        /// Behavior is undefined if this function is called with a code point
        /// for which `changes_when_upper_cased_special_casing` returns false,
        /// that does not have special uppercasing behavior.
        #[no_mangle]
        pub unsafe extern "C" fn append_upper_case_special_casing(code: u16, elements: *mut u16, index: *mut usize) {
            let ptr = elements.add(index.read());
            match code {
                #( #cases )*
                _ => panic!("bad input"),
            }
        }
    }
}

pub fn generate_special_casing_functions(
    scd: &special_casing::SpecialCasingData,
) -> proc_macro2::TokenStream {
    let changes_when_upper_cased_special_casing_fun =
        generate_changes_when_upper_cased_special_casing_fun(&scd.unconditional_toupper);

    let length_upper_case_special_casing_fun =
        generate_length_upper_case_special_casing_fun(&scd.unconditional_toupper);

    let append_upper_case_special_casing_fun =
        generate_append_upper_case_special_casing_fun(&scd.unconditional_toupper);

    quote! {
        #changes_when_upper_cased_special_casing_fun

        #length_upper_case_special_casing_fun

        #append_upper_case_special_casing_fun
    }
}
