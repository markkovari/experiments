import os
import psycopg2

def get_connection():
    return psycopg2.connect(
        dbname=os.getenv("DB_NAME"),
        user=os.getenv("DB_USERNAME"),
        password=os.getenv("DB_PASSWORD"),
        host=os.getenv("DB_HOST"),
        port=os.getenv("DB_PORT")
    )

def create_table():
    with get_connection() as conn:
        with conn.cursor() as cur:
            cur.execute("""
                CREATE TABLE IF NOT EXISTS customers (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    email VARCHAR(255) UNIQUE NOT NULL
                );
            """)
            conn.commit()

def drop_table():
    with get_connection() as conn:
        with conn.cursor() as cur:
            cur.execute("DROP TABLE customers;")
            conn.commit()

def create_customer(name, email):
    with get_connection() as conn:
        with conn.cursor() as cur:
            cur.execute("INSERT INTO customers (name, email) VALUES (%s, %s)", (name, email))
            conn.commit()

def get_all_customers():
    with get_connection() as conn:
        with conn.cursor() as cur:
            cur.execute("SELECT * FROM customers")
            return cur.fetchall()

def get_customer_by_email(email):
    with get_connection() as conn:
        with conn.cursor() as cur:
            cur.execute("SELECT * FROM customers WHERE email = %s", (email,))
            return cur.fetchone()

def delete_all_customers():
    with get_connection() as conn:
        with conn.cursor() as cur:
            cur.execute("DELETE FROM customers")
            conn.commit()
