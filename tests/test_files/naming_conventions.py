# Good naming conventions
def calculate_total(x, y):
    return x + y

class UserAccount:
    def __init__(self):
        self.balance = 0

# Bad naming conventions
def calculateTotal(x, y):  # should be snake_case
    return x + y

class user_account:  # should be PascalCase
    def __init__(self):
        self.Balance = 0  # should be snake_case

# Variable naming
good_variable_name = 42
BadVariableName = 24  # should be snake_case
CONSTANT_VALUE = 100  # this is fine
mixedCase = "bad"  # should be snake_case
