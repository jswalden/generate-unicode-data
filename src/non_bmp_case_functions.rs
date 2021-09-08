//! Generate case-mapping functions for non-BMP code points.

use itertools::Itertools;
use quote::format_ident;
use quote::quote;
use unicode_info::non_bmp;
use unicode_info::types;

/// A casing operation, either uppercasing or lowercasing.
enum Casing {
  Upper,
  Lower,
}

impl Casing {
  /// Return "upper" or "lower" for this.
  fn to_string(&self) -> &'static str {
    match self {
      Casing::Upper => "upper",
      Casing::Lower => "lower",
    }
  }
}

/// Decompose a code point into its UTF-16 representation.
fn utf16_encode(code: u32) -> (u16, u16) {
  const NON_BMP_MIN: u32 = 0x1_0000;
  const LEAD_SURROGATE_MIN: u16 = 0xD800;
  const TRAIL_SURROGATE_MIN: u16 = 0xDC00;

  let lead = ((code - NON_BMP_MIN) / 1024) as u16 + LEAD_SURROGATE_MIN;
  let trail = ((code - NON_BMP_MIN) % 1024) as u16 + TRAIL_SURROGATE_MIN;

  (lead, trail)
}

#[test]
fn check_utf16_encode() {
  assert_eq!(utf16_encode(0x10400), (0xD801, 0xDC00));
}

/// A representation of a non-BMP code point range beginning at `start` of
/// `length` code points (potentially 1 for a range of a single code point),
/// with each code point case-mapping by adding (with wrapping) `delta` to it,
/// where the UTF-16 decomposition of `start` is `(lead, trail)`.
#[derive(Copy, Clone)]
struct CaseMappingRange {
  start: u32,
  length: u16,
  delta: u16,
  lead: u16,
  trail: u16,
}

/// From the case mappings in `case_map`, return a list of `CaseMappingRange`s
/// within it.
fn compute_conversion_ranges(case_map: &types::CaseMap) -> Vec<CaseMappingRange> {
  Itertools::coalesce(
    case_map.iter().map(|(code, mapped)| {
      let (code, mapped) = (*code, *mapped);

      let (lead, trail) = utf16_encode(code);
      let (mapped_lead, mapped_trail) = utf16_encode(mapped);
      assert_eq!(
        lead, mapped_lead,
        "to_{{lower,upper}}_case_non_bmp_trail assumes that in UTF-16, every
         non-BMP code point has the same lead code unit as its case mapping,
         and only the trailing unit may differ"
      );

      let delta = u16::wrapping_sub(mapped_trail, trail);

      CaseMappingRange {
        start: code,
        length: 1,
        delta,
        lead,
        trail,
      }
    }),
    |range1, range2| {
      if range1.start + range1.length as u32 == range2.start
        && range1.delta == range2.delta
        && range1.lead == range2.lead
      {
        Ok(CaseMappingRange {
          length: range1.length + 1,
          ..range1
        })
      } else {
        Err((range1, range2))
      }
    },
  )
  .collect()
}

/// Generate a `changes_when_{upper,lower}_cased_non_bmp` function that when
/// passed the UTF-16 decomposition of a non-BMP code point will return `true`
/// iff the cased form of the code point is different from the code point
/// itself.  (The great majority of non-BMP code points do not have cased forms,
/// and the ones that do have cased forms appear in only a handful of ranges.)
fn generate_changes_when_cased_non_bmp(
  case: Casing,
  case_map: &types::CaseMap,
) -> proc_macro2::TokenStream {
  let case = case.to_string();

  let doc = format!(
    r#"
For a non-BMP code point whose UTF-16 decomposition consists of `lead` and
`trail`, return `true` iff its {case}cased form differs from the code point.
"#,
    case = case
  )
  .trim()
  .to_string();

  let name = format_ident!("changes_when_{case}_cased_non_bmp", case = case);

  let ranges = compute_conversion_ranges(&case_map);

  let tests: Vec<_> = ranges
    .into_iter()
    .map(|range| {
      let CaseMappingRange {
        start: _,
        length,
        delta: _,
        lead,
        trail,
      } = range;
      let to_trail = trail + length - 1;
      quote! {
        if lead == #lead && #trail <= trail && trail <= #to_trail {
          return true;
        }
      }
    })
    .collect();

  quote! {
    #[doc = #doc]
    #[inline]
    pub fn #name(lead: u16, trail: u16) -> bool {
      #( #tests )*

      false
    }
  }
}

