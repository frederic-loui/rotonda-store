use inetnum::addr::Prefix;
use rotonda_store::epoch;
use rotonda_store::match_options::{IncludeHistory, MatchOptions, MatchType};
use rotonda_store::prefix_record::{PrefixRecord, Record, RouteStatus};
use rotonda_store::rib::config::MemoryOnlyConfig;
use rotonda_store::rib::StarCastRib;
use rotonda_store::test_types::PrefixAs;

use std::error::Error;
use std::fs::File;
use std::process;

// #[create_store((
//     ([4, 4, 4, 4, 4, 4, 4, 4], 5, 17),
//     ([3, 4, 5, 4], 17, 29)
// ))]
// struct MyStore;

fn main() -> Result<(), Box<dyn Error>> {
    const CSV_FILE_PATH: &str = "./data/uniq_pfx_asn_dfz_rnd.csv";

    fn load_prefixes(
        pfxs: &mut Vec<PrefixRecord<PrefixAs>>,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::open(CSV_FILE_PATH)?;
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.records() {
            let record = result?;
            let ip: Vec<_> = record[0]
                .split('.')
                .map(|o| -> u8 { o.parse().unwrap() })
                .collect();
            let net = std::net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
            let len: u8 = record[1].parse().unwrap();
            let asn: u32 = record[2].parse().unwrap();
            let pfx = PrefixRecord::<PrefixAs>::new(
                Prefix::new(net.into(), len)?,
                vec![Record::new(
                    0,
                    0,
                    RouteStatus::Active,
                    PrefixAs::new(asn.into()),
                )],
            );
            pfxs.push(pfx);
        }
        Ok(())
    }

    println!("[");
    let strides_vec = [vec![4, 4, 4, 4, 4, 4, 4, 4], vec![3, 4, 5, 4]];

    for strides in strides_vec.iter().enumerate() {
        println!("[");
        for n in 1..6 {
            let mut rec_vec: Vec<PrefixRecord<PrefixAs>> = vec![];
            let config = MemoryOnlyConfig;
            let tree_bitmap =
                StarCastRib::<PrefixAs, _>::new_with_config(config)?;

            if let Err(err) = load_prefixes(&mut rec_vec) {
                println!("error running example: {}", err);
                process::exit(1);
            }
            // println!("finished loading {} prefixes...", pfxs.len());
            let start = std::time::Instant::now();

            let inserts_num = rec_vec.len();
            for rec in rec_vec.into_iter() {
                tree_bitmap.insert(&rec.prefix, rec.meta[0].clone(), None)?;
            }
            let ready = std::time::Instant::now();
            let dur_insert_nanos =
                ready.checked_duration_since(start).unwrap().as_nanos();

            let inet_max = 255;
            let len_max = 32;

            let start = std::time::Instant::now();
            let guard = &epoch::pin();
            // let locks = tree_bitmap.acquire_prefixes_rwlock_read();
            for i_net in 0..inet_max {
                for s_len in 0..len_max {
                    for ii_net in 0..inet_max {
                        if let Ok(pfx) = Prefix::new(
                            std::net::Ipv4Addr::new(i_net, ii_net, 0, 0)
                                .into(),
                            s_len,
                        ) {
                            tree_bitmap.match_prefix(
                                // (&locks.0, &locks.1),
                                &pfx,
                                &MatchOptions {
                                    match_type: MatchType::LongestMatch,
                                    include_withdrawn: false,
                                    include_less_specifics: false,
                                    include_more_specifics: false,
                                    mui: None,
                                    include_history: IncludeHistory::None,
                                },
                                guard,
                            )?;
                        }
                    }
                }
            }
            let ready = std::time::Instant::now();
            let dur_search_nanos =
                ready.checked_duration_since(start).unwrap().as_nanos();
            let searches_num =
                inet_max as u128 * inet_max as u128 * len_max as u128;

            println!("{{");
            println!("\"type\": \"treebitmap_univec\",");
            // println!(
            //     "\"strides v4 \": {:?},",
            //     &tree_bitmap
            //         .v4
            //         .get_stride_sizes()
            //         .iter()
            //         .map_while(|s| if s > &0 { Some(*s) } else { None })
            //         .collect::<Vec<_>>()
            // );
            // println!(
            //     "\"strides v6 \": {:?},",
            //     &tree_bitmap
            //         .v6
            //         .get_stride_sizes()
            //         .iter()
            //         .map_while(|s| if s > &0 { Some(*s) } else { None })
            //         .collect::<Vec<_>>()
            // );
            println!("\"run_no\": {},", n);
            println!("\"inserts_num\": {},", inserts_num);
            println!("\"insert_duration_nanos\": {},", dur_insert_nanos);
            println!(
                "\"global_prefix_vec_size\": {:?},",
                tree_bitmap.prefixes_count()
            );
            println!(
                "\"global_node_vec_size\": {},",
                tree_bitmap.nodes_count()
            );
            println!(
                "\"insert_time_nanos\": {},",
                dur_insert_nanos as f32 / inserts_num as f32
            );
            println!("\"searches_num\": {},", searches_num);
            println!("\"search_duration_nanos\": {},", dur_search_nanos);
            println!(
                "\"search_time_nanos\": {}",
                dur_search_nanos as f32 / searches_num as f32
            );
            println!("}}{}", if n != 5 { "," } else { "" });
        }

        println!(
            "]{}",
            if strides.0 != strides_vec.len() - 1 {
                ","
            } else {
                ""
            }
        );
    }
    println!("]");
    Ok(())
}
