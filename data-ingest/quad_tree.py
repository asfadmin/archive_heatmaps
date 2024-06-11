import shapely.geometry as geom
import geopandas as gpd
from copy import deepcopy


# Each child has a dict containing its corresponding data
class ChildNode:
    def __init__(self, data: dict):
        self.data = data

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


# Quad Tree data strucutre that handles shapely polygons
class QuadTree:
    def __init__(self, children: gpd.GeoDataFrame):
        self.children = []

        for row in children.iterrows():
            curr = row[1].to_dict()
            self.children.append(ChildNode(curr))

    def merge(self, tolerance):
        i = 0
        while i < len(self.children) - 1:
            if (
                self.children[i]
                .data["geometry"]
                .normalize()
                .equals_exact(
                    self.children[i + 1].data["geometry"].normalize(),
                    tolerance=tolerance,
                )
            ):

                # Get store nodes we are merging and remove them from children
                first = self.children.pop(i)
                second = self.children.pop(i)

                # Store x,y locations of both nodes geometrys vertexes
                x1, y1 = first.data["geometry"].exterior.xy
                x2, y2 = second.data["geometry"].exterior.xy

                # Remove geometry from merging nodes
                first.data.pop("geometry", None)
                second.data.pop("geometry", None)

                # Average vertex coordinates
                merge_coords = []
                j = 0
                while j < len(x1):
                    x_merge = (x1[j] + x2[j]) / 2
                    y_merge = (y1[j] + y2[j]) / 2
                    merge_coords.append([x_merge, y_merge])
                    j += 1

                # If this is the first merge and the child is not yet standerdized
                if "ancestors" not in first.data.keys():
                    ancestors = [first.data, second.data]

                # If this is the second merge handle pre-existing ancestors
                else:
                    ancestors = []

                    for anc in first.data["ancestors"]:
                        ancestors.append(anc)

                    ancestors.append(second.data)

                merge_dict = {}
                merge_dict["geometry"] = geom.Polygon(merge_coords)
                merge_dict["ancestors"] = ancestors

                merge_child = ChildNode(merge_dict)

                self.children.insert(i, merge_child)

            elif "ancestors" not in self.children[i].data.keys():
                self.children[i] = self.children[i].standardize()
                i += 1
            else:
                i += 1

        if "ancestors" not in self.children[-1].data.keys():
            self.children[-1] = self.children[-1].standardize()

    # Export the results of the quad tree to a dictionary
    def to_dict(self) -> dict:

        dictionary = {"geometry": [], "ancestors": []}

        for child in self.children:
            dictionary["geometry"].append(child.data["geometry"])
            dictionary["ancestors"].append(child.data["ancestors"])

        return dictionary
