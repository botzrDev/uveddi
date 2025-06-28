#!/usr/bin/env python3
"""
Database test script for CodeAtlas backend.

This script performs basic operations with the database to verify
that the connection, models, and CRUD operations are working correctly.
"""

import asyncio
import os
import sys
from datetime import datetime
from typing import List, Optional

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from dotenv import load_dotenv

# Add the current directory to the path
sys.path.append(os.path.dirname(os.path.realpath(__file__)))

# Import database and models
from database import init_database, close_database, get_async_session
from models import User, Organization, AntiPatternType


async def test_database_connection():
    """Test basic database operations."""
    print("Testing database connection and operations...")
    
    # Initialize the database
    await init_database()
    
    try:
        # Get a session
        async for session in get_async_session():
            await test_organizations(session)
            await test_users(session)
            await test_anti_pattern_types(session)
            break
    finally:
        # Close database connections
        await close_database()


async def test_organizations(session: AsyncSession):
    """Test organization CRUD operations."""
    print("\n=== Testing Organization Operations ===")
    
    # Create a test organization
    org_name = f"Test Organization {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}"
    
    org = Organization(
        name=org_name,
        description="A test organization for database validation",
    )
    
    # Add to database
    session.add(org)
    await session.commit()
    await session.refresh(org)
    
    print(f"Created organization: {org.name} (ID: {org.organization_id})")
    
    # Query organization
    result = await session.execute(select(Organization).where(Organization.name == org_name))
    fetched_org = result.scalars().first()
    
    if fetched_org:
        print(f"Successfully fetched organization: {fetched_org.name}")
    else:
        print("Failed to fetch organization!")


async def test_users(session: AsyncSession):
    """Test user CRUD operations."""
    print("\n=== Testing User Operations ===")
    
    # Get first organization (or create one if none exist)
    result = await session.execute(select(Organization).limit(1))
    org = result.scalars().first()
    
    if not org:
        org = Organization(name="Default Organization", description="Default organization for testing")
        session.add(org)
        await session.commit()
        await session.refresh(org)
    
    # Create a test user
    username = f"testuser_{int(datetime.now().timestamp())}"
    
    user = User(
        username=username,
        email=f"{username}@example.com",
        password_hash="$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW",  # "password"
        organization_id=org.organization_id
    )
    
    # Add to database
    session.add(user)
    await session.commit()
    await session.refresh(user)
    
    print(f"Created user: {user.username} (ID: {user.user_id})")
    
    # Query user
    result = await session.execute(select(User).where(User.username == username))
    fetched_user = result.scalars().first()
    
    if fetched_user:
        print(f"Successfully fetched user: {fetched_user.username}")
    else:
        print("Failed to fetch user!")


async def test_anti_pattern_types(session: AsyncSession):
    """Test anti-pattern types retrieval."""
    print("\n=== Testing Anti-Pattern Types ===")
    
    # Query anti-pattern types
    result = await session.execute(select(AntiPatternType))
    anti_patterns = result.scalars().all()
    
    if anti_patterns:
        print(f"Found {len(anti_patterns)} anti-pattern types:")
        for ap in anti_patterns:
            print(f"  - {ap.name}: {ap.description[:50]}...")
    else:
        print("No anti-pattern types found. Migrations may not have been run.")


if __name__ == "__main__":
    # Load environment variables
    load_dotenv()
    
    # Run the test
    asyncio.run(test_database_connection())
    
    print("\nDatabase test completed!")
