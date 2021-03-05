use super::{EdgeEndBuilder, IntersectionMatrix, RelateNodeFactory};
use crate::algorithm::dimensions::{Dimensions, HasDimensions};
use crate::geomgraph::{
    algorithm::{PointLocator, RobustLineIntersector},
    index::SegmentIntersector,
    EdgeEnd, Float, GeometryGraph, GraphComponent, Location, Node, NodeMap,
};

use geo_types::Geometry;

// JTS: /**
// JTS:  * @version 1.7
// JTS:  */
// JTS: import java.util.ArrayList;
// JTS: import java.util.Iterator;
// JTS: import java.util.List;
// JTS:
// JTS: import org.locationtech.jts.algorithm.LineIntersector;
// JTS: import org.locationtech.jts.algorithm.PointLocator;
// JTS: import org.locationtech.jts.algorithm.RobustLineIntersector;
// JTS: import org.locationtech.jts.geom.Coordinate;
// JTS: import org.locationtech.jts.geom.Geometry;
// JTS: import org.locationtech.jts.geom.IntersectionMatrix;
// JTS: import org.locationtech.jts.geom.Location;
// JTS: import org.locationtech.jts.geomgraph.Edge;
// JTS: import org.locationtech.jts.geomgraph.EdgeEnd;
// JTS: import org.locationtech.jts.geomgraph.EdgeIntersection;
// JTS: import org.locationtech.jts.geomgraph.GeometryGraph;
// JTS: import org.locationtech.jts.geomgraph.Label;
// JTS: import org.locationtech.jts.geomgraph.Node;
// JTS: import org.locationtech.jts.geomgraph.NodeMap;
// JTS: import org.locationtech.jts.geomgraph.index.SegmentIntersector;
// JTS: import org.locationtech.jts.util.Assert;
// JTS:
// JTS: /**
// JTS:  * Computes the topological relationship between two Geometries.
// JTS:  * <p>
// JTS:  * RelateComputer does not need to build a complete graph structure to compute
// JTS:  * the IntersectionMatrix.  The relationship between the geometries can
// JTS:  * be computed by simply examining the labelling of edges incident on each node.
// JTS:  * <p>
// JTS:  * RelateComputer does not currently support arbitrary GeometryCollections.
// JTS:  * This is because GeometryCollections can contain overlapping Polygons.
// JTS:  * In order to correct compute relate on overlapping Polygons, they
// JTS:  * would first need to be noded and merged (if not explicitly, at least
// JTS:  * implicitly).
// JTS:  *
// JTS:  * @version 1.7
// JTS:  */
// JTS: public class RelateComputer
// JTS: {
pub(crate) struct RelateComputer<'a, F>
where
    F: Float,
{
    graph_a: GeometryGraph<'a, F>,
    graph_b: GeometryGraph<'a, F>,
    nodes: NodeMap<F, RelateNodeFactory>,
    line_intersector: RobustLineIntersector<F>,
    point_locator: PointLocator<F>,
}

// CLEANUP: remove this.
pub fn _test_relate_computer() {
    use crate::{line_string, polygon};
    let square_a: Geometry<f64> = polygon![].into();
    let square_b: Geometry<f64> = polygon![].into();
    let mut relate_computer = RelateComputer::new(&square_a, &square_b);
    let _intersection_matrix = relate_computer.compute_intersection_matrix();
}

