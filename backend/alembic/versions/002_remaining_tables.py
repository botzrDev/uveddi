"""Add remaining tables for Uveddi core functionality

Revision ID: 002_remaining_tables
Revises: 001_initial_tables
Create Date: 2025-06-27 14:30:00.000000

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

# revision identifiers
revision: str = '002_remaining_tables'
down_revision: str = '001_initial_tables'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    """
    Create remaining tables for the Uveddi database.
    """
    # Create projects table
    op.create_table(
        'projects',
        sa.Column('project_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('organization_id', sa.Integer(), nullable=False),
        sa.Column('name', sa.String(length=255), nullable=False),
        sa.Column('repository_url', sa.String(length=500), nullable=False),
        sa.Column('last_analyzed_commit', sa.String(length=255), nullable=True),
        sa.Column('config_data', postgresql.JSONB(astext_type=sa.Text()), nullable=True),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.text('now()'), nullable=False),
        sa.PrimaryKeyConstraint('project_id'),
        sa.ForeignKeyConstraint(['organization_id'], ['organizations.organization_id'], ondelete='CASCADE'),
        sa.UniqueConstraint('organization_id', 'name', name='uq_org_project_name')
    )
    
    # Create indexes for projects
    op.create_index('idx_projects_org_name', 'projects', ['organization_id', 'name'])
    op.create_index('idx_projects_repo_url', 'projects', ['repository_url'])

    # Create analysis_runs table
    op.create_table(
        'analysis_runs',
        sa.Column('run_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('project_id', sa.Integer(), nullable=False),
        sa.Column('user_id', sa.Integer(), nullable=True),
        sa.Column('start_time', sa.DateTime(timezone=True), nullable=False),
        sa.Column('end_time', sa.DateTime(timezone=True), nullable=True),
        sa.Column('status', sa.String(length=50), nullable=False, default='running', server_default='running'),
        sa.Column('ai_model_used', sa.String(length=100), nullable=True),
        sa.Column('total_files_scanned', sa.Integer(), nullable=True),
        sa.Column('total_issues_found', sa.Integer(), nullable=True),
        sa.Column('exit_code', sa.Integer(), nullable=True),
        sa.Column('raw_analysis_output', postgresql.JSONB(astext_type=sa.Text()), nullable=True),
        sa.PrimaryKeyConstraint('run_id'),
        sa.ForeignKeyConstraint(['project_id'], ['projects.project_id'], ondelete='CASCADE'),
        sa.ForeignKeyConstraint(['user_id'], ['users.user_id'], ondelete='SET NULL'),
        sa.CheckConstraint("status IN ('running', 'completed', 'failed', 'cancelled')", name='check_analysis_run_status')
    )
    
    # Create indexes for analysis_runs
    op.create_index('idx_analysis_runs_project_start', 'analysis_runs', ['project_id', 'start_time'])
    op.create_index('idx_analysis_runs_status_start', 'analysis_runs', ['status', 'start_time'])
    op.create_index('idx_analysis_runs_user', 'analysis_runs', ['user_id'])

    # Create architectural_issues table
    op.create_table(
        'architectural_issues',
        sa.Column('issue_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('run_id', sa.Integer(), nullable=False),
        sa.Column('anti_pattern_type_id', sa.Integer(), nullable=False),
        sa.Column('file_path', sa.String(length=1000), nullable=False),
        sa.Column('line_start', sa.Integer(), nullable=True),
        sa.Column('line_end', sa.Integer(), nullable=True),
        sa.Column('severity', sa.String(length=50), nullable=False),
        sa.Column('title', sa.String(length=500), nullable=False),
        sa.Column('description', sa.Text(), nullable=False),
        sa.Column('ai_refactoring_suggestion', sa.Text(), nullable=True),
        sa.Column('is_ignored', sa.Boolean(), nullable=False, default=False, server_default='false'),
        sa.Column('ignored_by_user_id', sa.Integer(), nullable=True),
        sa.Column('ignored_at', sa.DateTime(timezone=True), nullable=True),
        sa.PrimaryKeyConstraint('issue_id'),
        sa.ForeignKeyConstraint(['run_id'], ['analysis_runs.run_id'], ondelete='CASCADE'),
        sa.ForeignKeyConstraint(['anti_pattern_type_id'], ['anti_pattern_types.anti_pattern_type_id']),
        sa.ForeignKeyConstraint(['ignored_by_user_id'], ['users.user_id'], ondelete='SET NULL'),
        sa.CheckConstraint("severity IN ('critical', 'high', 'medium', 'low')", name='check_issue_severity')
    )
    
    # Create indexes for architectural_issues
    op.create_index('idx_issues_run_severity', 'architectural_issues', ['run_id', 'severity'])
    op.create_index('idx_issues_file_lines', 'architectural_issues', ['file_path', 'line_start', 'line_end'])
    op.create_index('idx_issues_ignored', 'architectural_issues', ['is_ignored', 'ignored_at'])
    op.create_index('idx_issues_anti_pattern', 'architectural_issues', ['anti_pattern_type_id'])

    # Create code_snippets table
    op.create_table(
        'code_snippets',
        sa.Column('snippet_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('issue_id', sa.Integer(), nullable=False),
        sa.Column('content', sa.Text(), nullable=False),
        sa.Column('language', sa.String(length=50), nullable=True),
        sa.Column('context_lines_before', sa.Integer(), nullable=True),
        sa.Column('context_lines_after', sa.Integer(), nullable=True),
        sa.PrimaryKeyConstraint('snippet_id'),
        sa.ForeignKeyConstraint(['issue_id'], ['architectural_issues.issue_id'], ondelete='CASCADE')
    )
    
    # Create indexes for code_snippets
    op.create_index('idx_snippets_issue_language', 'code_snippets', ['issue_id', 'language'])

    # Create reports table
    op.create_table(
        'reports',
        sa.Column('report_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('run_id', sa.Integer(), nullable=False),
        sa.Column('file_path', sa.String(length=1000), nullable=True),
        sa.Column('content', sa.Text(), nullable=True),
        sa.Column('generated_at', sa.DateTime(timezone=True), server_default=sa.text('now()'), nullable=False),
        sa.PrimaryKeyConstraint('report_id'),
        sa.ForeignKeyConstraint(['run_id'], ['analysis_runs.run_id'], ondelete='CASCADE'),
        sa.UniqueConstraint('run_id')
    )
    
    # Create indexes for reports
    op.create_index('idx_reports_generated', 'reports', ['generated_at'])

    # Create diagrams table
    op.create_table(
        'diagrams',
        sa.Column('diagram_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('report_id', sa.Integer(), nullable=False),
        sa.Column('type', sa.String(length=50), nullable=False),
        sa.Column('syntax_content', sa.Text(), nullable=False),
        sa.Column('description', sa.Text(), nullable=True),
        sa.Column('associated_issue_id', sa.Integer(), nullable=True),
        sa.PrimaryKeyConstraint('diagram_id'),
        sa.ForeignKeyConstraint(['report_id'], ['reports.report_id'], ondelete='CASCADE'),
        sa.ForeignKeyConstraint(['associated_issue_id'], ['architectural_issues.issue_id'], ondelete='SET NULL')
    )
    
    # Create indexes for diagrams
    op.create_index('idx_diagrams_report_type', 'diagrams', ['report_id', 'type'])
    op.create_index('idx_diagrams_issue', 'diagrams', ['associated_issue_id'])

    # Create plugins table
    op.create_table(
        'plugins',
        sa.Column('plugin_id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('name', sa.String(length=255), nullable=False),
        sa.Column('version', sa.String(length=50), nullable=False),
        sa.Column('description', sa.Text(), nullable=True),
        sa.Column('author_user_id', sa.Integer(), nullable=True),
        sa.Column('github_repo_url', sa.String(length=500), nullable=True),
        sa.Column('is_official', sa.Boolean(), nullable=False, default=False, server_default='false'),
        sa.Column('is_approved', sa.Boolean(), nullable=False, default=False, server_default='false'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.text('now()'), nullable=False),
        sa.Column('last_updated', sa.DateTime(timezone=True), server_default=sa.text('now()'), 
                  onupdate=sa.text('now()'), nullable=False),
        sa.PrimaryKeyConstraint('plugin_id'),
        sa.ForeignKeyConstraint(['author_user_id'], ['users.user_id'], ondelete='SET NULL'),
        sa.UniqueConstraint('name', 'version', name='uq_plugin_name_version')
    )
    
    # Create indexes for plugins
    op.create_index('idx_plugins_official_approved', 'plugins', ['is_official', 'is_approved'])
    op.create_index('idx_plugins_author', 'plugins', ['author_user_id'])


def downgrade() -> None:
    """
    Drop all remaining tables in reverse order of creation.
    """
    op.drop_table('plugins')
    op.drop_table('diagrams')
    op.drop_table('reports')
    op.drop_table('code_snippets')
    op.drop_table('architectural_issues')
    op.drop_table('analysis_runs')
    op.drop_table('projects')
