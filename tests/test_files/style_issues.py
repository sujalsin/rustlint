# This file contains various style issues that our linter should detect

import sys, os                 # Multiple imports on one line
import json as j              # Unused import
from typing import List,Dict  # Missing spaces after comma

def badlyFormattedFunction( x,y):    # Bad spacing in function definition
    z=x+y                    # Missing spaces around operators
    return z                 # This is fine

class badClassName:          # Class name should use CamelCase
    def __init__(self):
        self.some_very_long_variable_name_that_exceeds_our_line_length_limit_and_should_be_detected_by_our_linter = 42
    
    def BadMethodName(self): # Method should use snake_case
        pass

# This is a very long comment line that definitely exceeds our line length limit of 88 characters and should trigger a warning from our linter because it is way too long and even longer than before
very_long_string = "This is a very long string that definitely exceeds our line length limit of 88 characters and should trigger a warning from our linter because it is way too long and even longer than before and this is just ridiculous"

# Inconsistent indentation
def inconsistent_indentation():
    x = 1
   y = 2  # Bad indentation (3 spaces)
      z = 3  # Bad indentation (6 spaces)
    return x + y + z
