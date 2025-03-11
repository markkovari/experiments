import os
import pytest
from testcontainers.postgres import PostgresContainer
from customers import customers

@pytest.fixture(scope="module", autouse=True)
def postgres():
    with PostgresContainer("postgres:16-alpine") as postgres:
        os.environ["DB_CONN"] = postgres.get_connection_url()
        os.environ["DB_HOST"] = postgres.get_container_host_ip()
        os.environ["DB_PORT"] = str(postgres.get_exposed_port(5432))
        os.environ["DB_USERNAME"] = postgres.username
        os.environ["DB_PASSWORD"] = postgres.password
        os.environ["DB_NAME"] = postgres.dbname
        customers.create_table()
        yield
        customers.drop_table()

def test_create_and_get_customer(postgres):
    customers.create_customer("Alice", "alice@example.com")
    customer = customers.get_customer_by_email("alice@example.com")
    assert customer[1] == "Alice"
    assert customer[2] == "alice@example.com"

def test_get_all_customers(postgres):
    customers.create_customer("Bob", "bob@example.com")
    customers_list = customers.get_all_customers()
    assert len(customers_list) == 2


def test_get_all_customer_a_lot(postgres):
    for i in range(0, 1000):
        customers.create_customer("Bob", f"bob_{i}@example.com")
    customers_list = customers.get_all_customers()
    assert len(customers_list) == 1002
