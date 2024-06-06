import matplotlib.pyplot as plt
import shapely.geometry as geom
import geopandas as gpd
import pandas as pd
import traceback
from copy import deepcopy


# Each child has a dict containing its corresponding data
class ChildNode:
    def __init__(self, data: dict):
        self.data = data

    def __str__(self):
        return str(self.data)

    # Return a standardized version of the current node
    #   The new version has two keys, 'geometry' and 'ancestors'
    def standardize(self):

        child = deepcopy(self)
        new_child = ChildNode({})

        # Store geometry in new child and remove it from self
        new_child.data["geometry"] = deepcopy(child.data["geometry"])
        child.data.pop("geometry", None)

        # Make child an ancestor of new child
        new_child.data["ancestors"] = [child.data]

        return new_child

    # Debug Function; Return a string of the current Node
    def print(self) -> str:
        out = "Child: " + str(hex(id(self))) + " " + str(self)

        return out

    # Debug Function; Plots the contents of the geometry key
    def plot(self, ax=None):
        x, y = self.data["geometry"].exterior.xy

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

    def add_child(self, child: dict):
        self.children.append(child)

    # Check if the center point of a polygon is within the current quad
    def contains(self, node: dict):
        center = node.data["geometry"].centroid

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
            [self.topLeft[0] + (self.xsize / 2), self.topLeft[1] - (self.ysize / 2)],
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
                    if quad.contains(child):
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

                # Remove a child from the list
                merge_child = children.pop(0)

                # Normalize the new childs geometry
                merge_child.data["geometry"].normalize()

                for other in children:
                    # Normalize the other childs geometry
                    other.data["geometry"].normalize()

                    # Check if the two polygons are equal within tolerance
                    if merge_child.data["geometry"].equals_exact(
                        other.data["geometry"], tolerance=tolerance
                    ):

                        # Remove the other node that we are merging
                        children.remove(other)

                        # Get x, y coords for the vertices of both polygons
                        x_other, y_other = other.data["geometry"].exterior.xy
                        x_child, y_child = merge_child.data["geometry"].exterior.xy

                        # Average the vertexes of the contained polygons
                        i = 0
                        merge_coords = []
                        while i < len(x_child):
                            x_merge = (x_child[i] + x_other[i]) / 2
                            y_merge = (y_child[i] + y_other[i]) / 2
                            merge_coords.append([x_merge, y_merge])
                            i += 1

                        # Create a copy of the original node
                        original = deepcopy(merge_child)

                        # Remove geometry from ancestor nodes
                        original.data.pop("geometry", None)
                        other.data.pop("geometry", None)

                        # Add ancestors to new node
                        merge_child.data["ancestors"].append(original.data)
                        merge_child.data["ancestors"].append(other.data)

                        # Store every key other than ancestors
                        rem = []
                        for key in merge_child.data:
                            if key != "ancestors" and key != "geometry":
                                rem.append(key)

                        # Remove all stored keys
                        for key in rem:
                            merge_child.data.pop(key)

                        # Create the geometry key
                        merge_child.data["geometry"] = geom.Polygon(merge_coords)

                # Standardize children who do not merge
                if merge_child.data["ancestors"] == []:
                    merge_child = merge_child.standardize()

                new_children.append(merge_child)

            # Update the Quadrants children
            self.children = new_children

        # Standardize children of nodes with only 1 child
        elif len(self.children) == 1:
            new_child = self.children[0].standardize()
            self.children = [new_child]

    # Export the results of the quad tree to a GeoDataFrame
    def to_gdf(self, crs) -> gpd.GeoDataFrame:
        gdf = gpd.GeoDataFrame(columns=["geometry", "ancestors"], crs=crs)
        if any(isinstance(child, QuadTree) for child in self.children):
            for child in self.children:
                gdf = pd.concat([gdf, child.to_gdf(crs)], ignore_index=True)
        else:
            for child in self.children:
                gdf.loc[len(gdf.index)] = [
                    child.data["geometry"],
                    child.data["ancestors"],
                ]

        return gdf

    # Debug Function; Graph the parent quad and all of its children
    def plot(self, ax=None):
        if any(isinstance(child, QuadTree) for child in self.children):
            for child in self.children:
                try:
                    child.plot(ax=ax)
                except Exception:
                    print(traceback.format_exc())
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
                if len(youngster.data["ancestors"]) > 2:
                    child_string += youngster.print() + "\n"
            if child_string == "":
                child_string = "None"
            else:
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
