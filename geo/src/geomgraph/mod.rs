#![allow(dead_code)]
#![allow(unused_imports)]

mod edge;
mod edge_end;
mod edge_end_star;
mod edge_intersection;
mod edge_intersection_list;
mod geometry_graph;
mod graph_component;
pub(crate) mod index;
mod label;
mod node;
mod node_factory;
mod node_map;
mod planar_graph;
mod quadrant;
mod topology_location;

pub(crate) use edge::Edge;
pub(crate) use edge_end::EdgeEnd;
pub(crate) use edge_end_star::EdgeEndStar;
pub(crate) use edge_intersection::EdgeIntersection;
pub(crate) use edge_intersection_list::EdgeIntersectionList;
pub(crate) use geometry_graph::GeometryGraph;
pub(crate) use graph_component::GraphComponent;
pub(crate) use label::Label;
pub(crate) use node::Node;
pub(crate) use node_factory::{BasicNodeFactory, NodeFactory};
pub(crate) use node_map::NodeMap;
use planar_graph::PlanarGraph;
pub(crate) use quadrant::Quadrant;
use topology_location::TopologyLocation;

// in just algorithm is an external pacakage (top level, still part of JTS) - not witin geomgraph
pub(crate) mod algorithm;

use geo_types::{Coordinate, Geometry};

pub(crate) trait Float:
    'static + num_traits::Float + crate::algorithm::kernels::HasKernel
{
}
impl<F: 'static + num_traits::Float + crate::algorithm::kernels::HasKernel> Float for F {}

// CLEANUP: use geo::kernels::Orientation instead?
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Position {
    // CLEANUP: get rid of the explicit discrimanator?
    On = 0,
    Left = 1,
    Right = 2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Location {
    // CLEANUP: get rid of the explicit discrimanator?
    Interior = 0,
    Boundary = 1,
    Exterior = 2,
}

// CLEANUP: get rid of Location and use geo::utils::CoordPos everywhere directly
use crate::utils::CoordPos;
impl From<CoordPos> for Location {
    fn from(coord_pos: CoordPos) -> Location {
        match coord_pos {
            CoordPos::Inside => Location::Interior,
            CoordPos::Outside => Location::Exterior,
            CoordPos::OnBoundary => Location::Boundary,
        }
    }
}
