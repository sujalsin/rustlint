# This file contains unused imports, variables, and functions

import os                  # Used import
import sys                # Unused import
import json               # Unused import
from datetime import datetime, date  # Partially used import

def used_function():
    return os.path.join("a", "b")

def unused_function():    # Unused function
    return "Never called"

class UsedClass:
    def __init__(self):
        self.used_var = 42
        self.unused_var = 100    # Unused class variable
    
    def used_method(self):
        return self.used_var
    
    def unused_method(self):     # Unused method
        return self.unused_var

def main():
    # Unused variable
    unused_var = 10
    
    # Used variable
    used_var = 20
    print(used_var)
    
    # Partially used tuple unpacking
    x, y, z = (1, 2, 3)  # z is never used
    print(x + y)
    
    # Used class
    obj = UsedClass()
    print(obj.used_method())
    
    # Used function
    print(used_function())
    
    # Used import
    current_time = datetime.now()
    print(current_time)

if __name__ == "__main__":
    main()
