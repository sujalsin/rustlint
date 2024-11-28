# This file contains various syntax errors to test our linter

# Missing parenthesis
print "Hello World"

# Invalid indentation mixing spaces and tabs
def mixed_indentation():
	x = 1  # Tab
    y = 2  # Spaces
    return x + y

# Incomplete function definition
def incomplete_function(x, y:

# Missing colon in if statement
if True
    print("This is wrong")

# Invalid comparison
if 1 =< 2:
    print("Wrong operator")

# Unclosed string
unclosed_string = "This string never ends

# Invalid class definition
class MissingColon
    def __init__(self):
        pass

# Invalid syntax in list comprehension
numbers = [x for x in range(10) if x % 2 == 0 for]

# Unmatched parentheses
result = ((1 + 2) * 3

# Invalid decorator syntax
@decorator
def func():
    pass

# Missing except or finally after try
try:
    x = 1/0
