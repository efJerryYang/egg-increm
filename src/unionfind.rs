use crate::Id;
use std::fmt::Debug;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct UnionFind {
    parents: Vec<Id>,
    deprecated_leaders: Vec<Id>,
}

impl UnionFind {
    pub fn make_set(&mut self) -> Id {
        let id = Id::from(self.parents.len());
        self.parents.push(id);
        id
    }

    pub fn size(&self) -> usize {
        self.parents.len()
    }

    fn parent(&self, query: Id) -> Id {
        self.parents[usize::from(query)]
    }

    fn parent_mut(&mut self, query: Id) -> &mut Id {
        &mut self.parents[usize::from(query)]
    }

    pub fn find(&self, mut current: Id) -> Id {
        let original = current;
        if self.deprecated_leaders.contains(&current) {
            panic!("Trying to find a deprecated leader <{}> in the eclass <{}>", current, original);
        }
        while current != self.parent(current) {
            if self.deprecated_leaders.contains(&current) {
                panic!("Deprecated leader <{}> found in the eclass <{}>", current, original);
            }
            current = self.parent(current)
        }
        current
    }

    pub fn find_mut(&mut self, mut current: Id) -> Id {
        let original = current;
        if self.deprecated_leaders.contains(&current) {
            panic!("Trying to find a deprecated leader <{}> in the eclass <{}>", current, original);
        }
        while current != self.parent(current) {
            if self.deprecated_leaders.contains(&current) {
                panic!("Deprecated leader <{}> found in the eclass <{}>", current, original);
            }
            let grandparent = self.parent(self.parent(current));
            *self.parent_mut(current) = grandparent;
            current = grandparent;
        }
        current
    }

    /// Given two leader ids, unions the two eclasses making root1 the leader.
    pub fn union(&mut self, root1: Id, root2: Id) -> Id {
        *self.parent_mut(root2) = root1;
        root1
    }

    /// Given the deprecated leader id and the new clusters, updates the parents.
    pub fn split(&mut self, leader_id: Id, clusters: Vec<(Id, Vec<Id>)>) {
        if !self.deprecated_leaders.contains(&leader_id) {
            self.deprecated_leaders.push(leader_id);
        }
        for (new_leader, members) in clusters {
            for member in members {
                *self.parent_mut(member) = new_leader;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(us: impl IntoIterator<Item = usize>) -> Vec<Id> {
        us.into_iter().map(|u| u.into()).collect()
    }

    #[test]
    fn union_find() {
        let n = 10;
        let id = Id::from;

        let mut uf = UnionFind::default();
        for _ in 0..n {
            uf.make_set();
        }

        // test the initial condition of everyone in their own set
        assert_eq!(uf.parents, ids(0..n));

        // build up one set
        uf.union(id(0), id(1));
        uf.union(id(0), id(2));
        uf.union(id(0), id(3));

        // build up another set
        uf.union(id(6), id(7));
        uf.union(id(6), id(8));
        uf.union(id(6), id(9));

        // this should compress all paths
        for i in 0..n {
            uf.find_mut(id(i));
        }

        // indexes:         0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        let expected = vec![0, 0, 0, 0, 4, 5, 6, 6, 6, 6];
        assert_eq!(uf.parents, ids(expected));
    }
    #[test]
    #[should_panic(expected = "Trying to find a deprecated leader <0> in the eclass <0>")]
    fn find_deprecated_leader() {
        let mut uf = UnionFind::default();
        let id1 = uf.make_set();
        let id2 = uf.make_set();
        let id3 = uf.make_set();
        assert_eq!(Id::from(0), id1);
        assert_eq!(Id::from(1), id2);
        assert_eq!(Id::from(2), id3);
        uf.union(id1, id2);
        uf.union(id1, id3);
        
        let id4 = uf.make_set(); // new cluster
        let id5 = uf.make_set(); // new cluster

        uf.split(id1, vec![(id4, vec![id2]), (id5, vec![id3])]);
        uf.find(id1);
    }

    #[test]
    #[should_panic(expected = "Trying to find a deprecated leader <0> in the eclass <0>")]
    fn find_mut_deprecated_leader() {
        let mut uf = UnionFind::default();
        let id1 = uf.make_set();
        let id2 = uf.make_set();
        let id3 = uf.make_set();
        uf.union(id1, id2);
        uf.union(id1, id3);

        let id4 = uf.make_set(); // new cluster
        let id5 = uf.make_set(); // new cluster

        uf.split(id1, vec![(id4, vec![id2]), (id5, vec![id3])]);

        uf.find_mut(id1);
    }

    #[test]
    fn split_union_find() {
        let mut uf = UnionFind::default();
        // 
        // original cluster:
        //  <id1>  -> {<id1>, <id2>, <id3>}
        //  <id4>  -> {<id4>, <id5>}
        //  <id6>  -> {<id6>, <id7>, <id8>, <id9>, <id10>}
        // 
        // split the original cluster into:
        // 
        //  <id11> -> {<id2>}
        //  <id12> -> {<id3>}
        // 
        //  <id4>  -> {<id4>, <id5>}
        // 
        //  <id13> -> {<id7>}
        //  <id14> -> {<id8>}
        //  <id15> -> {<id9>, <id10>}
        // 
        let id1 = uf.make_set();
        let id2 = uf.make_set();
        let id3 = uf.make_set();
        let id4 = uf.make_set();
        let id5 = uf.make_set();
        let id6 = uf.make_set();
        let id7 = uf.make_set();
        let id8 = uf.make_set();
        let id9 = uf.make_set();
        let id10 = uf.make_set();
        // parents: [id1, id2, id3, id4, id5, id6, id7, id8, id9, id10]
        uf.union(id1, id2);
        uf.union(id1, id3);
        uf.union(id4, id5);
        uf.union(id6, id7);
        uf.union(id6, id8);
        uf.union(id6, id9);
        uf.union(id6, id10);
        // parents: [id1, id1, id1, id4, id4, id6, id6, id6, id6, id6]

        let id11 = uf.make_set();
        let id12 = uf.make_set();
        let id13 = uf.make_set();
        let id14 = uf.make_set();
        let id15 = uf.make_set();
        // parents: [id1, id1, id1, id4, id4, id6, id6, id6, id6, id6, id11, id12, id13, id14, id15]

        uf.split(id1, vec![(id11, vec![id2]), (id12, vec![id3])]);
        // parents: [id1, id11, id12, id4, id4, id6, id6, id6, id6, id6, id11, id12, id13, id14, id15]
        uf.split(id6, vec![(id13, vec![id7]), (id14, vec![id8]), (id15, vec![id9, id10])]);
        // parents: [id1, id11, id12, id4, id4, id6, id13, id14, id15, id15, id11, id12, id13, id14, id15]
        
        // test the split clusters
        // id1 is a deprecated leader
        assert_eq!(uf.find(id2), id11);
        assert_eq!(uf.find(id3), id12);
        assert_eq!(uf.find(id4), id4);
        assert_eq!(uf.find(id5), id4);
        // id6 is a deprecated leader
        assert_eq!(uf.find(id7), id13);
        assert_eq!(uf.find(id8), id14);
        assert_eq!(uf.find(id9), id15);
        assert_eq!(uf.find(id10), id15);

        let expected_parents = vec![id1, id11, id12, id4, id4, id6, id13, id14, id15, id15, id11, id12, id13, id14, id15];
        assert_eq!(uf.parents, expected_parents);
        let expected_deprecated_leaders = vec![id1, id6];
        assert_eq!(uf.deprecated_leaders, expected_deprecated_leaders);
    }
}