impl<'a, F> RelateComputer<'a, F>
where
    F: 'static + Float,
{
    pub fn new(geom_a: &'a Geometry<F>, geom_b: &'a Geometry<F>) -> RelateComputer<'a, F> {
        Self {
            graph_a: GeometryGraph::new(0, geom_a),
            graph_b: GeometryGraph::new(1, geom_b),
            nodes: NodeMap::new(),
            line_intersector: RobustLineIntersector::new(),
            point_locator: PointLocator::new(),
        }
    }

    // JTS:   private LineIntersector li = new RobustLineIntersector();
    // JTS:   private PointLocator ptLocator = new PointLocator();
    // JTS:   private GeometryGraph[] arg;  // the arg(s) of the operation
    // JTS:   private NodeMap nodes = new NodeMap(new RelateNodeFactory());
    // JTS:   // this intersection matrix will hold the results compute for the relate
    // JTS:   private IntersectionMatrix im = null;
    // JTS:   private ArrayList isolatedEdges = new ArrayList();
    // JTS:
    // JTS:   // the intersection point found (if any)
    // JTS:   private Coordinate invalidPoint;
    // JTS:
    // JTS:   public RelateComputer(GeometryGraph[] arg) {
    // JTS:     this.arg = arg;
    // JTS:   }
    // JTS:
    // JTS:   public IntersectionMatrix computeIM()
    // JTS:   {
    pub fn compute_intersection_matrix(&mut self) -> IntersectionMatrix {
        // JTS:     IntersectionMatrix im = new IntersectionMatrix();
        let mut intersection_matrix = IntersectionMatrix::empty();
        // JTS:     // since Geometries are finite and embedded in a 2-D space, the EE element must always be 2
        // JTS:     im.set(Location.EXTERIOR, Location.EXTERIOR, 2);
        // since Geometries are finite and embedded in a 2-D space, the EE element must always be 2
        intersection_matrix.set(
            Location::Exterior,
            Location::Exterior,
            Dimensions::TwoDimensional,
        );

        // JTS:     // if the Geometries don't overlap there is nothing to do
        // JTS:     if (! arg[0].getGeometry().getEnvelopeInternal().intersects(
        // JTS:             arg[1].getGeometry().getEnvelopeInternal()) ) {
        // JTS:       computeDisjointIM(im);
        // JTS:       return im;
        // JTS:     }

        // if the Geometries don't overlap, we can skip most of the work
        use crate::algorithm::bounding_rect::BoundingRect;
        match (
            self.graph_a.geometry().bounding_rect(),
            self.graph_b.geometry().bounding_rect(),
        ) {
            (Some(bounding_rect_a), Some(bounding_rect_b)) => {
                use crate::algorithm::intersects::Intersects;
                if !bounding_rect_a.intersects(&bounding_rect_b) {
                    self.compute_disjoint_intersection_matrix(&mut intersection_matrix);
                    return intersection_matrix;
                }
            }
            _ => {
                self.compute_disjoint_intersection_matrix(&mut intersection_matrix);
                return intersection_matrix;
            }
        }

        // JTS:     arg[0].computeSelfNodes(li, false);
        // JTS:     arg[1].computeSelfNodes(li, false);
        // REVIEW: In JTS, second `false` is implied via a default arg from an overload
        // REVIEW: In JTS, self.line_intersection is just passed in. But it's mutated - seems like
        //         a bad idea and rust won't allow it. So we clone.
        self.graph_a
            .compute_self_nodes(Box::new(self.line_intersector.clone()), false, false);
        self.graph_b
            .compute_self_nodes(Box::new(self.line_intersector.clone()), false, false);

        // JTS:     // compute intersections between edges of the two input geometries
        // JTS:     SegmentIntersector intersector = arg[0].computeEdgeIntersections(arg[1], li, false);
        // compute intersections between edges of the two input geometries
        let segment_intersector = self.graph_a.compute_edge_intersections(
            &self.graph_b,
            Box::new(self.line_intersector.clone()),
            false,
        );

        // JTS: //System.out.println("computeIM: # segment intersection tests: " + intersector.numTests);
        // JTS:     computeIntersectionNodes(0);
        // JTS:     computeIntersectionNodes(1);
        self.compute_intersection_nodes(0);
        self.compute_intersection_nodes(1);
        // JTS:     /**
        // JTS:      * Copy the labelling for the nodes in the parent Geometries.  These override
        // JTS:      * any labels determined by intersections between the geometries.
        // JTS:      */
        // JTS:     copyNodesAndLabels(0);
        // JTS:     copyNodesAndLabels(1);
        // Copy the labelling for the nodes in the parent Geometries.  These override any labels
        // determined by intersections between the geometries.
        self.copy_nodes_and_labels(0);
        self.copy_nodes_and_labels(1);
        // JTS:
        // JTS:     // complete the labelling for any nodes which only have a label for a single geometry
        // JTS: //Debug.addWatch(nodes.find(new Coordinate(110, 200)));
        // JTS: //Debug.printWatch();
        // JTS:     labelIsolatedNodes();
        // complete the labelling for any nodes which only have a label for a single geometry
        self.label_isolated_nodes();
        // JTS: //Debug.printWatch();
        // JTS:
        // JTS:     // If a proper intersection was found, we can set a lower bound on the IM.
        // JTS:     computeProperIntersectionIM(intersector, im);
        // If a proper intersection was found, we can set a lower bound on the IM.
        self.compute_proper_intersection_im(&segment_intersector, &mut intersection_matrix);
        // JTS:
        // JTS:     /**
        // JTS:      * Now process improper intersections
        // JTS:      * (eg where one or other of the geometries has a vertex at the intersection point)
        // JTS:      * We need to compute the edge graph at all nodes to determine the IM.
        // JTS:      */
        // JTS:
        // JTS:     // build EdgeEnds for all intersections
        // JTS:     EdgeEndBuilder eeBuilder = new EdgeEndBuilder();
        // JTS:     List ee0 = eeBuilder.computeEdgeEnds(arg[0].getEdgeIterator());
        // JTS:     insertEdgeEnds(ee0);
        // JTS:     List ee1 = eeBuilder.computeEdgeEnds(arg[1].getEdgeIterator());
        // JTS:     insertEdgeEnds(ee1);
        // Now process improper intersections
        // (eg where one or other of the geometries has a vertex at the intersection point)
        // We need to compute the edge graph at all nodes to determine the IM.
        let edge_end_builder = EdgeEndBuilder::new();
        let edge_ends_a: Vec<_> = edge_end_builder.compute_ends_for_edges(self.graph_a.edges());
        self.insert_edge_ends(edge_ends_a);
        let edge_ends_b: Vec<_> = edge_end_builder.compute_ends_for_edges(self.graph_b.edges());
        self.insert_edge_ends(edge_ends_b);
        // JTS:
        // JTS: //Debug.println("==== NodeList ===");
        // JTS: //Debug.print(nodes);
        // JTS:
        // JTS:     labelNodeEdges();
        self.label_node_edges();
        todo!();
        // JTS:
        // JTS:   /**
        // JTS:    * Compute the labeling for isolated components
        // JTS:    * <br>
        // JTS:    * Isolated components are components that do not touch any other components in the graph.
        // JTS:    * They can be identified by the fact that they will
        // JTS:    * contain labels containing ONLY a single element, the one for their parent geometry.
        // JTS:    * We only need to check components contained in the input graphs, since
        // JTS:    * isolated components will not have been replaced by new components formed by intersections.
        // JTS:    */
        // JTS: //debugPrintln("Graph A isolated edges - ");
        // JTS:     labelIsolatedEdges(0, 1);
        // JTS: //debugPrintln("Graph B isolated edges - ");
        // JTS:     labelIsolatedEdges(1, 0);
        // JTS:
        // JTS:     // update the IM from all components
        // JTS:     updateIM(im);
        // JTS:     return im;
        // JTS:   }
        intersection_matrix
    }

    // JTS:   private void insertEdgeEnds(List ee)
    // JTS:   {
    fn insert_edge_ends(&mut self, edge_ends: Vec<EdgeEnd<F>>) {
        // JTS:     for (Iterator i = ee.iterator(); i.hasNext(); ) {
        // JTS:       EdgeEnd e = (EdgeEnd) i.next();
        // JTS:       nodes.add(e);
        // JTS:     }
        // JTS:   }
        for edge_end in edge_ends {
            self.nodes.add_edge_end(edge_end)
        }
    }

    // JTS:   private void computeProperIntersectionIM(SegmentIntersector intersector, IntersectionMatrix im)
    // JTS:   {
    fn compute_proper_intersection_im(
        &mut self,
        segment_intersector: &SegmentIntersector<F>,
        intersection_matrix: &mut IntersectionMatrix,
    ) {
        // JTS:     // If a proper intersection is found, we can set a lower bound on the IM.
        // JTS:     int dimA = arg[0].getGeometry().getDimension();
        // JTS:     int dimB = arg[1].getGeometry().getDimension();
        // If a proper intersection is found, we can set a lower bound on the IM.
        let dim_a = self.graph_a.geometry().dimensions();
        let dim_b = self.graph_b.geometry().dimensions();

        // JTS:     boolean hasProper         = intersector.hasProperIntersection();
        // JTS:     boolean hasProperInterior = intersector.hasProperInteriorIntersection();
        let has_proper = segment_intersector.has_proper_intersection();
        let has_proper_interior = segment_intersector.has_proper_interior_intersection();

        // JTS:       // For Geometry's of dim 0 there can never be proper intersections.
        debug_assert!(
            (dim_a != Dimensions::ZeroDimensional && dim_b != Dimensions::ZeroDimensional)
                || (!has_proper && !has_proper_interior)
        );

        match (dim_a, dim_b) {
            // JTS:       /**
            // JTS:        * If edge segments of Areas properly intersect, the areas must properly overlap.
            // JTS:        */
            // JTS:     if (dimA == 2 && dimB == 2) {
            // If edge segments of Areas properly intersect, the areas must properly overlap.
            (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                // JTS:       if (hasProper) im.setAtLeast("212101212");
                if has_proper {
                    intersection_matrix.set_at_least("212101212");
                }
                // JTS:     }
            }

            // JTS:       /**
            // JTS:        * If an Line segment properly intersects an edge segment of an Area,
            // JTS:        * it follows that the Interior of the Line intersects the Boundary of the Area.
            // JTS:        * If the intersection is a proper <i>interior</i> intersection, then
            // JTS:        * there is an Interior-Interior intersection too.
            // JTS:        * Note that it does not follow that the Interior of the Line intersects the Exterior
            // JTS:        * of the Area, since there may be another Area component which contains the rest of the Line.
            // JTS:        */
            // JTS:     else if (dimA == 2 && dimB == 1) {
            // If a Line segment properly intersects an edge segment of an Area, it follows that
            // the Interior of the Line intersects the Boundary of the Area.
            // If the intersection is a proper *interior* intersection, then there is an
            // Interior-Interior intersection too.
            // Note that it does not follow that the Interior of the Line intersects the Exterior
            // of the Area, since there may be another Area component which contains the rest of the Line.
            (Dimensions::TwoDimensional, Dimensions::OneDimensional) => {
                // JTS:       if (hasProper)          im.setAtLeast("FFF0FFFF2");
                if has_proper {
                    intersection_matrix.set_at_least("FFF0FFFF2");
                }

                // JTS:       if (hasProperInterior)  im.setAtLeast("1FFFFF1FF");
                if has_proper_interior {
                    intersection_matrix.set_at_least("1FFFFF1FF");
                }

                // JTS:     }
            }

            // JTS:     else if (dimA == 1 && dimB == 2) {
            (Dimensions::OneDimensional, Dimensions::TwoDimensional) => {
                // JTS:       if (hasProper)          im.setAtLeast("F0FFFFFF2");
                if has_proper {
                    intersection_matrix.set_at_least("F0FFFFFF2");
                }

                // JTS:       if (hasProperInterior)  im.setAtLeast("1F1FFFFFF");
                if has_proper_interior {
                    intersection_matrix.set_at_least("1F1FFFFFF");
                }

                // JTS:     }
            }

            // JTS:     /* If edges of LineStrings properly intersect *in an interior point*, all
            // JTS:         we can deduce is that
            // JTS:         the interiors intersect.  (We can NOT deduce that the exteriors intersect,
            // JTS:         since some other segments in the geometries might cover the points in the
            // JTS:         neighbourhood of the intersection.)
            // JTS:         It is important that the point be known to be an interior point of
            // JTS:         both Geometries, since it is possible in a self-intersecting geometry to
            // JTS:         have a proper intersection on one segment that is also a boundary point of another segment.
            // JTS:     */
            // JTS:     else if (dimA == 1 && dimB == 1) {
            // If edges of LineStrings properly intersect *in an interior point*, all we can deduce
            // is that the interiors intersect.  (We can NOT deduce that the exteriors intersect,
            // since some other segments in the geometries might cover the points in the
            // neighbourhood of the intersection.)
            // It is important that the point be known to be an interior point of both Geometries,
            // since it is possible in a self-intersecting geometry to have a proper intersection
            // on one segment that is also a boundary point of another segment.
            (Dimensions::OneDimensional, Dimensions::OneDimensional) => {
                // JTS:       if (hasProperInterior)    im.setAtLeast("0FFFFFFFF");
                // JTS:     }
                if has_proper_interior {
                    intersection_matrix.set_at_least("0FFFFFFFF");
                }
            }
            // JTS:   }
            _ => {}
        }
    }

    // JTS:     /**
    // JTS:      * Copy all nodes from an arg geometry into this graph.
    // JTS:      * The node label in the arg geometry overrides any previously computed
    // JTS:      * label for that argIndex.
    // JTS:      * (E.g. a node may be an intersection node with
    // JTS:      * a computed label of BOUNDARY,
    // JTS:      * but in the original arg Geometry it is actually
    // JTS:      * in the interior due to the Boundary Determination Rule)
    // JTS:      */
    // JTS:   private void copyNodesAndLabels(int argIndex)
    // JTS:   {
    // JTS:     for (Iterator i = arg[argIndex].getNodeIterator(); i.hasNext(); ) {
    // JTS:       Node graphNode = (Node) i.next();
    // JTS:       Node newNode = nodes.addNode(graphNode.getCoordinate());
    // JTS:       newNode.setLabel(argIndex, graphNode.getLabel().getLocation(argIndex));
    // JTS: //node.print(System.out);
    // JTS:     }
    // JTS:   }

    /// Copy all nodes from an arg geometry into this graph.
    ///
    /// The node label in the arg geometry overrides any previously computed label for that
    /// argIndex.  (E.g. a node may be an intersection node with a computed label of BOUNDARY, but
    /// in the original arg Geometry it is actually in the interior due to the Boundary
    /// Determination Rule)
    fn copy_nodes_and_labels(&mut self, geom_index: usize) {
        let graph = if geom_index == 0 {
            &self.graph_a
        } else {
            assert!(geom_index == 1);
            &self.graph_b
        };
        for graph_node in graph.node_iter() {
            let new_node = self
                .nodes
                .add_node_with_coordinate(*graph_node.coordinate());
            // CLEANUP: graph_node.labe().unwrap - label is never None for Nodes
            // CLEANUP: on_location().unwrap - can we get rid of it or check for it?
            new_node.set_label_on_location(
                geom_index,
                graph_node.label().unwrap().on_location(geom_index).unwrap(),
            );
        }
    }

    // JTS:   /**
    // JTS:    * Insert nodes for all intersections on the edges of a Geometry.
    // JTS:    * Label the created nodes the same as the edge label if they do not already have a label.
    // JTS:    * This allows nodes created by either self-intersections or
    // JTS:    * mutual intersections to be labelled.
    // JTS:    * Endpoint nodes will already be labelled from when they were inserted.
    // JTS:    */
    // JTS:   private void computeIntersectionNodes(int argIndex)
    // JTS:   {

    /// Insert nodes for all intersections on the edges of a Geometry.  
    ///
    /// Label the created nodes the same as the edge label if they do not already have a label.
    /// This allows nodes created by either self-intersections or mutual intersections to be
    /// labelled.  
    ///
    /// Endpoint nodes will already be labeled from when they were inserted.
    fn compute_intersection_nodes(&mut self, geom_index: usize) {
        let graph = if geom_index == 0 {
            &self.graph_a
        } else {
            assert!(geom_index == 1);
            &self.graph_b
        };

        // JTS:     for (Iterator i = arg[argIndex].getEdgeIterator(); i.hasNext(); ) {
        // JTS:       Edge e = (Edge) i.next();
        for edge in graph.edges() {
            let edge = edge.borrow();

            // JTS:       int eLoc = e.getLabel().getLocation(argIndex);
            // CLEANUP: `unwrap` - I believe label is *always* `Some` for edges.
            let edge_location = edge.label().unwrap().on_location(geom_index);
            // JTS:       for (Iterator eiIt = e.getEdgeIntersectionList().iterator(); eiIt.hasNext(); ) {
            for edge_intersection in edge.edge_intersections() {
                // JTS:         EdgeIntersection ei = (EdgeIntersection) eiIt.next();
                // JTS:         RelateNode n = (RelateNode) nodes.addNode(ei.coord);
                let new_node = self
                    .nodes
                    .add_node_with_coordinate(edge_intersection.coordinate());

                // JTS:         if (eLoc == Location.BOUNDARY)
                // JTS:           n.setLabelBoundary(argIndex);
                // JTS:         else {
                // JTS:           if (n.getLabel().isNull(argIndex))
                // JTS:             n.setLabel(argIndex, Location.INTERIOR);
                // JTS:         }
                if edge_location == Some(Location::Boundary) {
                    new_node.set_label_boundary(geom_index);
                } else {
                    // CLEANUP: unwrap - label is never null for nodes.
                    if new_node.label().unwrap().is_empty(geom_index) {
                        new_node.set_label_on_location(geom_index, Location::Interior);
                    }
                }
                // JTS: //Debug.println(n);
                // JTS:       }
                // JTS:     }
                // JTS:   }
            }
        }
    }

    // JTS:   /**
    // JTS:    * For all intersections on the edges of a Geometry,
    // JTS:    * label the corresponding node IF it doesn't already have a label.
    // JTS:    * This allows nodes created by either self-intersections or
    // JTS:    * mutual intersections to be labelled.
    // JTS:    * Endpoint nodes will already be labelled from when they were inserted.
    // JTS:    */
    // JTS:   private void labelIntersectionNodes(int argIndex)
    // JTS:   {
    // JTS:     for (Iterator i = arg[argIndex].getEdgeIterator(); i.hasNext(); ) {
    // JTS:       Edge e = (Edge) i.next();
    // JTS:       int eLoc = e.getLabel().getLocation(argIndex);
    // JTS:       for (Iterator eiIt = e.getEdgeIntersectionList().iterator(); eiIt.hasNext(); ) {
    // JTS:         EdgeIntersection ei = (EdgeIntersection) eiIt.next();
    // JTS:         RelateNode n = (RelateNode) nodes.find(ei.coord);
    // JTS:         if (n.getLabel().isNull(argIndex)) {
    // JTS:           if (eLoc == Location.BOUNDARY)
    // JTS:             n.setLabelBoundary(argIndex);
    // JTS:           else
    // JTS:             n.setLabel(argIndex, Location.INTERIOR);
    // JTS:         }
    // JTS: //n.print(System.out);
    // JTS:       }
    // JTS:     }
    // JTS:   }
    // JTS:   /**
    // JTS:    * If the Geometries are disjoint, we need to enter their dimension and
    // JTS:    * boundary dimension in the Ext rows in the IM
    // JTS:    */
    // JTS:   private void computeDisjointIM(IntersectionMatrix im)
    // JTS:   {
    /// If the Geometries are disjoint, we need to enter their dimension and boundary dimension in
    /// the Ext rows in the IM
    fn compute_disjoint_intersection_matrix(&self, intersection_matrix: &mut IntersectionMatrix) {
        // JTS:     Geometry ga = arg[0].getGeometry();
        // JTS:     if (! ga.isEmpty()) {
        // JTS:       im.set(Location.INTERIOR, Location.EXTERIOR, ga.getDimension());
        // JTS:       im.set(Location.BOUNDARY, Location.EXTERIOR, ga.getBoundaryDimension());
        // JTS:     }
        {
            let geometry_a = self.graph_a.geometry();
            let dimensions = geometry_a.dimensions();
            if dimensions != Dimensions::Empty {
                intersection_matrix.set(Location::Interior, Location::Exterior, dimensions);

                let boundary_dimensions = geometry_a.boundary_dimensions();
                if boundary_dimensions != Dimensions::Empty {
                    intersection_matrix.set(
                        Location::Boundary,
                        Location::Exterior,
                        boundary_dimensions,
                    );
                }
            }
        }

        // JTS:     Geometry gb = arg[1].getGeometry();
        // JTS:     if (! gb.isEmpty()) {
        // JTS:       im.set(Location.EXTERIOR, Location.INTERIOR, gb.getDimension());
        // JTS:       im.set(Location.EXTERIOR, Location.BOUNDARY, gb.getBoundaryDimension());
        // JTS:     }
        // JTS:   }
        {
            let geometry_b = self.graph_b.geometry();
            let dimensions = geometry_b.dimensions();
            if dimensions != Dimensions::Empty {
                intersection_matrix.set(Location::Exterior, Location::Interior, dimensions);

                let boundary_dimensions = geometry_b.boundary_dimensions();
                if boundary_dimensions != Dimensions::Empty {
                    intersection_matrix.set(
                        Location::Exterior,
                        Location::Boundary,
                        boundary_dimensions,
                    );
                }
            }
        }
    }

    // JTS:   private void labelNodeEdges()
    // JTS:   {
    // JTS:     for (Iterator ni = nodes.iterator(); ni.hasNext(); ) {
    // JTS:       RelateNode node = (RelateNode) ni.next();
    // JTS:       node.getEdges().computeLabelling(arg);
    // JTS: //Debug.print(node.getEdges());
    // JTS: //node.print(System.out);
    // JTS:     }
    // JTS:   }
    fn label_node_edges(&mut self) {
        for node in self.nodes.iter_mut() {
            node.edges().compute_labeling(&self.graph_a, &self.graph_b);
        }
    }

    // JTS:   /**
    // JTS:    * update the IM with the sum of the IMs for each component
    // JTS:    */
    // JTS:   private void updateIM(IntersectionMatrix im)
    // JTS:   {
    // JTS: //Debug.println(im);
    // JTS:     for (Iterator ei = isolatedEdges.iterator(); ei.hasNext(); ) {
    // JTS:       Edge e = (Edge) ei.next();
    // JTS:       e.updateIM(im);
    // JTS: //Debug.println(im);
    // JTS:     }
    // JTS:     for (Iterator ni = nodes.iterator(); ni.hasNext(); ) {
    // JTS:       RelateNode node = (RelateNode) ni.next();
    // JTS:       node.updateIM(im);
    // JTS: //Debug.println(im);
    // JTS:       node.updateIMFromEdges(im);
    // JTS: //Debug.println(im);
    // JTS: //node.print(System.out);
    // JTS:     }
    // JTS:   }
    // JTS:
    // JTS:   /**
    // JTS:    * Processes isolated edges by computing their labelling and adding them
    // JTS:    * to the isolated edges list.
    // JTS:    * Isolated edges are guaranteed not to touch the boundary of the target (since if they
    // JTS:    * did, they would have caused an intersection to be computed and hence would
    // JTS:    * not be isolated)
    // JTS:    */
    // JTS:   private void labelIsolatedEdges(int thisIndex, int targetIndex)
    // JTS:   {
    // JTS:     for (Iterator ei = arg[thisIndex].getEdgeIterator(); ei.hasNext(); ) {
    // JTS:       Edge e = (Edge) ei.next();
    // JTS:       if (e.isIsolated()) {
    // JTS:         labelIsolatedEdge(e, targetIndex, arg[targetIndex].getGeometry());
    // JTS:         isolatedEdges.add(e);
    // JTS:       }
    // JTS:     }
    // JTS:   }
    // JTS:   /**
    // JTS:    * Label an isolated edge of a graph with its relationship to the target geometry.
    // JTS:    * If the target has dim 2 or 1, the edge can either be in the interior or the exterior.
    // JTS:    * If the target has dim 0, the edge must be in the exterior
    // JTS:    */
    // JTS:   private void labelIsolatedEdge(Edge e, int targetIndex, Geometry target)
    // JTS:   {
    // JTS:     // this won't work for GeometryCollections with both dim 2 and 1 geoms
    // JTS:     if ( target.getDimension() > 0) {
    // JTS:     // since edge is not in boundary, may not need the full generality of PointLocator?
    // JTS:     // Possibly should use ptInArea locator instead?  We probably know here
    // JTS:     // that the edge does not touch the bdy of the target Geometry
    // JTS:       int loc = ptLocator.locate(e.getCoordinate(), target);
    // JTS:       e.getLabel().setAllLocations(targetIndex, loc);
    // JTS:     }
    // JTS:     else {
    // JTS:       e.getLabel().setAllLocations(targetIndex, Location.EXTERIOR);
    // JTS:     }
    // JTS: //System.out.println(e.getLabel());
    // JTS:   }
    // JTS:
    // JTS:   /**
    // JTS:    * Isolated nodes are nodes whose labels are incomplete
    // JTS:    * (e.g. the location for one Geometry is null).
    // JTS:    * This is the case because nodes in one graph which don't intersect
    // JTS:    * nodes in the other are not completely labelled by the initial process
    // JTS:    * of adding nodes to the nodeList.
    // JTS:    * To complete the labelling we need to check for nodes that lie in the
    // JTS:    * interior of edges, and in the interior of areas.
    // JTS:    */
    // JTS:   private void labelIsolatedNodes()
    // JTS:   {
    // JTS:     for (Iterator ni = nodes.iterator(); ni.hasNext(); ) {
    // JTS:       Node n = (Node) ni.next();
    // JTS:       Label label = n.getLabel();
    // JTS:       // isolated nodes should always have at least one geometry in their label
    // JTS:       Assert.isTrue(label.getGeometryCount() > 0, "node with empty label found");
    /// Isolated nodes are nodes whose labels are incomplete (e.g. the location for one Geometry is
    /// null).  
    /// This is the case because nodes in one graph which don't intersect nodes in the other
    /// are not completely labelled by the initial process of adding nodes to the nodeList.  To
    /// complete the labelling we need to check for nodes that lie in the interior of edges, and in
    /// the interior of areas.
    fn label_isolated_nodes(&mut self) {
        let geometry_a = self.graph_a.geometry();
        let geometry_b = self.graph_b.geometry();
        for node in self.nodes.iter_mut() {
            // CLEANUP: remove unwrap?
            let label = node.label().unwrap();
            // isolated nodes should always have at least one geometry in their label
            debug_assert!(label.geometry_count() > 0, "node with empty label found");
            // JTS:       if (n.isIsolated()) {
            // JTS:         if (label.isNull(0))
            // JTS:           labelIsolatedNode(n, 0);
            // JTS:         else
            // JTS:           labelIsolatedNode(n, 1);
            // JTS:       }
            if node.is_isolated() {
                if label.is_empty(0) {
                    Self::label_isolated_node(&mut self.point_locator, node, 0, geometry_a)
                } else {
                    Self::label_isolated_node(&mut self.point_locator, node, 1, geometry_b)
                }
            }
            // JTS:     }
            // JTS:   }
        }
    }

    // JTS:   /**
    // JTS:    * Label an isolated node with its relationship to the target geometry.
    // JTS:    */
    // JTS:   private void labelIsolatedNode(Node n, int targetIndex)
    // JTS:   {
    fn label_isolated_node(
        point_locator: &mut PointLocator<F>,
        node: &mut Node<F>,
        target_index: usize,
        geometry: &Geometry<F>,
    ) {
        // JTS:     int loc = ptLocator.locate(n.getCoordinate(), arg[targetIndex].getGeometry());
        let location = point_locator.locate(node.coordinate(), geometry);
        // JTS:     n.getLabel().setAllLocations(targetIndex, loc);
        // JTS: //debugPrintln(n.getLabel());
        // JTS:   }
        // JTS: }
        // CLEANUP: unwrap
        node.label_mut()
            .unwrap()
            .set_all_locations(target_index, location);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use geo_types::{polygon, Geometry, GeometryCollection};

    #[test]
    fn test_disjoint() {
        let square_a: Geometry<f64> = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_b: Geometry<f64> = polygon![
            (x: 55., y: 55.),
            (x: 50., y: 60.),
            (x: 60., y: 60.),
            (x: 60., y: 55.),
            (x: 55., y: 55.),
        ]
        .into();

        let mut relate_computer = RelateComputer::new(&square_a, &square_b);
        let intersection_matrix = relate_computer.compute_intersection_matrix();
        let expected = IntersectionMatrix::new([
            [
                Dimensions::Empty,
                Dimensions::Empty,
                Dimensions::TwoDimensional,
            ],
            [
                Dimensions::Empty,
                Dimensions::Empty,
                Dimensions::OneDimensional,
            ],
            [
                Dimensions::TwoDimensional,
                Dimensions::OneDimensional,
                Dimensions::TwoDimensional,
            ],
        ]);
        assert_eq!(intersection_matrix, expected);
    }

    #[test]
    fn test_a_contains_b() {
        let square_a: Geometry<f64> = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_b: Geometry<f64> = polygon![
            (x: 5., y: 5.),
            (x: 5., y: 10.),
            (x: 10., y: 10.),
            (x: 10., y: 5.),
            (x: 5., y: 5.),
        ]
        .into();

        let mut relate_computer = RelateComputer::new(&square_a, &square_b);
        let intersection_matrix = relate_computer.compute_intersection_matrix();
        // TODO: expected is incorrect
        let expected = IntersectionMatrix::new([
            [Dimensions::Empty, Dimensions::Empty, Dimensions::Empty],
            [Dimensions::Empty, Dimensions::Empty, Dimensions::Empty],
            [
                Dimensions::Empty,
                Dimensions::Empty,
                Dimensions::TwoDimensional,
            ],
        ]);
        assert_eq!(intersection_matrix, expected);
    }

    #[test]
    fn test_a_overlaps_b() {
        let square_a: Geometry<f64> = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_b: Geometry<f64> = polygon![
            (x: 5., y: 5.),
            (x: 5., y: 30.),
            (x: 30., y: 30.),
            (x: 30., y: 5.),
            (x: 5., y: 5.),
        ]
        .into();

        let mut relate_computer = RelateComputer::new(&square_a, &square_b);
        let intersection_matrix = relate_computer.compute_intersection_matrix();
        // TODO: expected is incorrect
        let expected = IntersectionMatrix::new([
            [Dimensions::Empty, Dimensions::Empty, Dimensions::Empty],
            [Dimensions::Empty, Dimensions::Empty, Dimensions::Empty],
            [
                Dimensions::Empty,
                Dimensions::Empty,
                Dimensions::TwoDimensional,
            ],
        ]);
        assert_eq!(intersection_matrix, expected);
    }

    #[test]
    fn test_a_overlaps_itself() {
        let square_a_1: Geometry<f64> = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_a_2: Geometry<f64> = polygon![
            (x: 5., y: 5.),
            (x: 5., y: 30.),
            (x: 30., y: 30.),
            (x: 30., y: 5.),
            (x: 5., y: 5.),
        ]
        .into();

        let collection_a: Geometry<f64> =
            Geometry::GeometryCollection(GeometryCollection(vec![square_a_1, square_a_2]));

        let square_b: Geometry<f64> = polygon![
            (x: 5., y: 5.),
            (x: 5., y: 30.),
            (x: 30., y: 30.),
            (x: 30., y: 5.),
            (x: 5., y: 5.),
        ]
        .into();

        let mut relate_computer = RelateComputer::new(&collection_a, &square_b);
        let intersection_matrix = relate_computer.compute_intersection_matrix();
        // TODO: expected is incorrect
        let expected = IntersectionMatrix::new([
            [Dimensions::Empty, Dimensions::Empty, Dimensions::Empty],
            [Dimensions::Empty, Dimensions::Empty, Dimensions::Empty],
            [
                Dimensions::Empty,
                Dimensions::Empty,
                Dimensions::TwoDimensional,
            ],
        ]);
        assert_eq!(intersection_matrix, expected);
    }
}
