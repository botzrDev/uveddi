"""
Database configuration and connection management for Uveddi backend.

This module handles the PostgreSQL database connection using SQLAlchemy ORM
for the centralized Uveddi subscriber database.
"""

import os
from typing import AsyncGenerator, Optional

from sqlalchemy import create_engine, MetaData
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from pydantic_settings import BaseSettings


class DatabaseSettings(BaseSettings):
    """Database configuration settings."""
    
    # Database type (sqlite for development, postgresql for production)
    db_type: str = "sqlite"
    
    # SQLite settings
    sqlite_file: str = "uveddi_backend.db"
    
    # PostgreSQL connection parameters
    postgres_host: str = "localhost"
    postgres_port: int = 5432
    postgres_user: str = "uveddi"
    postgres_password: str = "uveddi"
    postgres_db: str = "uveddi"
    
    # Connection pool settings
    pool_size: int = 10
    max_overflow: int = 20
    pool_pre_ping: bool = True
    
    # Environment-specific settings
    environment: str = "development"
    
    @property
    def database_url(self) -> str:
        """Construct the database URL based on db_type."""
        if self.db_type == "sqlite":
            return f"sqlite:///{self.sqlite_file}"
        else:
            return (
                f"postgresql://{self.postgres_user}:{self.postgres_password}"
                f"@{self.postgres_host}:{self.postgres_port}/{self.postgres_db}"
            )
    
    @property
    def async_database_url(self) -> str:
        """Construct the async database URL based on db_type."""
        if self.db_type == "sqlite":
            return f"sqlite+aiosqlite:///{self.sqlite_file}"
        else:
            return (
                f"postgresql+asyncpg://{self.postgres_user}:{self.postgres_password}"
                f"@{self.postgres_host}:{self.postgres_port}/{self.postgres_db}"
            )
    
    class Config:
        env_file = ".env"
        env_prefix = "DB_"


# Global database settings
db_settings = DatabaseSettings()

# SQLAlchemy metadata and base
metadata = MetaData()
Base = declarative_base(metadata=metadata)

# Synchronous engine for migrations
engine = create_engine(
    db_settings.database_url,
    pool_size=db_settings.pool_size,
    max_overflow=db_settings.max_overflow,
    pool_pre_ping=db_settings.pool_pre_ping,
    echo=db_settings.environment == "development",
)

# Asynchronous engine for FastAPI
async_engine = create_async_engine(
    db_settings.async_database_url,
    pool_size=db_settings.pool_size,
    max_overflow=db_settings.max_overflow,
    pool_pre_ping=db_settings.pool_pre_ping,
    echo=db_settings.environment == "development",
)

# Session factories
SessionLocal = sessionmaker(
    autocommit=False, 
    autoflush=False, 
    bind=engine
)

AsyncSessionLocal = async_sessionmaker(
    async_engine,
    class_=AsyncSession,
    expire_on_commit=False
)


async def get_async_session() -> AsyncGenerator[AsyncSession, None]:
    """
    Dependency to get an async database session.
    
    This function is used as a FastAPI dependency to provide
    database sessions to route handlers.
    
    Yields:
        AsyncSession: An async SQLAlchemy session
    """
    async with AsyncSessionLocal() as session:
        try:
            yield session
            await session.commit()
        except Exception:
            await session.rollback()
            raise
        finally:
            await session.close()


def get_sync_session():
    """
    Get a synchronous database session.
    
    This is mainly used for migrations and administrative tasks.
    
    Yields:
        Session: A synchronous SQLAlchemy session
    """
    session = SessionLocal()
    try:
        yield session
        session.commit()
    except Exception:
        session.rollback()
        raise
    finally:
        session.close()


async def init_database():
    """
    Initialize the database by creating all tables.
    
    This function should be called on application startup.
    """
    async with async_engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)


async def close_database():
    """
    Close database connections.
    
    This function should be called on application shutdown.
    """
    await async_engine.dispose()


# Example environment configuration
def get_database_url_from_env() -> str:
    """
    Get database URL from environment variables.
    
    This function demonstrates how to securely configure the database
    connection for production deployments.
    
    Environment variables expected:
    - DATABASE_URL: Complete PostgreSQL connection string
    or individual components:
    - DB_POSTGRES_HOST
    - DB_POSTGRES_PORT  
    - DB_POSTGRES_USER
    - DB_POSTGRES_PASSWORD
    - DB_POSTGRES_DB
    
    Returns:
        str: Database connection URL
    """
    # Option 1: Complete connection string
    database_url = os.getenv("DATABASE_URL")
    if database_url:
        return database_url
    
    # Option 2: Individual components
    host = os.getenv("DB_POSTGRES_HOST", "localhost")
    port = os.getenv("DB_POSTGRES_PORT", "5432")
    user = os.getenv("DB_POSTGRES_USER", "uveddi")
    password = os.getenv("DB_POSTGRES_PASSWORD", "")
    database = os.getenv("DB_POSTGRES_DB", "uveddi")
    
    if not password:
        raise ValueError("Database password must be provided via environment variables")
    
    return f"postgresql://{user}:{password}@{host}:{port}/{database}"


# Production configuration example
"""
Example .env file for production:

# Database Configuration
DB_POSTGRES_HOST=your-rds-endpoint.amazonaws.com
DB_POSTGRES_PORT=5432
DB_POSTGRES_USER=uveddi_user
DB_POSTGRES_PASSWORD=your-secure-password
DB_POSTGRES_DB=uveddi_prod
DB_POOL_SIZE=20
DB_MAX_OVERFLOW=40
DB_ENVIRONMENT=production

# Or use a complete connection string:
DATABASE_URL=postgresql://user:pass@host:port/db?sslmode=require

# For cloud services like AWS RDS, Google Cloud SQL:
DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require&connect_timeout=10

# For connection pooling with cloud services:
DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require&pool_size=20&max_overflow=40
"""
