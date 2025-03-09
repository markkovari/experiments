import os

import psycopg
from app.config import Settings

settings = Settings()


def get_connection():

    return psycopg.connect(
        f"host={settings.POSTGRES_HOST} dbname={settings.POSTGRES_DB} user={settings.POSTGRES_USER} password={settings.POSTGRES_PASSWORD} port={settings.DATABASE_PORT}"
    )
