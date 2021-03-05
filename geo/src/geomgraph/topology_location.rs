// JTS: import org.locationtech.jts.geom.Location;

use super::{Location, Position};

// JTS:
// JTS: /**
// JTS:   * A TopologyLocation is the labelling of a
// JTS:   * GraphComponent's topological relationship to a single Geometry.
// JTS:   * <p>
// JTS:   * If the parent component is an area edge, each side and the edge itself
// JTS:   * have a topological location.  These locations are named
// JTS:   * <ul>
// JTS:   * <li> ON: on the edge
// JTS:   * <li> LEFT: left-hand side of the edge
// JTS:   * <li> RIGHT: right-hand side
// JTS:   * </ul>
// JTS:   * If the parent component is a line edge or node, there is a single
// JTS:   * topological relationship attribute, ON.
// JTS:   * <p>
// JTS:   * The possible values of a topological location are
// JTS:   * {Location.NONE, Location.EXTERIOR, Location.BOUNDARY, Location.INTERIOR}
// JTS:   * <p>
// JTS:   * The labelling is stored in an array location[j] where
// JTS:   * where j has the values ON, LEFT, RIGHT
// JTS:   * @version 1.7
// JTS:  */
// JTS: public class TopologyLocation {

#[derive(Clone)]
pub struct TopologyLocation {
    // CLEANUP: can we make this non-optional?
    location: Vec<Option<Location>>,
}

impl TopologyLocation {
    // JTS:
    // JTS:   int location[];
    // JTS:
    // JTS:   public TopologyLocation(int[] location)
    // JTS:   {
    // JTS:     init(location.length);
    // JTS:   }
    // JTS:   /**
    // JTS:    * Constructs a TopologyLocation specifying how points on, to the left of, and to the
    // JTS:    * right of some GraphComponent relate to some Geometry. Possible values for the
    // JTS:    * parameters are Location.NULL, Location.EXTERIOR, Location.BOUNDARY,
    // JTS:    * and Location.INTERIOR.
    // JTS:    * @see Location
    // JTS:    */
    // JTS:   public TopologyLocation(int on, int left, int right) {
    // JTS:    init(3);
    // JTS:    location[Position.ON] = on;
    // JTS:    location[Position.LEFT] = left;
    // JTS:    location[Position.RIGHT] = right;
    // JTS:   }
    // JTS:
    // JTS:   public TopologyLocation(int on) {
    // JTS:    init(1);
    // JTS:    location[Position.ON] = on;
    // JTS:   }
    // JTS:   public TopologyLocation(TopologyLocation gl) {
    // JTS:     init(gl.location.length);
    // JTS:     if (gl != null) {
    // JTS:       for (int i = 0; i < location.length; i++) {
    // JTS:         location[i] = gl.location[i];
    // JTS:       }
    // JTS:     }
    // JTS:   }
    // JTS:   private void init(int size)
    // JTS:   {
    // JTS:     location = new int[size];
    // JTS:     setAllLocations(Location.NONE);
    // JTS:   }
    // JTS:   public int get(int posIndex)
    // JTS:   {
    // JTS:     if (posIndex < location.length) return location[posIndex];
    // JTS:     return Location.NONE;
    // JTS:   }
    pub fn get(&self, pos: Position) -> Option<Location> {
        if (pos as usize) < self.location.len() {
            return self.location[pos as usize];
        } else {
            return None;
        }
    }

