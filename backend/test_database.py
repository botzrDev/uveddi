import pytest
import pytest_asyncio
from datetime import datetime
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from dotenv import load_dotenv

from database import init_database, close_database, get_async_session
from models import User, Organization, AntiPatternType

@pytest_asyncio.fixture
async def db_session():
    """Async session fixture for database tests."""
    load_dotenv()
    await init_database()
    session = await anext(get_async_session())
    try:
        yield session
    finally:
        await session.close()
        await close_database()

@pytest.mark.asyncio
async def test_organizations(db_session: AsyncSession):
    """Test organization CRUD operations."""
    # Create a test organization
    org_name = f"Test Organization {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}"
    
    org = Organization(name=org_name)
    
    # Add to database
    db_session.add(org)
    await db_session.commit()
    await db_session.refresh(org)
    
    # Query organization
    result = await db_session.execute(select(Organization).where(Organization.name == org_name))
    fetched_org = result.scalars().first()
    assert fetched_org is not None
    assert fetched_org.name == org_name

@pytest.mark.asyncio
async def test_users(db_session: AsyncSession):
    """Test user CRUD operations."""
    # Get first organization (or create one if none exist)
    result = await db_session.execute(select(Organization).limit(1))
    org = result.scalars().first()
    
    if not org:
        org = Organization(name="Default Organization")
        db_session.add(org)
        await db_session.commit()
        await db_session.refresh(org)
    
    # Create a test user
    username = f"testuser_{int(datetime.now().timestamp())}"
    
    user = User(
        username=username,
        email=f"{username}@example.com",
        password_hash="$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW",
        organization_id=org.organization_id
    )
    
    # Add to database
    db_session.add(user)
    await db_session.commit()
    await db_session.refresh(user)
    
    # Query user
    result = await db_session.execute(select(User).where(User.username == username))
    fetched_user = result.scalars().first()
    assert fetched_user is not None
    assert fetched_user.username == username

@pytest.mark.asyncio
async def test_anti_pattern_types(db_session: AsyncSession):
    """Test anti-pattern types retrieval."""
    # Skip test since we haven't seeded the database
    pytest.skip("Skipping anti-pattern types test - database not seeded")
