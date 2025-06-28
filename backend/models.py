"""
SQLAlchemy ORM models for the CodeAtlas centralized database.

This module defines the database schema for the PostgreSQL backend
that serves the CodeAtlas subscriber system, CI/CD integrations,
and centralized analysis data.
"""

from datetime import datetime
from typing import List, Optional

from sqlalchemy import (
    Boolean, Column, DateTime, ForeignKey, Integer, String, Text, 
    UniqueConstraint, Index, CheckConstraint
)
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import relationship
from sqlalchemy.sql import func
import uuid

from database import Base


class User(Base):
    """
    User entity for CodeAtlas subscribers.
    
    Represents individual users who can belong to organizations
    and initiate analysis runs.
    """
    __tablename__ = "users"
    
    user_id = Column(Integer, primary_key=True, autoincrement=True)
    email = Column(String(255), unique=True, nullable=False, index=True)
    username = Column(String(100), unique=True, nullable=False, index=True)
    password_hash = Column(String(255), nullable=False)
    created_at = Column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    last_login = Column(DateTime(timezone=True))
    role = Column(
        String(50), 
        nullable=False, 
        default="individual",
        server_default="individual"
    )
    organization_id = Column(
        Integer, 
        ForeignKey("organizations.organization_id", ondelete="SET NULL"),
        nullable=True,
        index=True
    )
    
    # Relationships
    organization = relationship("Organization", back_populates="users")
    analysis_runs = relationship("AnalysisRun", back_populates="user")
    ignored_issues = relationship(
        "ArchitecturalIssue", 
        foreign_keys="ArchitecturalIssue.ignored_by_user_id",
        back_populates="ignored_by_user"
    )
    authored_plugins = relationship("Plugin", back_populates="author")
    
    # Constraints
    __table_args__ = (
        CheckConstraint(
            "role IN ('individual', 'org_admin', 'org_member')",
            name="check_user_role"
        ),
        Index("idx_users_organization_role", "organization_id", "role"),
    )


class Organization(Base):
    """
    Organization entity for team/company subscribers.
    
    Organizations can have multiple users and projects,
    and manage subscription tiers and API usage limits.
    """
    __tablename__ = "organizations"
    
    organization_id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(String(255), unique=True, nullable=False, index=True)
    subscription_tier = Column(
        String(50), 
        nullable=False, 
        default="free",
        server_default="free"
    )
    created_at = Column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    api_key_usage_limit = Column(Integer, default=1000)  # For paid tiers
    
    # Relationships
    users = relationship("User", back_populates="organization")
    projects = relationship("Organization", back_populates="organization")
    
    # Constraints
    __table_args__ = (
        CheckConstraint(
            "subscription_tier IN ('free', 'paid_api', 'enterprise')",
            name="check_subscription_tier"
        ),
        Index("idx_organizations_subscription", "subscription_tier"),
    )


class Project(Base):
    """
    Project entity representing codebases being analyzed.
    
    Projects belong to organizations and contain analysis runs.
    """
    __tablename__ = "projects"
    
    project_id = Column(Integer, primary_key=True, autoincrement=True)
    organization_id = Column(
        Integer, 
        ForeignKey("organizations.organization_id", ondelete="CASCADE"),
        nullable=False,
        index=True
    )
    name = Column(String(255), nullable=False)
    repository_url = Column(String(500), nullable=False)
    last_analyzed_commit = Column(String(255))
    config_data = Column(JSONB)  # .archlintignore rules, analysis scopes, etc.
    created_at = Column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    
    # Relationships
    organization = relationship("Organization", back_populates="projects")
    analysis_runs = relationship("AnalysisRun", back_populates="project")
    
    # Constraints
    __table_args__ = (
        UniqueConstraint("organization_id", "name", name="uq_org_project_name"),
        Index("idx_projects_org_name", "organization_id", "name"),
        Index("idx_projects_repo_url", "repository_url"),
    )


