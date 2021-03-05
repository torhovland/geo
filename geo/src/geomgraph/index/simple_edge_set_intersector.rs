use super::super::{Edge, Float};
use super::{EdgeSetIntersector, SegmentIntersector};

use std::cell::RefCell;
use std::rc::Rc;

// JTS: /**
// JTS:  * Finds all intersections in one or two sets of edges,
// JTS:  * using the straightforward method of
// JTS:  * comparing all segments.
// JTS:  * This algorithm is too slow for production use, but is useful for testing purposes.
// JTS:  * @version 1.7
// JTS:  */
pub(crate) struct SimpleEdgeSetIntersector {
    overlap_count: u32,
}

// JTS: public class SimpleEdgeSetIntersector
// JTS:   extends EdgeSetIntersector
// JTS: {
impl<F> EdgeSetIntersector<F> for SimpleEdgeSetIntersector
where
    F: Float,
{
    // JTS:   // statistics information
    // JTS:   int nOverlaps;
    // JTS:
    // JTS:   public SimpleEdgeSetIntersector() {
    // JTS:   }
    // JTS:
    // JTS:   public void computeIntersections(List edges, SegmentIntersector si, boolean testAllSegments)
    // JTS:   {
    fn compute_intersections(
        &mut self,
        edges: &[Rc<RefCell<Edge<F>>>],
        segment_intersector: &mut SegmentIntersector<F>,
        test_all_segments: bool,
    ) {
        // JTS:     nOverlaps = 0;
        self.overlap_count = 0;

        // JTS:     for (Iterator i0 = edges.iterator(); i0.hasNext(); ) {
        // JTS:       Edge edge0 = (Edge) i0.next();
        // JTS:       for (Iterator i1 = edges.iterator(); i1.hasNext(); ) {
        // JTS:         Edge edge1 = (Edge) i1.next();
        // JTS:         if (testAllSegments || edge0 != edge1)
        // JTS:           computeIntersects(edge0, edge1, si);
        // JTS:       }
        // JTS:     }
        for edge0 in edges.iter() {
            for edge1 in edges.iter() {
                // TODO: I expect this to explode if test_all_segments is true, due to the RefCell
                //       being borrowed twice simultaneously
                //
                if test_all_segments || edge0 != edge1 {
                    self.compute_intersects(edge0, edge1, segment_intersector);
                }
            }
        }
        // JTS:   }
    }

    // JTS:   public void computeIntersections(List edges0, List edges1, SegmentIntersector si)
    // JTS:   {
    fn compute_intersections_testing_all_segments(
        &mut self,
        edges0: &[Rc<RefCell<Edge<F>>>],
        edges1: &[Rc<RefCell<Edge<F>>>],
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        // JTS:     nOverlaps = 0;
        self.overlap_count = 0;

        // JTS:     for (Iterator i0 = edges0.iterator(); i0.hasNext(); ) {
        // JTS:       Edge edge0 = (Edge) i0.next();
        // JTS:       for (Iterator i1 = edges1.iterator(); i1.hasNext(); ) {
        // JTS:         Edge edge1 = (Edge) i1.next();
        // JTS:         computeIntersects(edge0, edge1, si);
        // JTS:       }
        // JTS:     }
        for edge0 in edges0 {
            for edge1 in edges1 {
                self.compute_intersects(edge0, edge1, segment_intersector);
            }
        }
        // JTS:   }
    }
}

impl SimpleEdgeSetIntersector {
    pub fn new() -> Self {
        SimpleEdgeSetIntersector { overlap_count: 0 }
    }

    // JTS:
    // JTS:   /**
    // JTS:    * Performs a brute-force comparison of every segment in each Edge.
    // JTS:    * This has n^2 performance, and is about 100 times slower than using
    // JTS:    * monotone chains.
    // JTS:    */
    // JTS:   private void computeIntersects(Edge e0, Edge e1, SegmentIntersector si)
    // JTS:   {
    fn compute_intersects<'a, F: Float>(
        &mut self,
        edge0: &Rc<RefCell<Edge<F>>>,
        edge1: &Rc<RefCell<Edge<F>>>,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        // JTS:    Coordinate[] pts0 = e0.getCoordinates();
        // JTS:     Coordinate[] pts1 = e1.getCoordinates();
        // JTS:     for (int i0 = 0; i0 < pts0.length - 1; i0++) {
        // JTS:       for (int i1 = 0; i1 < pts1.length - 1; i1++) {
        // JTS:         si.addIntersections(e0, i0, e1, i1);
        // JTS:       }
        // JTS:     }
        let edge0_coords_len = edge0.borrow().coords().len() - 1;
        let edge1_coords_len = edge1.borrow().coords().len() - 1;
        for i0 in 0..edge0_coords_len {
            for i1 in 0..edge1_coords_len {
                segment_intersector.add_intersections(edge0, i0, edge1, i1);
            }
        }
        // JTS: }
    }
    // JTS:   }
}
