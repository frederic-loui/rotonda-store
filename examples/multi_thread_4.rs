use log::trace;
use std::time::Duration;
use std::{sync::Arc, thread};


use rotonda_store::prelude::*;
use rotonda_store::prelude::multi::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ComplexPrefixAs(pub Vec<u32>);

// impl MergeUpdate for ComplexPrefixAs {
//     type UserDataIn = ();
//     type UserDataOut = ();

//     fn merge_update(
//         &mut self,
//         update_record: ComplexPrefixAs,
//         _: Option<&Self::UserDataIn>,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         self.0 = update_record.0;
//         Ok(())
//     }

//     fn clone_merge_update(
//         &self,
//         update_meta: &Self,
//         _: Option<&Self::UserDataIn>,
//     ) -> Result<(Self, Self::UserDataOut), Box<dyn std::error::Error>>
//     where
//         Self: std::marker::Sized,
//     {
//         let mut new_meta = update_meta.0.clone();
//         new_meta.push(self.0[0]);
//         Ok((ComplexPrefixAs(new_meta), ()))
//     }
// }

impl Meta for ComplexPrefixAs {
    type Orderable<'a> = ComplexPrefixAs;
    type TBI = ();
    
    fn as_orderable(&self, _tbi: Self::TBI) -> ComplexPrefixAs {
        self.clone()
    }
}

impl std::fmt::Display for ComplexPrefixAs {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AS{:?}", self.0)
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "cli")]
    env_logger::init();

    trace!("Starting multi-threaded yolo testing....");
    let tree_bitmap = Arc::new(MultiThreadedStore::<ComplexPrefixAs>::new()?);
    let f = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let pfx = Prefix::new_relaxed(
        0b1111_1111_1111_1111_1111_1111_1111_1111_u32.into_ipaddr(),
        32,
    );

    let threads = (0..1).enumerate().map(|(i, _)| {
        let tree_bitmap = tree_bitmap.clone();
        // let start_flag = Arc::clone(&f);

        std::thread::Builder::new()
            .name(i.to_string())
            .spawn(
                move || -> Result<(), Box<dyn std::error::Error + Send>> {
                    println!("park thread {}", i);
                    thread::park();

                    print!("\nstart {} ---", i);
                    let mut x: u32 = 0;
                    loop {
                        x += 1;

                        let pfx = Prefix::new_relaxed(x.into_ipaddr(), 32);
                        // print!("{}-", i);
                        match tree_bitmap.insert(
                            &pfx.unwrap(),
                            Record::new(
                                0,
                                0,
                                RouteStatus::Active,
                                ComplexPrefixAs([i as u32].to_vec()),
                            ),
                            None
                        ) {
                            Ok(metrics) => {
                                if metrics.cas_count > 0 {
                                    eprintln!("{} {} {:?} retry count: {},", std::thread::current().name().unwrap(), metrics.prefix_new, pfx, metrics.cas_count);
                                }
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        };

                        if x % 1_000_000 == 0 {
                            println!(
                                "{:?} {} (prefixes count: {}, nodes count: {}",
                                std::thread::current().name(),
                                x,
                                tree_bitmap.prefixes_count(),
                                tree_bitmap.nodes_count()
                            );
                        }

                    
                    }
                },
            )
            .unwrap()
    });

    f.store(true, std::sync::atomic::Ordering::Release);
    threads.for_each(|t| {
        t.thread().unpark();
    });

    thread::sleep(Duration::from_secs(60));

    println!("------ end of inserts\n");

    let guard = &epoch::pin();

    let s_spfx = tree_bitmap.match_prefix(
        &pfx.unwrap(),
        &MatchOptions {
            match_type: rotonda_store::MatchType::ExactMatch,
            include_withdrawn: true,
            include_less_specifics: true,
            include_more_specifics: true,
            mui: None
        },
        guard,
    );
    println!("query result");
    println!("{}", s_spfx);
    println!("{}", s_spfx.more_specifics.unwrap());

    println!("-----------");

    Ok(())
}
