use crossbeam_epoch::{self as epoch};
use epoch::Guard;
use log::trace;
use zerocopy::TryFromBytes;

use crate::errors::{FatalError, FatalResult};
use crate::match_options::{MatchOptions, MatchType, QueryResult};
use crate::prefix_record::RecordSet;
use crate::types::prefix_record::ZeroCopyRecord;
use crate::types::Record;
use crate::AddressFamily;
use crate::{prefix_record::Meta, rib::starcast_af::StarCastAfRib};
use inetnum::addr::Prefix;

use crate::types::errors::PrefixStoreError;
use crate::types::PrefixId;

use super::config::{Config, PersistStrategy};

//------------ Prefix Matching ----------------------------------------------

impl<
        'a,
        AF: AddressFamily,
        M: Meta,
        const N_ROOT_SIZE: usize,
        const P_ROOT_SIZE: usize,
        C: Config,
        const KEY_SIZE: usize,
    > StarCastAfRib<AF, M, N_ROOT_SIZE, P_ROOT_SIZE, C, KEY_SIZE>
{
    pub(crate) fn get_value(
        &'a self,
        prefix_id: PrefixId<AF>,
        mui: Option<u32>,
        include_withdrawn: bool,
        guard: &'a Guard,
    ) -> FatalResult<Option<Vec<Record<M>>>> {
        match self.persist_strategy() {
            PersistStrategy::PersistOnly => {
                trace!("get value from persist_store for {:?}", prefix_id);
                self.persist_tree
                    .as_ref()
                    .and_then(|tree| {
                        tree.records_for_prefix(
                            prefix_id,
                            mui,
                            include_withdrawn,
                            self.tree_bitmap.withdrawn_muis_bmin(guard),
                        )
                        .map(|v| {
                            v.iter()
                                .map(|bytes| {
                                    if let Ok(b) = bytes.as_ref() {
                                        let record: &ZeroCopyRecord<AF> =
                                        ZeroCopyRecord::try_ref_from_bytes(b)
                                            .map_err(|_| FatalError)?;
                                        Ok(Record::<M> {
                                            multi_uniq_id: record
                                                .multi_uniq_id,
                                            ltime: record.ltime,
                                            status: record.status,
                                            meta: <Vec<u8>>::from(
                                                record.meta.as_ref(),
                                            )
                                            .into(),
                                        })
                                    } else {
                                        Err(FatalError)
                                    }
                                })
                                .collect::<FatalResult<Vec<_>>>()
                        })
                    })
                    .transpose()
            }
            _ => Ok(self.prefix_cht.get_records_for_prefix(
                prefix_id,
                mui,
                include_withdrawn,
                self.tree_bitmap.withdrawn_muis_bmin(guard),
            )),
        }
    }

    pub(crate) fn more_specifics_from(
        &'a self,
        prefix_id: PrefixId<AF>,
        mui: Option<u32>,
        include_withdrawn: bool,
        guard: &'a Guard,
    ) -> FatalResult<QueryResult<M>> {
        let prefix = if !self.contains(prefix_id, mui) {
            Some(Prefix::from(prefix_id))
        } else {
            None
        };

        let records = self
            .get_value(prefix_id, mui, include_withdrawn, guard)?
            .unwrap_or_default();

        let more_specifics = self
            .tree_bitmap
            .more_specific_prefix_iter_from(prefix_id)
            .map(|p| {
                self.get_value(prefix_id, mui, include_withdrawn, guard)
                    .map(|res| res.map(|v| (p, v)))
            })
            .collect::<FatalResult<Option<RecordSet<M>>>>()?;

        Ok(QueryResult {
            prefix,
            records,
            match_type: MatchType::EmptyMatch,
            less_specifics: None,
            more_specifics,
        })
    }

    pub(crate) fn less_specifics_from(
        &'a self,
        prefix_id: PrefixId<AF>,
        mui: Option<u32>,
        include_withdrawn: bool,
        guard: &'a Guard,
    ) -> FatalResult<QueryResult<M>> {
        let prefix = if !self.contains(prefix_id, mui) {
            Some(Prefix::from(prefix_id))
        } else {
            None
        };
        let prefix_meta = self
            .get_value(prefix_id, mui, include_withdrawn, guard)?
            .unwrap_or_default();

        let less_specifics = self
            .tree_bitmap
            .less_specific_prefix_iter(prefix_id)
            .map(|p| {
                self.get_value(prefix_id, mui, include_withdrawn, guard)
                    .map(|res| res.map(|v| (p, v)))
            })
            .collect::<FatalResult<Option<RecordSet<M>>>>()?;

        Ok(QueryResult {
            prefix,
            records: prefix_meta,
            match_type: MatchType::EmptyMatch,
            less_specifics,
            more_specifics: None,
        })
    }

    pub(crate) fn more_specifics_iter_from(
        &'a self,
        prefix_id: PrefixId<AF>,
        mui: Option<u32>,
        include_withdrawn: bool,
        guard: &'a Guard,
    ) -> impl Iterator<Item = FatalResult<(PrefixId<AF>, Vec<Record<M>>)>> + 'a
    {
        println!("more_specifics_iter_from fn");
        // If the user wanted a specific mui and not withdrawn prefixes, we
        // may return early if the mui is globally withdrawn.
        (if mui.is_some_and(|m| {
            !include_withdrawn && self.mui_is_withdrawn(m, guard)
        }) {
            None
        } else {
            Some(
                self.tree_bitmap
                    .more_specific_prefix_iter_from(prefix_id)
                    .filter_map(move |p| {
                        self.get_value(p, mui, include_withdrawn, guard)
                            .map(|res| res.map(|v| (p, v)))
                            .transpose()
                    }),
            )
        })
        .into_iter()
        .flatten()
    }

    pub(crate) fn less_specifics_iter_from(
        &'a self,
        prefix_id: PrefixId<AF>,
        mui: Option<u32>,
        include_withdrawn: bool,
        guard: &'a Guard,
    ) -> impl Iterator<Item = FatalResult<(PrefixId<AF>, Vec<Record<M>>)>> + 'a
    {
        self.tree_bitmap
            .less_specific_prefix_iter(prefix_id)
            .filter_map(move |p| {
                self.get_value(p, mui, include_withdrawn, guard)
                    .map(|res| res.map(|v| (p, v)))
                    .transpose()
            })
    }

    pub(crate) fn match_prefix(
        &'a self,
        search_pfx: PrefixId<AF>,
        options: &MatchOptions,
        guard: &'a Guard,
    ) -> FatalResult<QueryResult<M>> {
        trace!("match_prefix rib {:?} {:?}", search_pfx, options);
        let res = self.tree_bitmap.match_prefix(search_pfx, options);

        trace!("res {:?}", res);
        let mut res = QueryResult::from(res);

        if let Some(Ok(Some(m))) = res.prefix.map(|p| {
            self.get_value(
                p.into(),
                options.mui,
                options.include_withdrawn,
                guard,
            )
            .map(|res| {
                res.and_then(|v| if v.is_empty() { None } else { Some(v) })
            })
        }) {
            res.records = m;
        } else {
            res.prefix = None;
            res.match_type = MatchType::EmptyMatch;
        }

        if options.include_more_specifics {
            res.more_specifics = res
                .more_specifics
                .map(|p| {
                    p.iter()
                        .filter_map(|mut r| {
                            if let Ok(mm) = self.get_value(
                                r.prefix.into(),
                                options.mui,
                                options.include_withdrawn,
                                guard,
                            ) {
                                if let Some(m) = mm {
                                    r.meta = m;
                                    Some(Ok(r))
                                } else {
                                    None
                                }
                            } else {
                                Some(Err(FatalError))
                            }
                        })
                        .collect::<FatalResult<RecordSet<M>>>()
                })
                .transpose()?;
        }
        if options.include_less_specifics {
            res.less_specifics = res
                .less_specifics
                .map(|p| {
                    p.iter()
                        .filter_map(|mut r| {
                            if let Ok(mm) = self.get_value(
                                r.prefix.into(),
                                options.mui,
                                options.include_withdrawn,
                                guard,
                            ) {
                                if let Some(m) = mm {
                                    r.meta = m;
                                    Some(Ok(r))
                                } else {
                                    None
                                }
                            } else {
                                Some(Err(FatalError))
                            }
                        })
                        .collect::<FatalResult<RecordSet<M>>>()
                })
                .transpose()?;
        }

        Ok(res)
    }

    pub(crate) fn best_path(
        &'a self,
        search_pfx: PrefixId<AF>,
        guard: &Guard,
    ) -> Option<Result<Record<M>, PrefixStoreError>> {
        self.prefix_cht
            .non_recursive_retrieve_prefix(search_pfx)
            .0
            .map(|p_rec| {
                p_rec.get_path_selections(guard).best().map_or_else(
                    || Err(PrefixStoreError::BestPathNotFound),
                    |mui| {
                        p_rec
                            .record_map
                            .get_record_for_mui(mui, false)
                            .ok_or(PrefixStoreError::StoreNotReadyError)
                    },
                )
            })
    }

    pub(crate) fn calculate_and_store_best_and_backup_path(
        &self,
        search_pfx: PrefixId<AF>,
        tbi: &<M as Meta>::TBI,
        guard: &Guard,
    ) -> Result<(Option<u32>, Option<u32>), PrefixStoreError> {
        self.prefix_cht
            .non_recursive_retrieve_prefix(search_pfx)
            .0
            .map_or(Err(PrefixStoreError::StoreNotReadyError), |p_rec| {
                p_rec.calculate_and_store_best_backup(tbi, guard)
            })
    }

    pub(crate) fn is_ps_outdated(
        &self,
        search_pfx: PrefixId<AF>,
        guard: &Guard,
    ) -> Result<bool, PrefixStoreError> {
        self.prefix_cht
            .non_recursive_retrieve_prefix(search_pfx)
            .0
            .map_or(Err(PrefixStoreError::StoreNotReadyError), |p| {
                Ok(p.is_ps_outdated(guard))
            })
    }
}