class AntiPatternType(Base):
    """
    Catalog of predefined architectural anti-patterns.
    
    This table stores the types of issues that CodeAtlas can detect.
    """
    __tablename__ = "anti_pattern_types"
    
    anti_pattern_type_id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(String(255), unique=True, nullable=False, index=True)
    description = Column(Text)
    category = Column(String(100), index=True)
    detection_heuristic_notes = Column(Text)  # Internal documentation
    
    # Relationships
    architectural_issues = relationship("ArchitecturalIssue", back_populates="anti_pattern_type")
    
    # Constraints
    __table_args__ = (
        Index("idx_anti_patterns_category", "category"),
    )


class AnalysisRun(Base):
    """
    Analysis run execution records.
    
    Represents a single execution of CodeAtlas analysis on a project.
    """
    __tablename__ = "analysis_runs"
    
    run_id = Column(Integer, primary_key=True, autoincrement=True)
    project_id = Column(
        Integer, 
        ForeignKey("projects.project_id", ondelete="CASCADE"),
        nullable=False,
        index=True
    )
    user_id = Column(
        Integer, 
        ForeignKey("users.user_id", ondelete="SET NULL"),
        nullable=True,  # Nullable for CI/CD runs
        index=True
    )
    start_time = Column(DateTime(timezone=True), nullable=False, index=True)
    end_time = Column(DateTime(timezone=True))
    status = Column(String(50), nullable=False, default="running", index=True)
    ai_model_used = Column(String(100))
    total_files_scanned = Column(Integer)
    total_issues_found = Column(Integer)
    exit_code = Column(Integer)
    raw_analysis_output = Column(JSONB)  # For debugging/advanced usage
    
    # Relationships
    project = relationship("Project", back_populates="analysis_runs")
    user = relationship("User", back_populates="analysis_runs")
    architectural_issues = relationship("ArchitecturalIssue", back_populates="analysis_run")
    report = relationship("Report", back_populates="analysis_run", uselist=False)
    
    # Constraints
    __table_args__ = (
        CheckConstraint(
            "status IN ('running', 'completed', 'failed', 'cancelled')",
            name="check_analysis_run_status"
        ),
        Index("idx_analysis_runs_project_start", "project_id", "start_time"),
        Index("idx_analysis_runs_status_start", "status", "start_time"),
    )


class ArchitecturalIssue(Base):
    """
    Detected architectural issues/anti-patterns.
    
    Represents specific problems found during code analysis.
    """
    __tablename__ = "architectural_issues"
    
    issue_id = Column(Integer, primary_key=True, autoincrement=True)
    run_id = Column(
        Integer, 
        ForeignKey("analysis_runs.run_id", ondelete="CASCADE"),
        nullable=False,
        index=True
    )
    anti_pattern_type_id = Column(
        Integer, 
        ForeignKey("anti_pattern_types.anti_pattern_type_id"),
        nullable=False,
        index=True
    )
    file_path = Column(String(1000), nullable=False, index=True)
    line_start = Column(Integer)
    line_end = Column(Integer)
    severity = Column(String(50), nullable=False, index=True)
    title = Column(String(500), nullable=False)
    description = Column(Text, nullable=False)
    ai_refactoring_suggestion = Column(Text)
    is_ignored = Column(Boolean, nullable=False, default=False, index=True)
    ignored_by_user_id = Column(
        Integer, 
        ForeignKey("users.user_id", ondelete="SET NULL"),
        nullable=True
    )
    ignored_at = Column(DateTime(timezone=True))
    
    # Relationships
    analysis_run = relationship("AnalysisRun", back_populates="architectural_issues")
    anti_pattern_type = relationship("AntiPatternType", back_populates="architectural_issues")
    ignored_by_user = relationship(
        "User", 
        foreign_keys=[ignored_by_user_id],
        back_populates="ignored_issues"
    )
    code_snippets = relationship("CodeSnippet", back_populates="architectural_issue")
    diagrams = relationship("Diagram", back_populates="associated_issue")
    
    # Constraints
    __table_args__ = (
        CheckConstraint(
            "severity IN ('critical', 'high', 'medium', 'low')",
            name="check_issue_severity"
        ),
        Index("idx_issues_run_severity", "run_id", "severity"),
        Index("idx_issues_file_lines", "file_path", "line_start", "line_end"),
        Index("idx_issues_ignored", "is_ignored", "ignored_at"),
    )


