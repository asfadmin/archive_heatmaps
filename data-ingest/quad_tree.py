import matplotlib.pyplot as plt
import geopandas as gpd
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
    def __init__(self, children: gpd.GeoDataFrame):
        self.children = children

    def merge(self, tolerance):
        count = 0
        i = 0
        while i < self.children.shape[0] - 1:
            if (
                self.children["geometry"]
                .iloc[i]
                .equals_exact(
                    self.children["geometry"].iloc[i + 1], tolerance=tolerance
                )
            ):
                print("merging")
                count += 1
            i += 1

        print("Total Merged: " + str(count))

    # Export the results of the quad tree to a dictionary
    def to_dict(self) -> dict:

        dictionary = {"geometry": [], "ancestors": []}

        # Recurse through children
        if any(isinstance(child, QuadTree) for child in self.children):
            for child in self.children:
                child_dict = child.to_dict()

                # Add child dictionarys to current dict
                for key in dictionary:
                    for ent in child_dict[key]:
                        dictionary[key].append(ent)

        else:
            for child in self.children:
                # Add child data to the dictionary
                dictionary["geometry"].append(child.data["geometry"])
                dictionary["ancestors"].append(child.data["ancestors"])

        return dictionary

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
