#![cfg(feature = "testutils")]

use crate::arbitrary::SorobanArbitrary;
use crate::env::IntoVal;
use crate::testutils::{Compare, Tag};
use crate::xdr::ScVal;
use crate::{Env, RawVal};
use crate::{FromVal, TryFromVal};
use crate::{Map, Vec};
use core::cmp::Ordering;
use proptest::prelude::*;
use proptest_arbitrary_interop::arb;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100000))]

    #[test]
    fn vec_unequal_lengths(
        v1_vec in arb::<<Vec<u32> as SorobanArbitrary>::Prototype>(),
        v2_vec in arb::<<Vec<u32> as SorobanArbitrary>::Prototype>(),
    ) {
        let env = &Env::default();
        let budget = env.budget().0;

        let v1_vec: Vec<u32> = v1_vec.into_val(env);
        let v2_vec: Vec<u32> = v2_vec.into_val(env);

        let v1_rawval = RawVal::from_val(env, &v1_vec);
        let v2_rawval = RawVal::from_val(env, &v2_vec);

        let v1_scval = ScVal::try_from_val(env, &v1_rawval).unwrap();
        let v2_scval = ScVal::try_from_val(env, &v2_rawval).unwrap();

        let cmp_env = env.compare(&v1_rawval, &v2_rawval).unwrap();
        let cmp_budget = budget.compare(&v1_scval, &v2_scval).unwrap();

        prop_assert_eq!(cmp_env, cmp_budget);
    }

    #[test]
    fn map_unequal_lengths(
        v1_map in arb::<<Map<u32, u32> as SorobanArbitrary>::Prototype>(),
        v2_map in arb::<<Map<u32, u32> as SorobanArbitrary>::Prototype>(),
    ) {
        let env = &Env::default();
        let budget = env.budget().0;

        let v1_map: Map<u32, u32> = v1_map.into_val(env);
        let v2_map: Map<u32, u32> = v2_map.into_val(env);

        let v1_rawval = RawVal::from_val(env, &v1_map);
        let v2_rawval = RawVal::from_val(env, &v2_map);

        let v1_scval = ScVal::try_from_val(env, &v1_rawval).unwrap();
        let v2_scval = ScVal::try_from_val(env, &v2_rawval).unwrap();

        let cmp_env = env.compare(&v1_rawval, &v2_rawval).unwrap();
        let cmp_budget = budget.compare(&v1_scval, &v2_scval).unwrap();

        prop_assert_eq!(cmp_env, cmp_budget);
    }

    #[test]
    fn different_objects_cmp(
        rawval_proto_1 in arb::<<RawVal as SorobanArbitrary>::Prototype>(),
        rawval_proto_2 in arb::<<RawVal as SorobanArbitrary>::Prototype>(),
    ) {
        let env = &Env::default();
        let budget = env.budget().0;

        let rawval_1 = RawVal::from_val(env, &rawval_proto_1);
        let rawval_2 = RawVal::from_val(env, &rawval_proto_2);

        let rawval_tag_1 = rawval_1.get_tag();
        let rawval_tag_2 = rawval_2.get_tag();

        // do not compare the same type
        if rawval_tag_1 == rawval_tag_2 {
            return Ok(());
        }

        let (scval_1, scval_2) = {
            let scval_1 = ScVal::try_from_val(env, &rawval_1);
            let scval_2 = ScVal::try_from_val(env, &rawval_2);

            let scval_1 = match scval_1 {
                Ok(scval_1) => scval_1,
                Err(e) => {
                    // Some statuses can't be serialized
                    // Vec and Map that contains Status can't be serialized
                    if rawval_tag_1 == Tag::Status
                        || rawval_tag_1 == Tag::VecObject
                        || rawval_tag_1 == Tag::MapObject {
                            return Ok(());
                        }
                    panic!(
                        "couldn't convert rawval to scval:\n\
                         {rawval_1:?},\n\
                         {e:#?}"
                    );
                }
            };

            let scval_2 = match scval_2 {
                Ok(scval_2) => scval_2,
                Err(e) => {
                    // Some statuses can't be serialized
                    // Vec and Map that contains Status can't be serialized
                    if rawval_tag_2 == Tag::Status
                        || rawval_tag_2 == Tag::VecObject
                        || rawval_tag_2 == Tag::MapObject {
                            return Ok(());
                        }
                    panic!(
                        "couldn't convert rawval to scval:\n\
                         {rawval_2:?},\n\
                         {e:#?}"
                    );
                }
            };

            (scval_1, scval_2)
        };

        // Check the comparison functions
        {
            let rawval_cmp = env.compare(&rawval_1, &rawval_2);
            let rawval_cmp = rawval_cmp.expect("cmp");
            let scval_cmp = Ord::cmp(&scval_1, &scval_2);

            let rawval_cmp_is_eq = rawval_cmp == Ordering::Equal;

            if rawval_cmp != scval_cmp {
                panic!(
                    "rawval and scval don't compare the same:\n\
                     {rawval_1:#?}\n\
                     {rawval_2:#?}\n\
                     {rawval_cmp:#?}\n\
                     {scval_1:#?}\n\
                     {scval_2:#?}\n\
                     {scval_cmp:#?}"
                );
            }

            let scval_cmp_partial = PartialOrd::partial_cmp(&scval_1, &scval_2);

            prop_assert_eq!(Some(scval_cmp), scval_cmp_partial);

            let scval_partial_eq = PartialEq::eq(&scval_1, &scval_2);
            prop_assert_eq!(rawval_cmp_is_eq, scval_partial_eq);

            // Compare<ScVal> for Budget
            let scval_budget_cmp = budget.compare(&scval_1, &scval_2);
            let scval_budget_cmp = scval_budget_cmp.expect("cmp");
            if rawval_cmp != scval_budget_cmp {
                panic!(
                    "rawval and scval (budget) don't compare the same:\n\
                     {rawval_1:#?}\n\
                     {rawval_2:#?}\n\
                     {rawval_cmp:#?}\n\
                     {scval_1:#?}\n\
                     {scval_2:#?}\n\
                     {scval_budget_cmp:#?}"
                );
            }
        }

        // Roundtrip checks
        {
            let rawval_after_1 = RawVal::try_from_val(env, &scval_1);
            let rawval_after_1 = match rawval_after_1 {
                Ok(rawval_after_1) => rawval_after_1,
                Err(e) => {
                    panic!(
                        "couldn't convert scval to rawval:\n\
                         {scval_1:?},\n\
                         {e:#?}"
                    );
                }
            };

            let rawval_cmp_before_after_1 = env.compare(&rawval_1, &rawval_after_1).expect("compare");

            prop_assert_eq!(rawval_cmp_before_after_1, Ordering::Equal);

            let rawval_after_2 = RawVal::try_from_val(env, &scval_2);
            let rawval_after_2 = match rawval_after_2 {
                Ok(rawval_after_2) => rawval_after_2,
                Err(e) => {
                    panic!(
                        "couldn't convert scval to rawval:\n\
                         {scval_2:?},\n\
                         {e:#?}"
                    );
                }
            };

            let rawval_cmp_before_after_2 = env.compare(&rawval_2, &rawval_after_2).expect("compare");

            prop_assert_eq!(rawval_cmp_before_after_2, Ordering::Equal);
        }
    }
}