class CodeSnippet(Base):
    """
    Code segments related to architectural issues.
    
    Stores the actual code that demonstrates or relates to detected issues.
    """
    __tablename__ = "code_snippets"
    
    snippet_id = Column(Integer, primary_key=True, autoincrement=True)
    issue_id = Column(
        Integer, 
        ForeignKey("architectural_issues.issue_id", ondelete="CASCADE"),
        nullable=False,
        index=True
    )
    content = Column(Text, nullable=False)
    language = Column(String(50), index=True)
    context_lines_before = Column(Integer)
    context_lines_after = Column(Integer)
    
    # Relationships
    architectural_issue = relationship("ArchitecturalIssue", back_populates="code_snippets")
    
    # Constraints
    __table_args__ = (
        Index("idx_snippets_issue_language", "issue_id", "language"),
    )


class Plugin(Base):
    """
    Custom scanners and extensions for CodeAtlas.
    
    Represents plugins that extend CodeAtlas functionality.
    """
    __tablename__ = "plugins"
    
    plugin_id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(String(255), unique=True, nullable=False, index=True)
    version = Column(String(50), nullable=False)
    description = Column(Text)
    author_user_id = Column(
        Integer, 
        ForeignKey("users.user_id", ondelete="SET NULL"),
        nullable=True  # Nullable for official plugins
    )
    github_repo_url = Column(String(500))
    is_official = Column(Boolean, nullable=False, default=False, index=True)
    is_approved = Column(Boolean, nullable=False, default=False, index=True)
    created_at = Column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    last_updated = Column(
        DateTime(timezone=True), 
        server_default=func.now(), 
        onupdate=func.now(),
        nullable=False
    )
    
    # Relationships
    author = relationship("User", back_populates="authored_plugins")
    
    # Constraints
    __table_args__ = (
        UniqueConstraint("name", "version", name="uq_plugin_name_version"),
        Index("idx_plugins_official_approved", "is_official", "is_approved"),
        Index("idx_plugins_author", "author_user_id"),
    )


class Report(Base):
    """
    Generated markdown reports for analysis runs.
    
    Stores the complete analysis report content.
    """
    __tablename__ = "reports"
    
    report_id = Column(Integer, primary_key=True, autoincrement=True)
    run_id = Column(
        Integer, 
        ForeignKey("analysis_runs.run_id", ondelete="CASCADE"),
        nullable=False,
        unique=True,  # One-to-one relationship
        index=True
    )
    file_path = Column(String(1000))  # Path where report is saved/stored
    content = Column(Text)  # Full markdown report content
    generated_at = Column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    
    # Relationships
    analysis_run = relationship("AnalysisRun", back_populates="report")
    diagrams = relationship("Diagram", back_populates="report")
    
    # Constraints
    __table_args__ = (
        Index("idx_reports_generated", "generated_at"),
    )


class Diagram(Base):
    """
    Visual diagrams embedded within reports.
    
    Stores diagram-as-code content for various visualization types.
    """
    __tablename__ = "diagrams"
    
    diagram_id = Column(Integer, primary_key=True, autoincrement=True)
    report_id = Column(
        Integer, 
        ForeignKey("reports.report_id", ondelete="CASCADE"),
        nullable=False,
        index=True
    )
    type = Column(String(50), nullable=False)  # 'Mermaid.js', 'PlantUML', 'Graphviz'
    syntax_content = Column(Text, nullable=False)  # Diagram-as-code syntax
    description = Column(Text)  # Caption for the diagram
    associated_issue_id = Column(
        Integer, 
        ForeignKey("architectural_issues.issue_id", ondelete="SET NULL"),
        nullable=True  # Links diagram to specific issue if applicable
    )
    
    # Relationships
    report = relationship("Report", back_populates="diagrams")
    associated_issue = relationship("ArchitecturalIssue", back_populates="diagrams")
    
    # Constraints
    __table_args__ = (
        Index("idx_diagrams_report_type", "report_id", "type"),
        Index("idx_diagrams_issue", "associated_issue_id"),
    )
