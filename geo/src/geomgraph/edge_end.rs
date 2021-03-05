use super::{Coordinate, Edge, Float, Label, Node, Quadrant};

use serde::export::Formatter;
use std::cell::RefCell;

// JTS: import org.locationtech.jts.algorithm.BoundaryNodeRule;
// JTS: import org.locationtech.jts.algorithm.Orientation;
// JTS: import org.locationtech.jts.geom.Coordinate;
// JTS: import org.locationtech.jts.util.Assert;
// JTS:
// JTS: /**
// JTS:  * Models the end of an edge incident on a node.
// JTS:  * EdgeEnds have a direction
// JTS:  * determined by the direction of the ray from the initial
// JTS:  * point to the next point.
// JTS:  * EdgeEnds are comparable under the ordering
// JTS:  * "a has a greater angle with the x-axis than b".
// JTS:  * This ordering is used to sort EdgeEnds around a node.
// JTS:  * @version 1.7
// JTS:  */
// JTS: public class EdgeEnd
// JTS:   implements Comparable
#[derive(Clone)]
pub(crate) struct EdgeEnd<F>
where
    F: Float,
{
    // edge: RefCell<Edge<F>>,
    label: Label,
    coord_0: Coordinate<F>,
    coord_1: Coordinate<F>,
    delta: Coordinate<F>,
    quadrant: Quadrant,
    node: *const Node<F>,
}

impl<F: Float> std::fmt::Debug for EdgeEnd<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdgeEnd")
            .field("label", &self.label)
            .field(
                "coords",
                &format!("{:?} -> {:?}", &self.coord_0, &self.coord_1),
            )
            .field("node", &self.node)
            .field("quadrant", &self.quadrant)
            .finish()
    }
}

impl<F> EdgeEnd<F>
where
    F: Float,
{
    // JTS: {
    // JTS:   protected Edge edge;  // the parent edge of this edge end
    // JTS:   protected Label label;
    // JTS:
    // JTS:   private Node node;          // the node this edge end originates at
    // JTS:   private Coordinate p0, p1;  // points of initial line segment
    // JTS:   private double dx, dy;      // the direction vector for this edge from its starting point
    // JTS:   private int quadrant;
    // JTS:
    // JTS:   protected EdgeEnd(Edge edge)
    // JTS:   {
    // JTS:     this.edge = edge;
    // JTS:   }
    // JTS:   public EdgeEnd(Edge edge, Coordinate p0, Coordinate p1) {
    // JTS:     this(edge, p0, p1, null);
    // JTS:   }

    // JTS:   public EdgeEnd(Edge edge, Coordinate p0, Coordinate p1, Label label) {
    // JTS:     this(edge);
    // JTS:     init(p0, p1);
    // JTS:     this.label = label;
    // JTS:   }
    pub fn new(
        //edge: RefCell<Edge<F>>,
        coord_0: Coordinate<F>,
        coord_1: Coordinate<F>,
        label: Label,
    ) -> EdgeEnd<F> {
        let delta = coord_1 - coord_0;
        let quadrant = Quadrant::new(delta.x, delta.y);
        EdgeEnd {
            // edge,
            label,
            coord_0,
            coord_1,
            delta,
            quadrant,
            node: std::ptr::null(),
        }
    }

    // JTS:   protected void init(Coordinate p0, Coordinate p1)
    // JTS:   {
    // JTS:     this.p0 = p0;
    // JTS:     this.p1 = p1;
    // JTS:     dx = p1.x - p0.x;
    // JTS:     dy = p1.y - p0.y;
    // JTS:     quadrant = Quadrant.quadrant(dx, dy);
    // JTS:     Assert.isTrue(! (dx == 0 && dy == 0), "EdgeEnd with identical endpoints found");
    // JTS:   }
    // JTS:
    // JTS:   public Edge getEdge() { return edge; }
    // JTS:   public Label getLabel() { return label; }
    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    // JTS:   public Coordinate getCoordinate() { return p0; }
    pub fn coordinate(&self) -> &Coordinate<F> {
        &self.coord_0
    }

    // JTS:   public Coordinate getDirectedCoordinate() { return p1; }
    pub fn directed_coordinate(&self) -> &Coordinate<F> {
        &self.coord_1
    }

    // JTS:   public int getQuadrant() { return quadrant; }
    // JTS:   public double getDx() { return dx; }
    // JTS:   public double getDy() { return dy; }
    // JTS:
    // JTS:   public void setNode(Node node) { this.node = node; }
    // JTS:   public Node getNode() { return node; }
    pub fn set_node(&mut self, node: &Node<F>) {
        self.node = &*node;
    }
}

impl<F> std::cmp::Eq for EdgeEnd<F> where F: Float {}

impl<F> std::cmp::PartialEq for EdgeEnd<F>
where
    F: Float,
{
    fn eq(&self, other: &EdgeEnd<F>) -> bool {
        self.delta == other.delta
    }
}

