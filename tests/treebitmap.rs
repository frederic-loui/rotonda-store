mod common {
    use std::io::Write;

    pub fn init() {
        let _ = env_logger::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .is_test(true)
            .try_init();
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use inetnum::addr::Prefix;
    use rotonda_store::{
        meta_examples::{NoMeta, PrefixAs},
        prelude::multi::*,
        prelude::*,
    };

    #[test]
    fn test_insert_extremes_ipv4() -> Result<(), Box<dyn std::error::Error>> {
        let trie = &mut MultiThreadedStore::<NoMeta>::new()?;
        let min_pfx = Prefix::new_relaxed(
            std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
            1,
        )
        .unwrap();

        trie.insert(
            &min_pfx,
            Record::new(0, 0, RouteStatus::Active, NoMeta::Empty),
            None,
        )?;
        let expect_pfx = Prefix::new_relaxed(
            std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
            1,
        );

        let guard = &epoch::pin();
        let res = trie.match_prefix(
            &expect_pfx?,
            &MatchOptions {
                match_type: MatchType::LongestMatch,
                include_withdrawn: false,
                include_less_specifics: true,
                include_more_specifics: false,
                mui: None,
            },
            guard,
        );
        println!("prefix: {:?}", &expect_pfx);
        println!("result: {:#?}", &res);
        assert!(res.prefix.is_some());
        assert_eq!(res.prefix, Some(expect_pfx?));

        let max_pfx = Prefix::new_relaxed(
            std::net::Ipv4Addr::new(255, 255, 255, 255).into(),
            32,
        );

        // drop(locks);
        trie.insert(
            &max_pfx?,
            Record::new(0, 0, RouteStatus::Active, NoMeta::Empty),
            None,
        )?;
        let expect_pfx = Prefix::new_relaxed(
            std::net::Ipv4Addr::new(255, 255, 255, 255).into(),
            32,
        );

        // let guard = &epoch::pin();
        let res = trie.match_prefix(
            &expect_pfx?,
            &MatchOptions {
                match_type: MatchType::ExactMatch,
                include_withdrawn: false,
                include_less_specifics: true,
                include_more_specifics: false,
                mui: None,
            },
            guard,
        );
        assert!(res.prefix.is_some());
        assert_eq!(res.prefix, Some(expect_pfx?));
        Ok(())
    }

    #[test]
    fn test_tree_ipv4() -> Result<(), Box<dyn std::error::Error>> {
        crate::common::init();

        let tree_bitmap = MultiThreadedStore::<PrefixAs>::new()?;
        let pfxs = vec![
            // Prefix::new_relaxed(0b0000_0000_0000_0000_0000_0000_0000_000 0_u32.into_ipaddr(), 0),
            Prefix::new_relaxed(
                0b1111_1111_1111_1111_1111_1111_1111_1111_u32.into_ipaddr(),
                32,
            ),
            Prefix::new_relaxed(
                0b0000_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b0001_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b0010_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b0011_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b0100_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b0101_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b0110_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b0111_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1000_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1001_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1010_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1011_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1100_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1101_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1110_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1111_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1111_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                9,
            ),
            Prefix::new_relaxed(
                0b1111_0000_1000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                9,
            ),
            Prefix::new_relaxed(
                0b0111_0111_1000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                12,
            ),
            Prefix::new_relaxed(
                0b1111_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                9,
            ),
            Prefix::new_relaxed(
                0b0111_0111_1000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                9,
            ),
            Prefix::new_relaxed(
                0b0111_0111_1000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                10,
            ),
            Prefix::new_relaxed(
                0b0111_0111_1000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                11,
            ),
            Prefix::new_relaxed(
                0b0111_0111_1000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                12,
            ),
            Prefix::new_relaxed(
                0b0111_0111_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                12,
            ),
            Prefix::new_relaxed(
                0b0111_0111_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                13,
            ),
            Prefix::new_relaxed(
                0b0111_0111_1000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                13,
            ),
            Prefix::new_relaxed(
                0b0111_0111_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                14,
            ),
            Prefix::new_relaxed(
                0b0111_0111_0100_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                14,
            ),
            Prefix::new_relaxed(
                0b0111_0111_1000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                14,
            ),
            Prefix::new_relaxed(
                0b0111_0111_1100_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                14,
            ),
            Prefix::new_relaxed(
                0b1110_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(192, 0, 0, 0).into(),
                23,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(192, 0, 0, 0).into(),
                16,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(192, 0, 10, 0).into(),
                23,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(192, 0, 9, 0).into(),
                24,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(193, 0, 0, 0).into(),
                23,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(193, 0, 10, 0).into(),
                23,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(209, 0, 0, 0).into(),
                16,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(193, 0, 9, 0).into(),
                24,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(193, 0, 10, 0).into(),
                24,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(193, 0, 14, 0).into(),
                23,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(193, 0, 14, 0).into(),
                24,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(193, 0, 15, 0).into(),
                24,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(193, 0, 10, 10).into(),
                32,
            ),
            Prefix::new_relaxed(
                0b0011_0000_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                4,
            ),
            Prefix::new_relaxed(
                0b1000_0011_1000_1111_0000_0000_0000_0000_u32.into_ipaddr(),
                11,
            ),
            Prefix::new_relaxed(
                0b1000_0010_0101_0111_1111_1000_0000_0000_u32.into_ipaddr(),
                13,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(130, 55, 240, 0).into(),
                24,
            ),
            Prefix::new_relaxed(
                0b1111_1111_0000_0001_0000_0000_0000_0000_u32.into_ipaddr(),
                12,
            ),
            Prefix::new_relaxed(
                0b1111_1111_0011_0111_0000_0000_0000_0000_u32.into_ipaddr(),
                17,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(100, 0, 12, 0).into(),
                24,
            ),
            Prefix::new_relaxed(
                0b0000_0001_0000_0000_0000_0000_0000_0000_u32.into_ipaddr(),
                24,
            ),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(1, 0, 128, 0).into(),
                24,
            ),
        ];

        for pfx in pfxs.into_iter() {
            tree_bitmap.insert(
                &pfx?,
                Record::new(0, 0, RouteStatus::Active, PrefixAs(666)),
                None,
            )?;
        }

        // let (store_v4, store_v6) = tree_bitmap.acquire_prefixes_rwlock_read();
        // let prefixes_iter = rotonda_store::HashMapPrefixRecordIterator {
        //     v4: Some(store_v4),
        //     v6: store_v6,
        // };

        let guard = &epoch::pin();
        for pfx in tree_bitmap.prefixes_iter() {
            // let pfx_nm = pfx.strip_meta();
            let res = tree_bitmap.match_prefix(
                &pfx.prefix,
                &MatchOptions {
                    match_type: MatchType::LongestMatch,
                    include_withdrawn: false,
                    include_less_specifics: false,
                    include_more_specifics: false,
                    mui: None,
                },
                guard,
            );
            println!("{}", pfx);
            assert_eq!(res.prefix.unwrap(), pfx.prefix);
        }

        let res = tree_bitmap.match_prefix(
            &Prefix::new(std::net::Ipv4Addr::new(192, 0, 1, 0).into(), 24)?,
            &MatchOptions {
                match_type: MatchType::LongestMatch,
                include_withdrawn: false,
                include_less_specifics: true,
                include_more_specifics: false,
                mui: None,
            },
            guard,
        );
        println!("prefix {:?}", res.prefix);
        println!("res: {:#?}", &res);

        assert_eq!(
            res.prefix.unwrap(),
            Prefix::new_relaxed(
                std::net::Ipv4Addr::new(192, 0, 0, 0).into(),
                23
            )?
        );

        let less_specifics = res.less_specifics.unwrap();

        assert!(less_specifics.iter().any(|r| {
            r.prefix
                == Prefix::new(
                    std::net::Ipv4Addr::new(192, 0, 0, 0).into(),
                    16,
                )
                .unwrap()
        }));
        assert!(less_specifics.iter().any(|r| {
            r.prefix
                == Prefix::new(
                    std::net::Ipv4Addr::new(192, 0, 0, 0).into(),
                    4,
                )
                .unwrap()
        }));
        Ok(())
    }

    #[test]
    fn test_ranges_ipv4() -> Result<(), Box<dyn std::error::Error>> {
        for i_net in 0..255 {
            let tree_bitmap = MultiThreadedStore::<NoMeta>::new()?;

            let pfx_vec: Vec<Prefix> = (1..32)
                .collect::<Vec<u8>>()
                .into_iter()
                .map(|i_len| {
                    Prefix::new_relaxed(
                        std::net::Ipv4Addr::new(i_net, 0, 0, 0).into(),
                        i_len,
                    )
                    .unwrap()
                })
                .collect();

            let mut i_len_s = 0;
            for pfx in pfx_vec {
                i_len_s += 1;
                tree_bitmap.insert(
                    &pfx,
                    Record::new(0, 0, RouteStatus::Active, NoMeta::Empty),
                    None,
                )?;

                let res_pfx = Prefix::new_relaxed(
                    std::net::Ipv4Addr::new(i_net, 0, 0, 0).into(),
                    i_len_s,
                );

                let guard = &epoch::pin();
                for s_len in i_len_s..32 {
                    let pfx = Prefix::new_relaxed(
                        std::net::Ipv4Addr::new(i_net, 0, 0, 0).into(),
                        s_len,
                    )?;
                    let res = tree_bitmap.match_prefix(
                        &pfx,
                        &MatchOptions {
                            match_type: MatchType::LongestMatch,
                            include_withdrawn: false,
                            include_less_specifics: false,
                            include_more_specifics: false,
                            mui: None,
                        },
                        guard,
                    );
                    println!("{:?}", pfx);

                    assert_eq!(res.prefix.unwrap(), res_pfx?);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_multi_ranges_ipv4() -> Result<(), Box<dyn std::error::Error>> {
        crate::common::init();

        let tree_bitmap = MultiThreadedStore::<NoMeta>::new()?;
        for mui in [1_u32, 2, 3, 4, 5] {
            println!("Multi Uniq ID {mui}");

            for i_net in 0..2 {
                let pfx_vec: Vec<Prefix> = (16..18)
                    .collect::<Vec<u8>>()
                    .into_iter()
                    .map(|i_len| {
                        Prefix::new_relaxed(
                            std::net::Ipv4Addr::new(i_net, 0, 0, 0).into(),
                            i_len,
                        )
                        .unwrap()
                    })
                    .collect();

                let mut i_len_s = 0;

                for pfx in &pfx_vec {
                    i_len_s += 1;
                    tree_bitmap.insert(
                        pfx,
                        Record::new(
                            mui,
                            0,
                            RouteStatus::Active,
                            NoMeta::Empty,
                        ),
                        None,
                    )?;

                    let _res_pfx = Prefix::new_relaxed(
                        std::net::Ipv4Addr::new(i_net, 0, 0, 0).into(),
                        i_len_s,
                    );

                    let guard = &epoch::pin();

                    for s_len in i_len_s..4 {
                        let pfx = Prefix::new_relaxed(
                            std::net::Ipv4Addr::new(i_net, 0, 0, 0).into(),
                            s_len,
                        )?;
                        let _res = tree_bitmap.match_prefix(
                            &pfx,
                            &MatchOptions {
                                match_type: MatchType::LongestMatch,
                                include_withdrawn: false,
                                include_less_specifics: false,
                                include_more_specifics: false,
                                mui: Some(mui),
                            },
                            guard,
                        );
                        // println!("{:?}", pfx);

                        // assert_eq!(res.prefix.unwrap(), res_pfx?);
                    }
                }
            }
        }

        let guard = &epoch::pin();
        println!("records for mui {}", 5);
        for rec in tree_bitmap
            .iter_records_for_mui_v4(5, false, guard)
            .collect::<Vec<_>>()
        {
            println!("{}", rec);

            assert_eq!(rec.meta.len(), 1);
            assert_eq!(rec.meta[0].multi_uniq_id, 5);
            assert_eq!(rec.meta[0].status, RouteStatus::Active);
        }
        for rec in tree_bitmap
            .iter_records_for_mui_v4(1, false, guard)
            .collect::<Vec<_>>()
        {
            println!("{}", rec);
        }

        // println!("all records");
        // for rec in tree_bitmap.prefixes_iter(guard).collect::<Vec<_>>() {
        //     println!("{}", rec);
        // };

        // Withdraw records for mui 1 globally.
        tree_bitmap.mark_mui_as_withdrawn_v4(1)?;

        let all_recs_for_pfx = tree_bitmap.match_prefix(
            &Prefix::from_str("1.0.0.0/16")?,
            &MatchOptions {
                match_type: MatchType::LongestMatch,
                include_withdrawn: true,
                include_less_specifics: false,
                include_more_specifics: false,
                mui: None,
            },
            guard,
        );
        print!(".pfx {:#?}.", all_recs_for_pfx);
        assert_eq!(all_recs_for_pfx.prefix_meta.len(), 5);
        let wd_rec = all_recs_for_pfx
            .prefix_meta
            .iter()
            .filter(|r| r.status == RouteStatus::Withdrawn)
            .collect::<Vec<_>>();
        assert_eq!(wd_rec.len(), 1);
        assert_eq!(wd_rec[0].multi_uniq_id, 1);

        let active_recs_for_pfx = tree_bitmap.match_prefix(
            &Prefix::from_str("1.0.0.0/16")?,
            &MatchOptions {
                match_type: MatchType::LongestMatch,
                include_withdrawn: false,
                include_less_specifics: false,
                include_more_specifics: false,
                mui: None,
            },
            guard,
        );
        assert_eq!(active_recs_for_pfx.prefix_meta.len(), 4);
        assert!(!active_recs_for_pfx
            .prefix_meta
            .iter()
            .any(|r| r.multi_uniq_id == 1));

        let wd_pfx = Prefix::from_str("1.0.0.0/16")?;
        tree_bitmap.mark_mui_as_withdrawn_for_prefix(&wd_pfx, 2)?;

        println!("all records");

        let all_recs = tree_bitmap.prefixes_iter();

        for rec in tree_bitmap.prefixes_iter().collect::<Vec<_>>() {
            println!("{}", rec);
        }

        let mui_2_recs =
            all_recs.filter_map(|r| r.get_record_for_mui(2).cloned());
        let wd_2_rec = mui_2_recs
            .filter(|r| r.status == RouteStatus::Withdrawn)
            .collect::<Vec<_>>();
        assert_eq!(wd_2_rec.len(), 1);
        assert_eq!(wd_2_rec[0].multi_uniq_id, 2);

        let mui_2_recs = tree_bitmap.prefixes_iter().filter_map(|r| {
            r.get_record_for_mui(2).cloned().map(|rec| (r.prefix, rec))
        });
        println!("mui_2_recs prefixes_iter");
        for rec in mui_2_recs {
            println!("{} {:#?}", rec.0, rec.1);
        }
        let mui_2_recs = tree_bitmap.prefixes_iter().filter_map(|r| {
            r.get_record_for_mui(2).cloned().map(|rec| (r.prefix, rec))
        });

        let active_2_rec = mui_2_recs
            .filter(|r| r.1.status == RouteStatus::Active)
            .collect::<Vec<_>>();
        assert_eq!(active_2_rec.len(), 3);
        assert!(!active_2_rec.iter().any(|r| r.0 == wd_pfx));

        let mui_2_recs = tree_bitmap.iter_records_for_mui_v4(2, false, guard);
        println!("mui_2_recs iter_records_for_mui_v4");
        for rec in mui_2_recs {
            println!("{} {:#?}", rec.prefix, rec.meta);
        }

        let mui_1_recs = tree_bitmap
            .iter_records_for_mui_v4(1, false, guard)
            .collect::<Vec<_>>();
        assert!(mui_1_recs.is_empty());

        println!("mui_1_recs iter_records_for_mui_v4");
        assert!(mui_1_recs.is_empty());

        let mui_1_recs = tree_bitmap
            .iter_records_for_mui_v4(1, true, guard)
            .collect::<Vec<_>>();
        assert_eq!(mui_1_recs.len(), 4);
        println!("mui_1_recs iter_records_for_mui_v4 w/ withdrawn");
        for rec in mui_1_recs {
            assert_eq!(rec.meta[0].status, RouteStatus::Withdrawn);
        }

        //--------------

        let more_specifics = tree_bitmap.match_prefix(
            &Prefix::from_str("1.0.0.0/16")?,
            &MatchOptions {
                match_type: MatchType::LongestMatch,
                include_withdrawn: true,
                include_less_specifics: false,
                include_more_specifics: true,
                mui: None,
            },
            guard,
        );

        println!("more_specifics match {} w/ withdrawn", more_specifics);
        let more_specifics = more_specifics.more_specifics.unwrap();
        assert_eq!(more_specifics.len(), 1);
        assert_eq!(more_specifics.v4.len(), 1);
        let more_specifics = &more_specifics.v4[0];
        assert_eq!(more_specifics.prefix, Prefix::from_str("1.0.0.0/17")?);
        assert_eq!(more_specifics.meta.len(), 5);
        assert_eq!(
            more_specifics
                .meta
                .iter()
                .filter(|r| r.status == RouteStatus::Active)
                .collect::<Vec<_>>()
                .len(),
            4
        );
        let rec = more_specifics
            .meta
            .iter()
            .filter(|r| r.status == RouteStatus::Withdrawn)
            .collect::<Vec<_>>();
        assert_eq!(rec.len(), 1);
        assert_eq!(rec[0].multi_uniq_id, 1);

        //---------------

        let more_specifics = tree_bitmap.match_prefix(
            &Prefix::from_str("1.0.0.0/16")?,
            &MatchOptions {
                match_type: MatchType::LongestMatch,
                include_withdrawn: false,
                include_less_specifics: false,
                include_more_specifics: true,
                mui: None,
            },
            guard,
        );

        println!("more_specifics match {} w/o withdrawn", more_specifics);
        let more_specifics = more_specifics.more_specifics.unwrap();
        assert_eq!(more_specifics.len(), 1);
        assert_eq!(more_specifics.v4.len(), 1);
        let more_specifics = &more_specifics.v4[0];
        assert_eq!(more_specifics.prefix, Prefix::from_str("1.0.0.0/17")?);
        assert_eq!(more_specifics.meta.len(), 4);
        assert_eq!(
            more_specifics
                .meta
                .iter()
                .filter(|r| r.status == RouteStatus::Active)
                .collect::<Vec<_>>()
                .len(),
            4
        );
        let rec = more_specifics
            .meta
            .iter()
            .filter(|r| r.status == RouteStatus::Withdrawn)
            .collect::<Vec<_>>();
        assert!(rec.is_empty());

        //------------------

        tree_bitmap.mark_mui_as_withdrawn_for_prefix(&wd_pfx, 1)?;
        tree_bitmap.mark_mui_as_active_v4(1)?;

        let more_specifics = tree_bitmap.match_prefix(
            &Prefix::from_str("1.0.0.0/16")?,
            &MatchOptions {
                match_type: MatchType::LongestMatch,
                include_withdrawn: false,
                include_less_specifics: false,
                include_more_specifics: true,
                mui: None,
            },
            guard,
        );

        println!("more_specifics match w/o withdrawn #2 {}", more_specifics);
        // We withdrew mui 1 for the requested prefix itself, since mui 2 was
        // already withdrawn above, we're left with 3 records
        assert_eq!(more_specifics.prefix_meta.len(), 3);

        let more_specifics = more_specifics.more_specifics.unwrap();
        assert_eq!(more_specifics.len(), 1);
        assert_eq!(more_specifics.v4.len(), 1);
        let more_specifics = &more_specifics.v4[0];
        assert_eq!(more_specifics.prefix, Prefix::from_str("1.0.0.0/17")?);

        // one more more_specific should have been added due to mui 1 being
        // Active again, for all but the requested prefix above.
        assert_eq!(more_specifics.meta.len(), 5);
        assert_eq!(
            more_specifics
                .meta
                .iter()
                .filter(|r| r.status == RouteStatus::Active)
                .collect::<Vec<_>>()
                .len(),
            5
        );

        // we didn't ask to see withdrawn routes
        let rec = more_specifics
            .meta
            .iter()
            .filter(|r| r.status == RouteStatus::Withdrawn)
            .collect::<Vec<_>>();
        assert!(rec.is_empty());

        // withdraw muis 2,3,4,5 for the requested prefix
        tree_bitmap.mark_mui_as_withdrawn_for_prefix(&wd_pfx, 2)?;
        tree_bitmap.mark_mui_as_withdrawn_for_prefix(&wd_pfx, 3)?;
        tree_bitmap.mark_mui_as_withdrawn_for_prefix(&wd_pfx, 4)?;
        tree_bitmap.mark_mui_as_withdrawn_for_prefix(&wd_pfx, 5)?;

        let more_specifics = tree_bitmap.match_prefix(
            &Prefix::from_str("1.0.0.0/16")?,
            &MatchOptions {
                match_type: MatchType::ExactMatch,
                include_withdrawn: false,
                include_less_specifics: false,
                include_more_specifics: true,
                mui: None,
            },
            guard,
        );
        println!("more_specifics match w/o withdrawn #3 {}", more_specifics);

        // This prefix should not be found, since we withdrew all records for it.
        assert!(more_specifics.prefix_meta.is_empty());

        // ..as a result, its resulting match_type should be EmptyMatch
        assert_eq!(more_specifics.match_type, MatchType::EmptyMatch);

        let more_specifics = more_specifics.more_specifics.unwrap();
        assert_eq!(more_specifics.len(), 1);
        assert_eq!(more_specifics.v4.len(), 1);
        let more_specifics = &more_specifics.v4[0];
        assert_eq!(more_specifics.prefix, Prefix::from_str("1.0.0.0/17")?);

        // all muis should be visible for the more specifics
        assert_eq!(more_specifics.meta.len(), 5);
        assert_eq!(
            more_specifics
                .meta
                .iter()
                .filter(|r| r.status == RouteStatus::Active)
                .collect::<Vec<_>>()
                .len(),
            5
        );

        // we didn't ask to see withdrawn routes,
        let rec = more_specifics
            .meta
            .iter()
            .filter(|r| r.status == RouteStatus::Withdrawn)
            .collect::<Vec<_>>();
        assert!(rec.is_empty());

        //----------------------

        // Change the requested prefix to the more specific from the former
        // queries.
        let less_specifics = tree_bitmap.match_prefix(
            &Prefix::from_str("1.0.0.0/17")?,
            &MatchOptions {
                match_type: MatchType::ExactMatch,
                include_withdrawn: false,
                include_less_specifics: true,
                include_more_specifics: false,
                mui: None,
            },
            guard,
        );

        println!("less_specifics match w/o withdrawn #4 {}", less_specifics);

        assert_eq!(less_specifics.prefix_meta.len(), 5);

        let less_specifics = less_specifics.less_specifics.unwrap();
        // All records for the less specific /16 are withdrawn, so this should be empty.
        assert!(less_specifics.is_empty());

        //--------------------

        tree_bitmap.mark_mui_as_active_for_prefix(&wd_pfx, 5)?;

        let less_specifics = tree_bitmap.match_prefix(
            &Prefix::from_str("1.0.0.0/17")?,
            &MatchOptions {
                match_type: MatchType::ExactMatch,
                include_withdrawn: false,
                include_less_specifics: true,
                include_more_specifics: false,
                mui: None,
            },
            guard,
        );
        println!("more_specifics match w/o withdrawn #5 {}", less_specifics);
        let less_specifics = less_specifics.less_specifics.unwrap();

        assert_eq!(less_specifics.v4.len(), 1);
        let less_specifics = &less_specifics.v4[0];
        assert_eq!(less_specifics.prefix, Prefix::from_str("1.0.0.0/16")?);
        // We should only see the record for mui 5
        assert_eq!(less_specifics.meta.len(), 1);
        assert_eq!(less_specifics.meta[0].multi_uniq_id, 5);
        assert_eq!(less_specifics.meta[0].status, RouteStatus::Active);

        Ok(())
    }
}