/// Generate `changes_when_{upper,lower}_cased_non_bmp` functions.
fn generate_changes_when_cased_non_bmp_functions(
  non_bmp: &non_bmp::NonBMPInfo,
) -> proc_macro2::TokenStream {
  let changes_when_upper_cased_non_bmp_function =
    generate_changes_when_cased_non_bmp(Casing::Upper, &non_bmp.uppercase_map);
  let changes_when_lower_cased_non_bmp_function =
    generate_changes_when_cased_non_bmp(Casing::Lower, &non_bmp.lowercase_map);

  quote! {
    #changes_when_upper_cased_non_bmp_function

    #changes_when_lower_cased_non_bmp_function
  }
}

/// Generate a `to_{upper,lower}_case_non_bmp_trail` function that returns the
/// UTF-16 trailing code unit of its cased form.
///
/// For most code points this will just return the `trail` code unit passed in.
/// For the few code points for which `changes_when_{upper,lower}_cased_non_bmp`
/// returns true, this will return a value that differs from the provided
/// `trail`.
fn generate_to_case_non_bmp_trail(
  case: Casing,
  case_map: &types::CaseMap,
) -> proc_macro2::TokenStream {
  let case = case.to_string();

  let doc = format!(
    r#"
For a non-BMP code point whose UTF-16 decomposition consists of `lead` and
`trail`, return the UTF-16 trailing unit of its {case}cased form.

For most code points this will just return `trail`.  For the few code points for
which `changes_when_{case}_cased_non_bmp` returns true, this will return a value
different from `trail`.
"#,
    case = case
  )
  .trim()
  .to_string();

  let name = format_ident!("to_{case}_case_non_bmp_trail", case = case);

  let ranges = compute_conversion_ranges(&case_map);

  let tests: Vec<_> = ranges
    .into_iter()
    .map(|range| {
      let CaseMappingRange {
        start: _,
        length,
        delta,
        lead,
        trail,
      } = range;
      let to_trail = trail + length - 1;
      quote! {
        if lead == #lead && #trail <= trail && trail <= #to_trail {
          return u16::wrapping_add(trail, #delta);
        }
      }
    })
    .collect();

  quote! {
    #[doc = #doc]
    #[inline]
    pub fn #name(lead: u16, trail: u16) -> u16 {
      #( #tests )*

      trail
    }
  }
}

/// Generate `to_{upper,lower}_case_non_bmp_trail` functions.
fn generate_to_case_non_bmp_trail_functions(
  non_bmp: &non_bmp::NonBMPInfo,
) -> proc_macro2::TokenStream {
  let to_upper_case_non_bmp_trail_function =
    generate_to_case_non_bmp_trail(Casing::Upper, &non_bmp.uppercase_map);

  let to_lower_case_non_bmp_trail_function =
    generate_to_case_non_bmp_trail(Casing::Lower, &non_bmp.lowercase_map);

  quote! {
    #to_upper_case_non_bmp_trail_function

    #to_lower_case_non_bmp_trail_function
  }
}

/// Generate functions that indicate whether a non-BMP code point
/// {upper,lower}cases to a different value and what the trailing code unit in
/// the {upper,lower}cased form of a non-BMP code point will be.
pub fn generate_non_bmp_case_functions(non_bmp: &non_bmp::NonBMPInfo) -> proc_macro2::TokenStream {
  let changes_when_cased_non_bmp_functions =
    generate_changes_when_cased_non_bmp_functions(&non_bmp);

  let to_case_non_bmp_trail_functions = generate_to_case_non_bmp_trail_functions(&non_bmp);

  quote! {
    #changes_when_cased_non_bmp_functions

    #to_case_non_bmp_trail_functions
  }
}
