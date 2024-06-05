import matplotlib.pyplot as plt
import shapely.geometry as geom
import traceback


# Each child has a polygon and a count for how many images it represents
class ChildNode:
    def __init__(self, poly: geom.Polygon, ancestors: list):
        self.poly = poly
        self.ancestors = ancestors

    def print(self) -> str:
        out = (
            "Child: "
            + str(self.poly)
            + "\tAncestors: "
            + str(len(self.ancestors))
            + "\n"
            + str(self.ancestors)
        )
        return out

    def plot(self, ax=None):
        x, y = self.poly.exterior.xy

        if isinstance(ax, plt.Axes):
            ax.plot(x, y)
        else:
            plt.plot(x, y)


# Quad Tree data strucutre that handles shapely polygons
class QuadTree:
    def __init__(self, topL: list, xsize, ysize, children: list):
        self.topLeft = topL
        self.xsize = xsize
        self.ysize = ysize
        self.children = children

    def add_child(self, child: geom.Polygon):
        self.children.append(child)

    # Check if the center point of a polygon is within the current quad
    def contains(self, poly: geom.Polygon):
        center = poly.centroid

        # Checks if the polygon is outside of the current quadrant
        if (
            (self.topLeft[0] > center.x)
            or ((self.topLeft[0] + self.xsize) < center.x)
            or (self.topLeft[1] < center.y)
            or ((self.topLeft[1] - self.ysize) > center.y)
        ):
            return False

        return True

    # Creates four new quads
    def quarter(self):

        topLeft = QuadTree(self.topLeft, self.xsize / 2, self.ysize / 2, [])

        topMid = QuadTree(
            [self.topLeft[0] + (self.xsize / 2), self.topLeft[1]],
            self.xsize / 2,
            self.ysize / 2,
            [],
        )

        center = QuadTree(
            [
                self.topLeft[0] + (self.xsize / 2),
                self.topLeft[1] - (self.ysize / 2),
            ],  # noqa: 501
            self.xsize / 2,
            self.ysize / 2,
            [],
        )

        leftMid = QuadTree(
            [self.topLeft[0], self.topLeft[1] - (self.ysize / 2)],
            self.xsize / 2,
            self.ysize / 2,
            [],
        )

        return [topLeft, topMid, center, leftMid]

    # Recursivley splits the QuadTree until there is one child per quadrant
    #   or the mimimum size is reached at which point it will merge polygons
    #   if there vertexes are equal within tolerance
    def split(self, tolerance):

        # Recursivley split data until each quadrant contains one child
        #   or has reached minimum size
        if self.ysize > 0.1 and self.xsize > 0.1 and len(self.children) > 1:
            quads = self.quarter()

            # Categorize current children
            for child in self.children:
                for quad in quads:
                    if quad.contains(child.poly):
                        quad.add_child(child)

            # Store a reference to the contained quads
            self.children = quads

            # Split each child quadrant
            for child in self.children:
                child.split(tolerance)

        # Handle merging polygons if necessary
        elif len(self.children) > 1:

            # Copy children so we can modify the data structure
            #   without effecting the QuadTree Node
            children = self.children.copy()
            new_children = []

            while 0 < len(children):

                # Remove a child from the list and create a new child node
                merge_child = ChildNode(children.pop(0).poly.normalize(), [])

                for other in children:
                    # Check if the two polygons are equal within tolerance
                    if merge_child.poly.equals_exact(
                        other.poly.normalize(), tolerance=tolerance
                    ):

                        # Add combined polygons to the new polygons ancestory
                        merge_child.ancestors.append(merge_child.poly)
                        merge_child.ancestors.append(other.poly)

                        # Remove the other node that we are merging
                        children.remove(other)

                        # Get x, y coords for the vertices of both polygons
                        x_other, y_other = other.poly.normalize().exterior.xy
                        x_child, y_child = merge_child.poly.exterior.xy

                        # Average the vertexes of the contained polygons
                        i = 0
                        merge_coords = []
                        while i < len(x_child):
                            x_merge = (x_child[i] + x_other[i]) / 2
                            y_merge = (y_child[i] + y_other[i]) / 2
                            merge_coords.append([x_merge, y_merge])
                            i += 1

                        # Update the merged child based on the new vertexes
                        merge_child.poly = geom.Polygon(merge_coords)

                new_children.append(merge_child)

            # Update the Quadrants children
            self.children = new_children

    # Graph the parent quad and all of its children
    def plot(self, ax=None):
        if any(isinstance(child, QuadTree) for child in self.children):
            for child in self.children:
                try:
                    child.plot(ax=ax)
                except Exception:
                    traceback.print_exception()
        else:
            # Graph the boundarys of the QuadTree
            x = [self.topLeft[0]]
            y = [self.topLeft[1]]

            x.append(self.topLeft[0] + self.xsize)
            y.append(self.topLeft[1])

            x.append(self.topLeft[0] + self.xsize)
            y.append(self.topLeft[1] - self.ysize)

            x.append(self.topLeft[0])
            y.append(self.topLeft[1] - self.ysize)

            x.append(x[0])
            y.append(y[0])

            if isinstance(ax, plt.Axes):
                ax.plot(x, y, color="black")
            else:
                plt.plot(x, y, color="black")

            # Graph the children of the QuadTree
            for child in self.children:
                child.plot(ax=ax)

    # Debug Function; Print the parent quad and all of its children
    def print(self):
        if any(isinstance(child, QuadTree) for child in self.children):
            for child in self.children:
                child.print()
        else:
            child_string = ""
            for youngster in self.children:
                child_string += youngster.print() + "\t"
            if child_string == "":
                child_string = "None"
            print(
                "Coords: ("
                + str(self.topLeft[0])
                + ", "
                + str(self.topLeft[1])
                + ")\tX Size: "
                + str(self.xsize)
                + "\tChildren: "
                + child_string
                + "\n"
            )

    # Debug Function; Count total number of children a QuadTree contains
    def count_children(self):
        sum = 0

        if any(isinstance(child, QuadTree) for child in self.children):
            for child in self.children:
                sum += child.count_children()
        else:
            sum = len(self.children)

        return sum
