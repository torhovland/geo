use super::{Coordinate, EdgeEnd, Float, GraphComponent, Location, Node, NodeFactory};

use std::collections::BTreeMap;
use std::marker::PhantomData;

// JTS: import org.locationtech.jts.geom.Coordinate;
// JTS: import org.locationtech.jts.geom.Location;
// JTS:
// JTS: /**
// JTS:  * A map of nodes, indexed by the coordinate of the node
// JTS:  * @version 1.7
// JTS:  */
// JTS: public class NodeMap
/// A map of nodes, indexed by the coordinate of the node
pub(crate) struct NodeMap<F, NF>
where
    F: Float,
    NF: NodeFactory<F>,
{
    map: BTreeMap<NodeKey<F>, Node<F>>,
    _node_factory: PhantomData<NF>,
}

impl<F, NF> std::fmt::Debug for NodeMap<F, NF>
where
    F: Float,
    NF: NodeFactory<F>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeMap")
            .field("map.len()", &self.map.len())
            .finish()
    }
}

#[derive(Clone)]
struct NodeKey<F: Float>(Coordinate<F>);

impl<F: Float> std::cmp::Ord for NodeKey<F> {
    fn cmp(&self, other: &NodeKey<F>) -> std::cmp::Ordering {
        // TODO: BTree requires Ord - can we guarantee all coords in the graph are non-NaN?
        debug_assert!(!self.0.x.is_nan());
        debug_assert!(!self.0.y.is_nan());
        debug_assert!(!other.0.x.is_nan());
        debug_assert!(!other.0.y.is_nan());

        crate::utils::lex_cmp(&self.0, &other.0)
    }
}

impl<F: Float> std::cmp::PartialOrd for NodeKey<F> {
    fn partial_cmp(&self, other: &NodeKey<F>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: Float> std::cmp::PartialEq for NodeKey<F> {
    fn eq(&self, other: &NodeKey<F>) -> bool {
        // TODO: BTree requires Eq - can we guarantee all coords in the graph are non-NaN?
        debug_assert!(!self.0.x.is_nan());
        debug_assert!(!self.0.y.is_nan());
        debug_assert!(!other.0.x.is_nan());
        debug_assert!(!other.0.y.is_nan());
        return self.0 == other.0;
    }
}

impl<F: Float> std::cmp::Eq for NodeKey<F> {}

impl<F, NF> NodeMap<F, NF>
where
    F: Float,
    NF: NodeFactory<F>,
{
    // JTS: {
    // JTS:   //Map nodeMap = new HashMap();
    // JTS:   Map nodeMap = new TreeMap();
    // JTS:   NodeFactory nodeFact;
    // JTS:
    // JTS:   public NodeMap(NodeFactory nodeFact) {
    // JTS:     this.nodeFact = nodeFact;
    // JTS:   }
    pub fn new() -> Self {
        NodeMap {
            map: BTreeMap::new(),
            _node_factory: PhantomData,
        }
    }
    // JTS:
    // JTS:   /**
    // JTS:    * Factory function - subclasses can override to create their own types of nodes
    // JTS:    */
    // JTS:    /*
    // JTS:   protected Node createNode(Coordinate coord)
    // JTS:   {
    // JTS:     return new Node(coord);
    // JTS:   }
    // JTS:   */
    // JTS:   /**
    // JTS:    * This method expects that a node has a coordinate value.
    // JTS:    */
    // JTS:   public Node addNode(Coordinate coord)
    // JTS:   {
    // JTS:     Node node = (Node) nodeMap.get(coord);
    // JTS:     if (node == null) {
    // JTS:       node = nodeFact.createNode(coord);
    // JTS:       nodeMap.put(coord, node);
    // JTS:     }
    // JTS:     return node;
    // JTS:   }
    pub fn add_node_with_coordinate(&mut self, coord: Coordinate<F>) -> &mut Node<F> {
        let key = NodeKey(coord);
        self.map.entry(key).or_insert(NF::create_node(coord))
    }

    // JTS:   public Node addNode(Node n)
    // JTS:   {
    // JTS:     Node node = (Node) nodeMap.get(n.getCoordinate());
    // JTS:     if (node == null) {
    // JTS:       nodeMap.put(n.getCoordinate(), n);
    // JTS:       return n;
    // JTS:     }
    // JTS:     node.mergeLabel(n);
    // JTS:     return node;
    // JTS:   }

    // JTS:    * Adds a node for the start point of this EdgeEnd
    // JTS:    * (if one does not already exist in this map).
    // JTS:    * Adds the EdgeEnd to the (possibly new) node.
    // JTS:    */
    // JTS:   public void add(EdgeEnd e)
    // JTS:   {
    // JTS:     Coordinate p = e.getCoordinate();
    // JTS:     Node n = addNode(p);
    // JTS:     n.add(e);
    // JTS:   }
    pub fn add_edge_end(&mut self, edge_end: EdgeEnd<F>) {
        let coord = edge_end.coordinate();
        let mut node = self.add_node_with_coordinate(*coord);
        node.add_edge_end(edge_end);
    }

    // JTS:   /**
    // JTS:    * @return the node if found; null otherwise
    // JTS:    */
    // JTS:   public Node find(Coordinate coord)  {    return (Node) nodeMap.get(coord);  }
    /// returns the node if found
    pub fn find(&self, coord: Coordinate<F>) -> Option<&Node<F>> {
        self.map.get(&NodeKey(coord))
    }

    // JTS:   public Iterator iterator()
    // JTS:   {
    // JTS:     return nodeMap.values().iterator();
    // JTS:   }
    pub fn iter(&self) -> impl Iterator<Item = &Node<F>> {
        self.map.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Node<F>> {
        self.map.values_mut()
    }

    // JTS:   public Collection values()
    // JTS:   {
    // JTS:     return nodeMap.values();
    // JTS:   }

    // JTS:   public Collection getBoundaryNodes(int geomIndex)
    // JTS:   {
    // JTS:     Collection bdyNodes = new ArrayList();
    // JTS:     for (Iterator i = iterator(); i.hasNext(); ) {
    // JTS:       Node node = (Node) i.next();
    // JTS:       if (node.getLabel().getLocation(geomIndex) == Location.BOUNDARY)
    // JTS:         bdyNodes.add(node);
    // JTS:     }
    // JTS:     return bdyNodes;
    // JTS:   }
    pub fn boundary_nodes(&self, geom_index: usize) -> Vec<&Node<F>> {
        // CLEANUP: `unwrap` - nodes *always* have a label. edges sometimes do not, but they
        // inherit the same `label` API via GraphComponent. Might be nice to separate them
        // to remove this unwrap
        self.map
            .values()
            .filter(|node| {
                matches!(
                    node.label().unwrap().on_location(geom_index),
                    Some(Location::Boundary)
                )
            })
            .collect()
    }

    // JTS:
    // JTS:   public void print(PrintStream out)
    // JTS:   {
    // JTS:     for (Iterator it = iterator(); it.hasNext(); )
    // JTS:     {
    // JTS:       Node n = (Node) it.next();
    // JTS:       n.print(out);
    // JTS:     }
    // JTS:   }
    // JTS: }
}
