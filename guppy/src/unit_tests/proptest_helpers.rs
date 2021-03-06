// Copyright (c) The cargo-guppy Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::graph::{DependencyDirection, PackageGraph};
use crate::unit_tests::dep_helpers::{assert_depends_on_any, assert_link_order};
use crate::PackageId;
use proptest09::sample::Index;

macro_rules! proptest_suite {
    ($name: ident) => {
        mod $name {
            use crate::graph::DependencyDirection;
            use crate::unit_tests::fixtures::Fixture;
            use crate::unit_tests::proptest_helpers::*;
            use proptest09::collection::vec;
            use proptest09::prelude::*;
            use proptest09::sample::Index;

            #[test]
            fn proptest_select_depends_on() {
                let fixture = Fixture::$name();
                let graph = fixture.graph();

                proptest!(|(
                    ids in vec(graph.prop09_id_strategy(), 1..16),
                    select_direction in any::<DependencyDirection>(),
                    query_direction in any::<DependencyDirection>(),
                    query_indexes in vec(any::<Index>(), 0..16),
                )| {
                    depends_on(graph, &ids, select_direction, query_direction, query_indexes, "select_depends_on");
                });
            }

            #[test]
            fn proptest_select_link_order() {
                let fixture = Fixture::$name();
                let graph = fixture.graph();

                proptest!(|(
                    ids in vec(graph.prop09_id_strategy(), 1..16),
                    select_direction in any::<DependencyDirection>(),
                    query_direction in any::<DependencyDirection>(),
                )| {
                    link_order(graph, &ids, select_direction, query_direction, "select_link_order");
                });
            }
        }
    }
}

/// Test that all results of an into_iter_ids query depend on at least one of the ids in the query
/// set.
pub(crate) fn depends_on(
    graph: &PackageGraph,
    ids: &[&PackageId],
    select_direction: DependencyDirection,
    query_direction: DependencyDirection,
    query_indexes: Vec<Index>,
    msg: &str,
) {
    let msg = format!("{}: reachable means depends on", msg);

    let select = graph
        .select_directed(ids.iter().copied(), select_direction)
        .unwrap();

    let reachable_ids: Vec<_> = select.into_iter_ids(Some(query_direction)).collect();

    let mut cache = graph.new_depends_cache();

    for index in query_indexes {
        let query_id = index.get(&reachable_ids);
        assert_depends_on_any(ids, query_id, &mut cache, select_direction, &msg);
    }
}

/// Test that all results of an into_iter_links query follow link order.
pub(crate) fn link_order(
    graph: &PackageGraph,
    ids: &[&PackageId],
    select_direction: DependencyDirection,
    query_direction: DependencyDirection,
    msg: &str,
) {
    let select = graph
        .select_directed(ids.iter().copied(), select_direction)
        .unwrap();
    // If the select and query directions are the opposite, the set of initial IDs will be
    // different as well. Compute the root IDs from the graph in that case.
    let initials = if select_direction != query_direction {
        select.clone().into_root_ids(query_direction).collect()
    } else {
        ids.to_vec()
    };
    let links = select.into_iter_links(Some(query_direction));
    assert_link_order(
        links,
        initials,
        query_direction,
        &format!("{}: link order", msg),
    );
}
