"""
FastAPI application for Uveddi backend service.

This module sets up the FastAPI application with database dependency injection,
authentication, and CRUD endpoints for organizations and projects.
"""

from typing import List, Optional
from datetime import datetime

from fastapi import FastAPI, Depends, HTTPException, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
from sqlalchemy.orm import selectinload
from pydantic import BaseModel, ConfigDict
import logging

from database import get_async_session, init_database, close_database
from models import User, Organization, Project

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# FastAPI app instance
app = FastAPI(
    title="Uveddi Backend API",
    description="Centralized backend service for Uveddi code analysis platform",
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc"
)

# Security
security = HTTPBearer()


# Pydantic models for API requests/responses
class OrganizationCreate(BaseModel):
    """Schema for creating a new organization."""
    name: str
    subscription_tier: str = "free"
    api_key_usage_limit: Optional[int] = 1000

    model_config = ConfigDict(from_attributes=True)


class OrganizationResponse(BaseModel):
    """Schema for organization API responses."""
    organization_id: int
    name: str
    subscription_tier: str
    created_at: datetime
    api_key_usage_limit: Optional[int]

    model_config = ConfigDict(from_attributes=True)


class ProjectCreate(BaseModel):
    """Schema for creating a new project."""
    name: str
    repository_url: str
    last_analyzed_commit: Optional[str] = None
    config_data: Optional[dict] = None

    model_config = ConfigDict(from_attributes=True)


class ProjectResponse(BaseModel):
    """Schema for project API responses."""
    project_id: int
    organization_id: int
    name: str
    repository_url: str
    last_analyzed_commit: Optional[str]
    config_data: Optional[dict]
    created_at: datetime

    model_config = ConfigDict(from_attributes=True)


class UserResponse(BaseModel):
    """Schema for user API responses."""
    user_id: int
    email: str
    username: str
    role: str
    organization_id: Optional[int]
    created_at: datetime
    last_login: Optional[datetime]

    model_config = ConfigDict(from_attributes=True)


# Dependency injection for database sessions
async def get_db() -> AsyncSession:
    """
    FastAPI dependency to provide database sessions.
    
    This function yields an async database session that automatically
    handles commits, rollbacks, and cleanup.
    
    Yields:
        AsyncSession: Database session for the request
    """
    async for session in get_async_session():
        yield session


# Mock authentication dependency (replace with real authentication)
async def get_current_user(
    credentials: HTTPAuthorizationCredentials = Depends(security),
    db: AsyncSession = Depends(get_db)
) -> User:
    """
    Dependency to get the current authenticated user.
    
    In a real implementation, this would validate the JWT token
    and return the corresponding user from the database.
    
    Args:
        credentials: HTTP Authorization header with Bearer token
        db: Database session
        
    Returns:
        User: The authenticated user
        
    Raises:
        HTTPException: If authentication fails
    """
    # Mock implementation - replace with real JWT validation
    token = credentials.credentials
    
    # For demo purposes, assume token is a user_id
    try:
        user_id = int(token)
        result = await db.execute(
            select(User).where(User.user_id == user_id)
        )
        user = result.scalar_one_or_none()
        
        if not user:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Invalid authentication credentials"
            )
        
        return user
    except ValueError:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid token format"
        )


# Dependency to extract organization_id from current user
async def get_current_organization_id(
    current_user: User = Depends(get_current_user)
) -> int:
    """
    Extract organization ID from the current authenticated user.
    
    Args:
        current_user: The authenticated user
        
    Returns:
        int: Organization ID
        
    Raises:
        HTTPException: If user doesn't belong to an organization
    """
    if not current_user.organization_id:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="User must belong to an organization to access this resource"
        )
    
    return current_user.organization_id


# Application lifecycle events
@app.on_event("startup")
async def startup_event():
    """Initialize database on application startup."""
    logger.info("Initializing database...")
    await init_database()
    logger.info("Database initialized successfully")


@app.on_event("shutdown")
async def shutdown_event():
    """Clean up database connections on application shutdown."""
    logger.info("Closing database connections...")
    await close_database()
    logger.info("Database connections closed")


# Health check endpoint
@app.get("/health")
async def health_check():
    """Health check endpoint for load balancers and monitoring."""
    return {"status": "healthy", "timestamp": datetime.utcnow()}