    // JTS:   /**
    // JTS:    * @return true if all locations are NULL
    // JTS:    */
    // JTS:   public boolean isNull()
    // JTS:   {
    // JTS:     for (int i = 0; i < location.length; i++) {
    // JTS:       if (location[i] != Location.NONE) return false;
    // JTS:     }
    // JTS:     return true;
    // JTS:   }
    // JTS:   /**
    // JTS:    * @return true if any locations are NULL
    // JTS:    */
    // JTS:   public boolean isAnyNull()
    // JTS:   {
    // JTS:     for (int i = 0; i < location.length; i++) {
    // JTS:       if (location[i] == Location.NONE) return true;
    // JTS:     }
    // JTS:     return false;
    // JTS:   }
    // JTS:   public boolean isEqualOnSide(TopologyLocation le, int locIndex)
    // JTS:   {
    // JTS:     return location[locIndex] == le.location[locIndex];
    // JTS:   }
    // JTS:   public boolean isArea() { return location.length > 1; }
    // JTS:   public boolean isLine() { return location.length == 1; }
    // JTS:
    // JTS:   public void flip()
    // JTS:   {
    // JTS:     if (location.length <= 1) return;
    // JTS:     int temp = location[Position.LEFT];
    // JTS:     location[Position.LEFT] = location[Position.RIGHT];
    // JTS:     location[Position.RIGHT] = temp;
    // JTS:   }
    // JTS:
    // JTS:
    // JTS:   public void setAllLocations(int locValue)
    // JTS:   {
    // JTS:     for (int i = 0; i < location.length; i++) {
    // JTS:       location[i]     = locValue;
    // JTS:     }
    // JTS:   }
    // JTS:   public void setAllLocationsIfNull(int locValue)
    // JTS:   {
    // JTS:     for (int i = 0; i < location.length; i++) {
    // JTS:       if (location[i] == Location.NONE) location[i]     = locValue;
    // JTS:     }
    // JTS:   }
    // JTS:
    // JTS:   public void setLocation(int locIndex, int locValue)
    // JTS:   {
    // JTS:       location[locIndex] = locValue;
    // JTS:   }
    pub fn set_location(&mut self, position: Position, location: Location) {
        self.location[position as usize] = Some(location);
    }

    // JTS:   public void setLocation(int locValue)
    // JTS:   {
    // JTS:     setLocation(Position.ON, locValue);
    // JTS:   }
    // JTS:   public int[] getLocations() { return location; }
    // JTS:   public void setLocations(int on, int left, int right) {
    // JTS:       location[Position.ON] = on;
    // JTS:       location[Position.LEFT] = left;
    // JTS:       location[Position.RIGHT] = right;
    // JTS:   }
    // JTS:   public boolean allPositionsEqual(int loc)
    // JTS:   {
    // JTS:     for (int i = 0; i < location.length; i++) {
    // JTS:       if (location[i] != loc) return false;
    // JTS:     }
    // JTS:     return true;
    // JTS:   }
    // JTS:
    // JTS:   /**
    // JTS:    * merge updates only the NULL attributes of this object
    // JTS:    * with the attributes of another.
    // JTS:    */
    // JTS:   public void merge(TopologyLocation gl)
    // JTS:   {
    // JTS:     // if the src is an Area label & and the dest is not, increase the dest to be an Area
    // JTS:     if (gl.location.length > location.length) {
    // JTS:       int [] newLoc = new int[3];
    // JTS:       newLoc[Position.ON] = location[Position.ON];
    // JTS:       newLoc[Position.LEFT] = Location.NONE;
    // JTS:       newLoc[Position.RIGHT] = Location.NONE;
    // JTS:       location = newLoc;
    // JTS:     }
    // JTS:     for (int i = 0; i < location.length; i++) {
    // JTS:       if (location[i] == Location.NONE && i < gl.location.length)
    // JTS:         location[i] = gl.location[i];
    // JTS:     }
    // JTS:   }
    // JTS:
    // JTS:   public String toString()
    // JTS:   {
    // JTS:     StringBuffer buf = new StringBuffer();
    // JTS:     if (location.length > 1) buf.append(Location.toLocationSymbol(location[Position.LEFT]));
    // JTS:     buf.append(Location.toLocationSymbol(location[Position.ON]));
    // JTS:     if (location.length > 1) buf.append(Location.toLocationSymbol(location[Position.RIGHT]));
    // JTS:     return buf.toString();
    // JTS:   }
    // JTS: }
}
