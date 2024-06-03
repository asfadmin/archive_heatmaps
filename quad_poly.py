import matplotlib.pyplot as plt
import shapely.geometry
import geopandas as gpd
import numpy as np
import traceback

class QuadTree:
    def __init__(self, topL: list, xsize, ysize, children: list):
        self.topLeft = topL
        self.xsize = xsize
        self.ysize = ysize
        self.children = children
        
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
    def split(self):
         # Stops recurssion
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
                    child.split()
        
        return self
      
    #Graph the parent quad and all of its children 

    def plot(self,ax = None):
        if any(isinstance(child, QuadTree) for child in self.children):
            for child in self.children:
                try:
                    child.plot(ax=ax)
                except:
                    traceback.print_exception()
        else:
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
                ax.plot(x,y,color = "black")
            else:
                plt.plot(x,y,color = "black")
            
    
    #Debug Function; Print the parent quad and all of its children        
    def print(self):
        if any(isinstance(child, QuadTree) for child in self.children):
            for child in self.children:
                child.print()
        else:
            print("Coords: (" + str(self.topLeft[0]) + ", " + str(self.topLeft[1]) + ")\tX Size: " + str(self.xsize) + "\tChildren: " + str(len(self.children)) + "\n")