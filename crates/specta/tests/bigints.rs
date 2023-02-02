use specta::ts::{BigIntExportBehavior, ExportConfiguration};

macro_rules! for_bigint_types {
    (T -> $s:expr) => {{
        for_bigint_types!(usize, isize, i64, u64, i128, u128; $s);
    }};
    ($($i:ty),+; $s:expr) => {{
        $({
            type T = $i;
            $s
        })*
    }};
}

#[test]
fn test_bigint_types() {
    // TODO: Assert error type is exactly what is expected for these ones
    for_bigint_types!(T -> assert!(specta::ts::inline::<T>(&ExportConfiguration::default()).is_err()));
    for_bigint_types!(T -> assert!(specta::ts::inline::<T>(&ExportConfiguration { bigint: BigIntExportBehavior::Fail, ..Default::default() }).is_err()));
    for_bigint_types!(T -> assert!(specta::ts::inline::<T>(&ExportConfiguration { bigint: BigIntExportBehavior::FailWithReason("some reason"), ..Default::default() }).is_err()));

    for_bigint_types!(T -> assert_eq!(specta::ts::inline::<T>(&ExportConfiguration { bigint: BigIntExportBehavior::String, ..Default::default() }).unwrap(), "string"));
    for_bigint_types!(T -> assert_eq!(specta::ts::inline::<T>(&ExportConfiguration { bigint: BigIntExportBehavior::Number, ..Default::default() }).unwrap(), "number"));
    for_bigint_types!(T -> assert_eq!(specta::ts::inline::<T>(&ExportConfiguration { bigint: BigIntExportBehavior::BigInt, ..Default::default() }).unwrap(), "BigInt"));
}
