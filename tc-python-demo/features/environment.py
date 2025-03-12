import os
import logging
from behave.fixture import fixture, use_fixture_by_tag
from testcontainers.postgres import PostgresContainer


@fixture
def postgres_container(context):
    logging.getLogger("testcontainers").setLevel(logging.CRITICAL)
    context.postgres = PostgresContainer("postgres:16-alpine")
    context.postgres.start()

    os.environ["DB_HOST"] = context.postgres.get_container_host_ip()
    os.environ["DB_PORT"] = context.postgres.get_exposed_port(5432)
    os.environ["DB_USERNAME"] = context.postgres.username
    os.environ["DB_PASSWORD"] = context.postgres.password
    os.environ["DB_NAME"] = context.postgres.dbname

    yield context.postgres

    context.postgres.stop()


fixture_registry = {
    "fixture.postgres": postgres_container,
}


def before_tag(context, tag):
    if tag.startswith("fixture."):
        return use_fixture_by_tag(tag, context, fixture_registry)
