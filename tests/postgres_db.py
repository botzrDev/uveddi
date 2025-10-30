# tests/postgres_db.py
"""
Comprehensive integration tests for PostgreSQL database operations in Uveddi backend.
Uses psycopg2 for direct DB access.
"""
import os
import psycopg2
import pytest

DB_CONFIG = {
    'host': os.getenv('DB_POSTGRES_HOST', 'localhost'),
    'port': int(os.getenv('DB_POSTGRES_PORT', 5433)),
    'user': os.getenv('DB_POSTGRES_USER', 'uveddi'),
    'password': os.getenv('DB_POSTGRES_PASSWORD', 'uveddi'),
    'dbname': os.getenv('DB_POSTGRES_DB', 'uveddi'),
}

@pytest.fixture(scope="function")
def conn():
    connection = psycopg2.connect(**DB_CONFIG)
    connection.autocommit = True
    yield connection
    connection.close()

@pytest.fixture(autouse=True)
def clean_db(conn):
    # Truncate all tables before each test (respecting FK constraints)
    with conn.cursor() as cur:
        cur.execute('''
            DO $$ DECLARE
                r RECORD;
            BEGIN
                FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = 'public') LOOP
                    EXECUTE 'TRUNCATE TABLE ' || quote_ident(r.tablename) || ' RESTART IDENTITY CASCADE';
                END LOOP;
            END $$;
        ''')


def test_organization_crud(conn):
    with conn.cursor() as cur:
        # Create
        cur.execute("""
            INSERT INTO organizations (name, subscription_tier, created_at, api_key_usage_limit)
            VALUES (%s, %s, %s, %s) RETURNING organization_id
        """, ("TestOrg", "free", "2025-06-30T00:00:00Z", 100))
        org_id = cur.fetchone()[0]
        # Read
        cur.execute("SELECT name FROM organizations WHERE organization_id = %s", (org_id,))
        assert cur.fetchone()[0] == "TestOrg"
        # Update (use valid value for subscription_tier)
        cur.execute("UPDATE organizations SET subscription_tier = %s WHERE organization_id = %s", ("paid_api", org_id))
        cur.execute("SELECT subscription_tier FROM organizations WHERE organization_id = %s", (org_id,))
        assert cur.fetchone()[0] == "paid_api"
        # Delete
        cur.execute("DELETE FROM organizations WHERE organization_id = %s", (org_id,))
        cur.execute("SELECT COUNT(*) FROM organizations WHERE organization_id = %s", (org_id,))
        assert cur.fetchone()[0] == 0

def test_user_and_project_relationship(conn):
    with conn.cursor() as cur:
        # Insert org
        cur.execute("""
            INSERT INTO organizations (name, subscription_tier, created_at)
            VALUES (%s, %s, %s) RETURNING organization_id
        """, ("OrgA", "free", "2025-06-30T00:00:00Z"))
        org_id = cur.fetchone()[0]
        # Insert user
        cur.execute("""
            INSERT INTO users (email, username, password_hash, created_at, role, organization_id)
            VALUES (%s, %s, %s, %s, %s, %s) RETURNING user_id
        """, ("user@a.com", "usera", "hash", "2025-06-30T00:00:00Z", "org_admin", org_id))
        user_id = cur.fetchone()[0]
        # Insert project
        cur.execute("""
            INSERT INTO projects (organization_id, name, repository_url, created_at)
            VALUES (%s, %s, %s, %s) RETURNING project_id
        """, (org_id, "ProjA", "https://repo", "2025-06-30T00:00:00Z"))
        project_id = cur.fetchone()[0]
        # Check project belongs to org
        cur.execute("SELECT COUNT(*) FROM projects WHERE organization_id = %s", (org_id,))
        assert cur.fetchone()[0] == 1

def test_foreign_key_constraints(conn):
    with conn.cursor() as cur:
        # Insert org
        cur.execute("""
            INSERT INTO organizations (name, subscription_tier, created_at)
            VALUES (%s, %s, %s) RETURNING organization_id
        """, ("OrgB", "free", "2025-06-30T00:00:00Z"))
        org_id = cur.fetchone()[0]
        # Try to insert project with invalid org_id
        with pytest.raises(psycopg2.errors.ForeignKeyViolation):
            cur.execute("""
                INSERT INTO projects (organization_id, name, repository_url, created_at)
                VALUES (%s, %s, %s, %s)
            """, (9999, "ProjB", "https://repo", "2025-06-30T00:00:00Z"))
        # Insert valid project
        cur.execute("""
            INSERT INTO projects (organization_id, name, repository_url, created_at)
            VALUES (%s, %s, %s, %s) RETURNING project_id
        """, (org_id, "ProjB", "https://repo", "2025-06-30T00:00:00Z"))
        assert cur.fetchone()[0] is not None

def test_insert_and_query_anti_pattern_types(conn):
    with conn.cursor() as cur:
        # Insert default anti-patterns if not present
        cur.execute("SELECT COUNT(*) FROM anti_pattern_types")
        count = cur.fetchone()[0]
        if count < 6:
            cur.execute(
                """
                INSERT INTO anti_pattern_types (name, description, category, detection_heuristic_notes) VALUES
                ('Cyclic Dependency', 'Circular dependencies between modules or components', 'Dependency-Based', 'Detect using dependency graph analysis'),
                ('God Object', 'Classes or modules with too many responsibilities', 'Abstraction-Based', 'Analyze class size, method count, and coupling metrics'),
                ('Leaky Abstraction', 'Abstractions that expose implementation details', 'Abstraction-Based', 'Look for inappropriate exposure of internal state'),
                ('Tight Coupling', 'Excessive dependencies between components', 'Dependency-Based', 'Measure coupling metrics and dependency counts'),
                ('Long Parameter List', 'Functions with too many parameters', 'Code Smell', 'Count function parameters beyond reasonable threshold'),
                ('Duplicate Code', 'Repeated code blocks across the codebase', 'Code Smell', 'Use AST similarity analysis and string matching');
                """
            )
        cur.execute("SELECT COUNT(*) FROM anti_pattern_types")
        count = cur.fetchone()[0]
        assert count >= 6  # Default anti-patterns inserted