#[derive(Debug)]
pub(crate) struct TreeQueryResult<AF: AddressFamily> {
    pub match_type: MatchType,
    pub prefix: Option<PrefixId<AF>>,
    pub less_specifics: Option<Vec<PrefixId<AF>>>,
    pub more_specifics: Option<Vec<PrefixId<AF>>>,
}

impl<AF: AddressFamily, M: Meta> From<TreeQueryResult<AF>>
    for QueryResult<M>
{
    fn from(value: TreeQueryResult<AF>) -> Self {
        Self {
            match_type: value.match_type,
            prefix: value.prefix.map(|p| p.into()),
            records: vec![],
            less_specifics: value
                .less_specifics
                .map(|ls| ls.into_iter().map(|p| (p, vec![])).collect()),
            more_specifics: value
                .more_specifics
                .map(|ms| ms.into_iter().map(|p| (p, vec![])).collect()),
        }
    }
}

impl<AF: AddressFamily, M: Meta> From<TreeQueryResult<AF>>
    for FamilyQueryResult<AF, M>
{
    fn from(value: TreeQueryResult<AF>) -> Self {
        Self {
            match_type: value.match_type,
            prefix: value.prefix,
            prefix_meta: vec![],
            less_specifics: None,
            more_specifics: None,
        }
    }
}

pub(crate) type FamilyRecord<AF, M> = Vec<(PrefixId<AF>, Vec<Record<M>>)>;

pub(crate) struct FamilyQueryResult<AF: AddressFamily, M: Meta> {
    pub match_type: MatchType,
    pub prefix: Option<PrefixId<AF>>,
    pub prefix_meta: Vec<Record<M>>,
    pub less_specifics: Option<FamilyRecord<AF, M>>,
    pub more_specifics: Option<FamilyRecord<AF, M>>,
}

impl<AF: AddressFamily, M: Meta> From<FamilyQueryResult<AF, M>>
    for QueryResult<M>
{
    fn from(value: FamilyQueryResult<AF, M>) -> Self {
        QueryResult {
            match_type: value.match_type,
            prefix: value.prefix.map(|p| p.into()),
            records: value.prefix_meta,
            less_specifics: value
                .less_specifics
                .map(|ls| ls.into_iter().collect()),
            more_specifics: value
                .more_specifics
                .map(|ms| ms.into_iter().collect()),
        }
    }
}