impl<F> std::cmp::PartialOrd for EdgeEnd<F>
where
    F: Float,
{
    fn partial_cmp(&self, other: &EdgeEnd<F>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<F> std::cmp::Ord for EdgeEnd<F>
where
    F: Float,
{
    // JTS:   public int compareTo(Object obj)
    // JTS:   {
    // JTS:       EdgeEnd e = (EdgeEnd) obj;
    // JTS:       return compareDirection(e);
    // JTS:   }
    fn cmp(&self, other: &EdgeEnd<F>) -> std::cmp::Ordering {
        self.compare_direction(other)
    }
}

impl<F> EdgeEnd<F>
where
    F: Float,
{
    // JTS:   /**
    // JTS:    * Implements the total order relation:
    // JTS:    * <p>
    // JTS:    *    a has a greater angle with the positive x-axis than b
    // JTS:    * <p>
    // JTS:    * Using the obvious algorithm of simply computing the angle is not robust,
    // JTS:    * since the angle calculation is obviously susceptible to roundoff.
    // JTS:    * A robust algorithm is:
    // JTS:    * - first compare the quadrant.  If the quadrants
    // JTS:    * are different, it it trivial to determine which vector is "greater".
    // JTS:    * - if the vectors lie in the same quadrant, the computeOrientation function
    // JTS:    * can be used to decide the relative orientation of the vectors.
    // JTS:    */
    // JTS:   public int compareDirection(EdgeEnd e)
    // JTS:   {
    // JTS:     if (dx == e.dx && dy == e.dy)
    // JTS:       return 0;
    // JTS:     // if the rays are in different quadrants, determining the ordering is trivial
    // JTS:     if (quadrant > e.quadrant) return 1;
    // JTS:     if (quadrant < e.quadrant) return -1;
    // JTS:     // vectors are in the same quadrant - check relative orientation of direction vectors
    // JTS:     // this is > e if it is CCW of e
    // JTS:     return Orientation.index(e.p0, e.p1, p1);
    // JTS:   }
    pub(crate) fn compare_direction(&self, other: &EdgeEnd<F>) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if self.delta == other.delta {
            Ordering::Equal
        } else if (self.quadrant as usize) > (other.quadrant as usize) {
            Ordering::Greater
        } else if (self.quadrant as usize) < (other.quadrant as usize) {
            Ordering::Less
        } else {
            use crate::algorithm::kernels::{Kernel, Orientation, RobustKernel};
            // REVIEW:
            match RobustKernel::orient2d(other.coord_0, other.coord_1, self.coord_1) {
                Orientation::Clockwise => Ordering::Less,
                Orientation::CounterClockwise => Ordering::Greater,
                Orientation::Collinear => Ordering::Equal,
            }
        }
    }

    // JTS:   public void computeLabel(BoundaryNodeRule boundaryNodeRule)
    // JTS:   {
    // JTS:     // subclasses should override this if they are using labels
    // JTS:   }
    // JTS:   public void print(PrintStream out)
    // JTS:   {
    // JTS:     double angle = Math.atan2(dy, dx);
    // JTS:     String className = getClass().getName();
    // JTS:     int lastDotPos = className.lastIndexOf('.');
    // JTS:     String name = className.substring(lastDotPos + 1);
    // JTS:     out.print("  " + name + ": " + p0 + " - " + p1 + " " + quadrant + ":" + angle + "   " + label);
    // JTS:   }
    // JTS:   public String toString()
    // JTS:   {
    // JTS:     double angle = Math.atan2(dy, dx);
    // JTS:     String className = getClass().getName();
    // JTS:     int lastDotPos = className.lastIndexOf('.');
    // JTS:     String name = className.substring(lastDotPos + 1);
    // JTS:     return "  " + name + ": " + p0 + " - " + p1 + " " + quadrant + ":" + angle + "   " + label;
    // JTS:   }
    // JTS: }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ord() {
        let fake_label = Label::new_with_geom_on_location(0, None);
        let edge_end_1 = EdgeEnd::new(
            Coordinate::zero(),
            Coordinate { x: 1.0, y: 1.0 },
            fake_label.clone(),
        );
        let edge_end_2 = EdgeEnd::new(
            Coordinate::zero(),
            Coordinate { x: 1.0, y: 1.0 },
            fake_label.clone(),
        );
        assert_eq!(edge_end_1.cmp(&edge_end_2), std::cmp::Ordering::Equal);

        // edge_end_3 is clockwise from edge_end_1
        let edge_end_3 = EdgeEnd::new(
            Coordinate::zero(),
            Coordinate { x: 1.0, y: -1.0 },
            fake_label.clone(),
        );
        assert_eq!(edge_end_1.cmp(&edge_end_3), std::cmp::Ordering::Less);
        assert_eq!(edge_end_3.cmp(&edge_end_1), std::cmp::Ordering::Greater);
    }
}
