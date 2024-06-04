import matplotlib.pyplot as plt
import shapely.geometry
import traceback

#Each child has a polygon and a count for how many images it represents
class ChildNode:
    def __init__(self,poly: shapely.geometry.Polygon, count = 1):
        self.poly = poly
        self.count = count
        
    def print(self) -> str:
        out = str(self.poly) + "\t" + str(self.count)
        return out
    
    def plot(self, ax):
        x,y = self.poly.exterior.xy
        
        if isinstance(ax, plt.Axes):
            ax.plot(x,y)
        else:
            plt.plot(x,y)
   
#Quad Tree data strucutre that handles shapely polygons
class QuadTree:
    def __init__(self, topL: list, xsize, ysize, children: list):
        self.topLeft = topL
        self.xsize = xsize
        self.ysize = ysize
        self.children = children
        self.mergable = 0
        
    def add_child(self, child: shapely.geometry.Polygon):
        self.children.append(child)
    
    #Check if the center point of a polygon is within the current quad    
    def contains(self, poly: shapely.geometry.Polygon):
        center = poly.centroid
        # Checks that the polygon is horizontally within the current square
        if (self.topLeft[0] > center.x) or ((self.topLeft[0] + self.xsize) < center.x):
            return False
        
        # Checks that the polygon is vertically within the square
        if (self.topLeft[1] < center.y) or ((self.topLeft[1] - self.ysize) > center.y):
            return False
            
        return True
    
    #Creates four new quads
    def quarter(self):
       
        topLeft = QuadTree(self.topLeft, self.xsize / 2, self.ysize / 2, [])
        
        topMid = QuadTree([self.topLeft[0] + (self.xsize / 2), self.topLeft[1]], self.xsize / 2, self.ysize / 2, [])
        
        center = QuadTree([self.topLeft[0] + (self.xsize / 2), self.topLeft[1] - (self.ysize / 2) ], self.xsize / 2, self.ysize / 2, [])
        
        leftMid = QuadTree([ self.topLeft[0], self.topLeft[1] - (self.ysize / 2) ], self.xsize / 2, self.ysize / 2, [])
        
        return [topLeft, topMid, center, leftMid]
    
    #Recursivley split the current quad until the minimum size is reached or each quad contains only 1 child    
    def split(self, tolerance):
        # Recursivley split data until each quadrant contains one child or has reached minimum size
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
                    child.split(1)
                    
        #Handle merging polygons if necessary          
        elif len(self.children) > 1:
            
            #Copy children so we can modify the data structure without effecting the QuadTree Node
            children_copy = self.children.copy()
            new_children = []
            
            while 0 < len(children_copy):
                
                #Remove a child from the list and create a new child node
                merge_child = ChildNode(children_copy.pop(0).poly.normalize())
                
                for other in children_copy:
                    #Check if the two polygons are equal within a tolerance
                    if merge_child.poly.equals_exact(other.poly.normalize(),tolerance=tolerance):
                        
                        #Remove the other node that we are merging with our first
                        children_copy.remove(other)
                        
                        #Get x, y coordinates for all the vertices of both polygons
                        x_other, y_other = other.poly.normalize().exterior.xy
                        x_child, y_child = merge_child.poly.exterior.xy
                        
                        #Average the vertexes of the contained polygons
                        i = 0
                        merge_coords = []
                        while i < len(x_child):
                            x_merge = ((x_child[i] + x_other[i]) / 2)
                            y_merge = ((y_child[i] + y_other[i]) / 2)
                            merge_coords.append([x_merge, y_merge])
                            i += 1
                        
                        #Update the merged child based on the new vertexes and increment count
                        merge_child.poly = shapely.geometry.Polygon(merge_coords)
                        merge_child.count += 1
                        
                new_children.append(merge_child)
            
            #Update the Quadrants children    
            self.children = new_children
                        
        
        return self           
      
    #Graph the parent quad and all of its children    
    def plot(self,ax):
        if any(isinstance(child, QuadTree) for child in self.children):
            for child in self.children:
                try:
                    child.plot()
                except:
                    traceback.print_exception()
        else:
            #Graph the boundarys of the QuadTree
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
                ax.plot(x,y,color="black")
            else:
                plt.plot(x,y,color = "black")
            
            #Graph the children of the QuadTree
            for child in self.children:
                child.plot(ax=ax)
    
    #Debug Function; Print the parent quad and all of its children        
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
            print("Coords: (" + str(self.topLeft[0]) + ", " + str(self.topLeft[1]) + ")\tX Size: " + str(self.xsize) + "\tChildren: " + child_string + "\n")
    
    #Debug Function; Count total number of children a QuadTree contains
    def count_children(self):
        sum = 0
        
        if any(isinstance(child,QuadTree) for child in self.children):
            for child in self.children:
                sum += child.count_children()
        else:
            sum = len(self.children)
        
        return sum