# Organization endpoints
@app.post("/organizations", response_model=OrganizationResponse)
async def create_organization(
    organization_data: OrganizationCreate,
    db: AsyncSession = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """
    Create a new organization.
    
    Only users with appropriate permissions can create organizations.
    In a real implementation, this would check admin privileges.
    """
    # Check if organization name already exists
    result = await db.execute(
        select(Organization).where(Organization.name == organization_data.name)
    )
    existing_org = result.scalar_one_or_none()
    
    if existing_org:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Organization name already exists"
        )
    
    # Create new organization
    new_organization = Organization(
        name=organization_data.name,
        subscription_tier=organization_data.subscription_tier,
        api_key_usage_limit=organization_data.api_key_usage_limit
    )
    
    db.add(new_organization)
    await db.flush()  # Flush to get the ID
    await db.refresh(new_organization)
    
    logger.info(f"Created organization: {new_organization.name} (ID: {new_organization.organization_id})")
    
    return OrganizationResponse.model_validate(new_organization)


@app.get("/organizations/{organization_id}", response_model=OrganizationResponse)
async def get_organization(
    organization_id: int,
    db: AsyncSession = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """
    Get organization details by ID.
    
    Users can only access their own organization's details.
    """
    # Check if user has access to this organization
    if current_user.organization_id != organization_id:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Access denied to this organization"
        )
    
    result = await db.execute(
        select(Organization).where(Organization.organization_id == organization_id)
    )
    organization = result.scalar_one_or_none()
    
    if not organization:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Organization not found"
        )
    
    return OrganizationResponse.model_validate(organization)


# Project endpoints
@app.post("/projects", response_model=ProjectResponse)
async def create_project(
    project_data: ProjectCreate,
    db: AsyncSession = Depends(get_db),
    organization_id: int = Depends(get_current_organization_id)
):
    """
    Create a new project within the user's organization.
    
    The organization_id is automatically extracted from the authenticated user.
    """
    # Check if project name already exists within the organization
    result = await db.execute(
        select(Project).where(
            Project.organization_id == organization_id,
            Project.name == project_data.name
        )
    )
    existing_project = result.scalar_one_or_none()
    
    if existing_project:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Project name already exists in this organization"
        )
    
    # Create new project
    new_project = Project(
        organization_id=organization_id,
        name=project_data.name,
        repository_url=project_data.repository_url,
        last_analyzed_commit=project_data.last_analyzed_commit,
        config_data=project_data.config_data
    )
    
    db.add(new_project)
    await db.flush()
    await db.refresh(new_project)
    
    logger.info(f"Created project: {new_project.name} (ID: {new_project.project_id}) for org {organization_id}")
    
    return ProjectResponse.model_validate(new_project)


@app.get("/organizations/{organization_id}/projects", response_model=List[ProjectResponse])
async def get_organization_projects(
    organization_id: int,
    db: AsyncSession = Depends(get_db),
    current_user: User = Depends(get_current_user),
    limit: int = 100,
    offset: int = 0
):
    """
    Get all projects belonging to a specific organization.
    
    Supports pagination with limit and offset parameters.
    """
    # Check if user has access to this organization
    if current_user.organization_id != organization_id:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Access denied to this organization"
        )
    
    result = await db.execute(
        select(Project)
        .where(Project.organization_id == organization_id)
        .order_by(Project.created_at.desc())
        .limit(limit)
        .offset(offset)
    )
    projects = result.scalars().all()
    
    return [ProjectResponse.model_validate(project) for project in projects]


@app.get("/projects/{project_id}", response_model=ProjectResponse)
async def get_project(
    project_id: int,
    db: AsyncSession = Depends(get_db),
    organization_id: int = Depends(get_current_organization_id)
):
    """
    Get project details by ID.
    
    Users can only access projects within their organization.
    """
    result = await db.execute(
        select(Project).where(
            Project.project_id == project_id,
            Project.organization_id == organization_id
        )
    )
    project = result.scalar_one_or_none()
    
    if not project:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Project not found or access denied"
        )
    
    return ProjectResponse.model_validate(project)


# User management endpoints
@app.get("/users/me", response_model=UserResponse)
async def get_current_user_info(
    current_user: User = Depends(get_current_user)
):
    """Get current user's profile information."""
    return UserResponse.model_validate(current_user)


@app.get("/organizations/{organization_id}/users", response_model=List[UserResponse])
async def get_organization_users(
    organization_id: int,
    db: AsyncSession = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """
    Get all users in an organization.
    
    Only organization admins can access this endpoint.
    """
    # Check if user has access to this organization and is an admin
    if (current_user.organization_id != organization_id or 
        current_user.role not in ['org_admin']):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Only organization administrators can access user lists"
        )
    
    result = await db.execute(
        select(User)
        .where(User.organization_id == organization_id)
        .order_by(User.created_at.desc())
    )
    users = result.scalars().all()
    
    return [UserResponse.model_validate(user) for user in users]


if __name__ == "__main__":
    import uvicorn
    
    # For development only
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
        log_level="info"
    )
