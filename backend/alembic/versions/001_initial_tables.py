"""Initial migration: Create User and Organization tables

Revision ID: 001_initial_tables
Revises: 
Create Date: 2025-06-27 12:00:00.000000

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

# revision identifiers
revision: str = '001_initial_tables'
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    """
    Create User and Organization tables with their relationships.
    """
    # Create organizations table
    op.create_table(
        'organizations',
        sa.Column('organization_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('name', sa.String(length=255), nullable=False),
        sa.Column('subscription_tier', sa.String(length=50), nullable=False, server_default='free'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.text('now()'), nullable=False),
        sa.Column('api_key_usage_limit', sa.Integer(), default=1000),
        sa.PrimaryKeyConstraint('organization_id'),
        sa.UniqueConstraint('name'),
        sa.CheckConstraint("subscription_tier IN ('free', 'paid_api', 'enterprise')", name='check_subscription_tier')
    )
    
    # Create indexes for organizations
    op.create_index('idx_organizations_subscription', 'organizations', ['subscription_tier'])
    op.create_index(op.f('ix_organizations_name'), 'organizations', ['name'])

    # Create users table
    op.create_table(
        'users',
        sa.Column('user_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('email', sa.String(length=255), nullable=False),
        sa.Column('username', sa.String(length=100), nullable=False),
        sa.Column('password_hash', sa.String(length=255), nullable=False),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.text('now()'), nullable=False),
        sa.Column('last_login', sa.DateTime(timezone=True), nullable=True),
        sa.Column('role', sa.String(length=50), nullable=False, server_default='individual'),
        sa.Column('organization_id', sa.Integer(), nullable=True),
        sa.PrimaryKeyConstraint('user_id'),
        sa.ForeignKeyConstraint(['organization_id'], ['organizations.organization_id'], ondelete='SET NULL'),
        sa.UniqueConstraint('email'),
        sa.UniqueConstraint('username'),
        sa.CheckConstraint("role IN ('individual', 'org_admin', 'org_member')", name='check_user_role')
    )
    
    # Create indexes for users
    op.create_index(op.f('ix_users_email'), 'users', ['email'])
    op.create_index(op.f('ix_users_username'), 'users', ['username'])
    op.create_index('idx_users_organization_role', 'users', ['organization_id', 'role'])
    op.create_index(op.f('ix_users_organization_id'), 'users', ['organization_id'])

    # Create anti_pattern_types table (reference data)
    op.create_table(
        'anti_pattern_types',
        sa.Column('anti_pattern_type_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('name', sa.String(length=255), nullable=False),
        sa.Column('description', sa.Text(), nullable=True),
        sa.Column('category', sa.String(length=100), nullable=True),
        sa.Column('detection_heuristic_notes', sa.Text(), nullable=True),
        sa.PrimaryKeyConstraint('anti_pattern_type_id'),
        sa.UniqueConstraint('name')
    )
    
    # Create indexes for anti_pattern_types
    op.create_index(op.f('ix_anti_pattern_types_name'), 'anti_pattern_types', ['name'])
    op.create_index('idx_anti_patterns_category', 'anti_pattern_types', ['category'])

    # Insert default anti-pattern types
    op.execute("""
        INSERT INTO anti_pattern_types (name, description, category, detection_heuristic_notes) VALUES
        ('Cyclic Dependency', 'Circular dependencies between modules or components', 'Dependency-Based', 'Detect using dependency graph analysis'),
        ('God Object', 'Classes or modules with too many responsibilities', 'Abstraction-Based', 'Analyze class size, method count, and coupling metrics'),
        ('Leaky Abstraction', 'Abstractions that expose implementation details', 'Abstraction-Based', 'Look for inappropriate exposure of internal state'),
        ('Tight Coupling', 'Excessive dependencies between components', 'Dependency-Based', 'Measure coupling metrics and dependency counts'),
        ('Long Parameter List', 'Functions with too many parameters', 'Code Smell', 'Count function parameters beyond reasonable threshold'),
        ('Duplicate Code', 'Repeated code blocks across the codebase', 'Code Smell', 'Use AST similarity analysis and string matching')
    """)


def downgrade() -> None:
    """
    Drop User and Organization tables and related objects.
    """
    # Drop tables in reverse order of creation (to handle foreign keys)
    op.drop_table('anti_pattern_types')
    op.drop_table('users')
    op.drop_table('organizations')
