# This line is exactly 88 characters long which is fine according to our default configuration......
# This line is longer than 88 characters and should trigger a warning in our linter because it exceeds the limit
short_line = "ok"
multiline_string = """
This is a very long line in a multiline string that should also be checked by our linter because it exceeds 88 chars
"""

def function_with_long_line():
    return "This is a very long line inside a function that should definitely trigger a warning in our linter implementation"
