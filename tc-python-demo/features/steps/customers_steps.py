from behave import given, when, then
from customers import customers


@given("an empty customers table")
def empty_step_impl(context):
    customers.create_table()
    assert len(customers.get_all_customers()) == 0


@when("I add {amount:d} users to the database")
def amount_step_impl(context, amount):
    for i in range(0, amount):
        customers.create_customer("Boby", f"bob_{i}@gmail.com")


@then("it should have {new_amount:d} cusotmer in the database")
def amount_check_step_impl(context, new_amount):
    assert len(customers.get_all_customers()) == new_amount
