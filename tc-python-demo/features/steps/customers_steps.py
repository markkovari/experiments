from behave import given, when, then
from customers import customers
from typing import List, Tuple


@given("an empty customers table")
def empty_step_impl(context):
    customers.create_table()
    assert len(customers.get_all_customers()) == 0


@when("I add {amount:d} users to the database")
def amount_step_impl(context, amount):
    values: List[Tuple[str, str]] = [
        ("Bob", f"bob_${i}@gmail.com") for i in range(0, amount)
    ]
    customers.create_customers(values)


@then("it should have {new_amount:d} customer in the database")
def amount_check_step_impl(context, new_amount):
    assert len(customers.get_all_customers()) == new_amount
