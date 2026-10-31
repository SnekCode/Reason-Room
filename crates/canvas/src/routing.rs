//! Camera-independent relationship geometry. Visibility and clipping belong to
//! the caller; neither is an input to the cached route.
use crate::{
    Point, Rect, card_parent_ids, cards_are_in_containment_chain,
    relationship_path_around_obstacles,
};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct RelationshipRouteCache {
    cards: BTreeMap<u32, Rect>,
    boundaries: Vec<Rect>,
    parents: BTreeMap<u32, u32>,
    paths: BTreeMap<(u32, u32), Option<Vec<Point>>>,
}

impl RelationshipRouteCache {
    /// Returns whether layout changed. Call with all placed cards, including
    /// offscreen and semantically hidden records. No camera values are stored.
    pub fn update_layout(&mut self, cards: BTreeMap<u32, Rect>, boundaries: Vec<Rect>) -> bool {
        if self.cards == cards && self.boundaries == boundaries {
            return false;
        }
        self.parents = card_parent_ids(cards.iter().map(|(id, bounds)| (*id, *bounds)));
        self.cards = cards;
        self.boundaries = boundaries;
        self.paths.clear();
        true
    }

    pub fn route(&mut self, source: u32, target: u32) -> Option<&[Point]> {
        if !self.paths.contains_key(&(source, target)) {
            let route = self.calculate(source, target);
            self.paths.insert((source, target), route);
        }
        self.paths[&(source, target)].as_deref()
    }

    fn calculate(&self, source: u32, target: u32) -> Option<Vec<Point>> {
        let source_bounds = *self.cards.get(&source)?;
        let target_bounds = *self.cards.get(&target)?;
        if cards_are_in_containment_chain(&self.parents, source, target) {
            return None;
        }
        let obstacles: Vec<_> = self
            .cards
            .iter()
            .filter_map(|(id, bounds)| {
                // Endpoints, their containers and their own nested content are
                // part of the connection's context, not unrelated obstacles.
                (*id != source
                    && *id != target
                    && !cards_are_in_containment_chain(&self.parents, source, *id)
                    && !cards_are_in_containment_chain(&self.parents, target, *id))
                .then_some(*bounds)
            })
            .chain(self.boundaries.iter().copied().filter(|bounds| {
                !encloses(*bounds, source_bounds) && !encloses(*bounds, target_bounds)
            }))
            .collect();
        // A child inside an unrelated solid obstacle adds no routing constraint.
        // Removing it bounds the visibility graph for deeply nested documents.
        let obstacles: Vec<_> = obstacles
            .iter()
            .copied()
            .filter(|bounds| {
                !obstacles
                    .iter()
                    .any(|other| other != bounds && encloses(*other, *bounds))
            })
            .collect();
        let mut route =
            relationship_path_around_obstacles(source_bounds, target_bounds, &obstacles);
        if route.len() < 2 {
            return None;
        }
        route[0] = port_toward(source_bounds, route[1]);
        let last = route.len() - 1;
        route[last] = port_toward(target_bounds, route[last - 1]);
        Some(route)
    }
}

fn encloses(outer: Rect, inner: Rect) -> bool {
    let outer = outer.normalized();
    let inner = inner.normalized();
    outer.origin.x <= inner.origin.x
        && outer.origin.y <= inner.origin.y
        && outer.max_x() >= inner.max_x()
        && outer.max_y() >= inner.max_y()
}

fn port_toward(bounds: Rect, target: Point) -> Point {
    let bounds = bounds.normalized();
    let center = bounds.center();
    let delta = target - center;
    if delta.x.abs() >= delta.y.abs() {
        Point::new(
            if delta.x >= 0.0 {
                bounds.max_x()
            } else {
                bounds.origin.x
            },
            center.y,
        )
    } else {
        Point::new(
            center.x,
            if delta.y >= 0.0 {
                bounds.max_y()
            } else {
                bounds.origin.y
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bounds(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(x, y), Point::new(w, h))
    }

    #[test]
    fn routes_are_reused_until_layout_changes() {
        let mut cache = RelationshipRouteCache::default();
        let mut cards = BTreeMap::from([
            (1, bounds(0.0, 0.0, 100.0, 60.0)),
            (2, bounds(500.0, 0.0, 100.0, 60.0)),
            (3, bounds(220.0, -40.0, 160.0, 140.0)),
        ]);
        assert!(cache.update_layout(cards.clone(), vec![]));
        let original = cache.route(1, 2).unwrap().to_vec();
        assert!(original.len() > 2);
        let pointer = cache.route(1, 2).unwrap().as_ptr();
        assert!(!cache.update_layout(cards.clone(), vec![]));
        assert_eq!(pointer, cache.route(1, 2).unwrap().as_ptr());
        cards.remove(&3);
        assert!(cache.update_layout(cards.clone(), vec![]));
        assert_eq!(cache.route(1, 2).unwrap().len(), 2);
        assert_ne!(cache.route(1, 2).unwrap(), original);
        // A moved boundary also invalidates an otherwise unchanged layout.
        assert!(cache.update_layout(cards.clone(), vec![bounds(220.0, -40.0, 160.0, 140.0)]));
        assert!(cache.route(1, 2).unwrap().len() > 2);
        cards.get_mut(&2).unwrap().origin.y = 300.0;
        assert!(cache.update_layout(cards, vec![]));
        assert_eq!(cache.route(1, 2).unwrap().last().unwrap().y, 330.0);
    }

    #[test]
    fn nested_context_does_not_block_its_own_endpoint() {
        let mut cache = RelationshipRouteCache::default();
        cache.update_layout(
            BTreeMap::from([
                (1, bounds(0.0, 0.0, 400.0, 300.0)),
                (2, bounds(500.0, 0.0, 400.0, 300.0)),
                (3, bounds(100.0, 100.0, 200.0, 150.0)),
            ]),
            vec![],
        );
        assert_eq!(cache.route(1, 2).unwrap().len(), 2);
        assert!(cache.route(1, 3).is_none());
        assert!(cache.route(1, 99).is_none());
    }
}
