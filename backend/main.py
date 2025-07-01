"""
FastAPI application for Uveddi backend service.

This module sets up the FastAPI application with database dependency injection,
authentication, and CRUD endpoints for organizations and projects.
"""

from typing import List, Optional
from datetime import datetime, timedelta

from fastapi import FastAPI, Depends, HTTPException, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
from sqlalchemy.orm import selectinload
from pydantic import BaseModel, ConfigDict, EmailStr
import logging
import hashlib
import secrets
from jose import jwt, JWTError
from passlib.context import CryptContext

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

# CORS middleware configuration
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:3000",  # React dev server (Create React App)
        "http://localhost:5173",  # Vite dev server default
        "http://localhost:7777",  # Our custom frontend server
        "http://localhost:9999",  # New frontend port
        "http://127.0.0.1:3000",
        "http://127.0.0.1:5173",
        "http://127.0.0.1:7777",
        "http://127.0.0.1:9999",
        # Add production domains here when deploying
    ],
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    allow_headers=["*"],
)

# Security configuration
SECRET_KEY = "your-secret-key-change-in-production"  # Change this in production!
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30

# Password hashing
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

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


# Authentication models
class UserCreate(BaseModel):
    """Schema for user registration."""
    email: EmailStr
    username: str
    password: str

    model_config = ConfigDict(from_attributes=True)


class UserLogin(BaseModel):
    """Schema for user login."""
    email: EmailStr
    password: str

    model_config = ConfigDict(from_attributes=True)


class Token(BaseModel):
    """Schema for JWT token response."""
    access_token: str
    token_type: str
    user: UserResponse

    model_config = ConfigDict(from_attributes=True)


# Authentication helper functions
def verify_password(plain_password: str, hashed_password: str) -> bool:
    """Verify a password against its hash."""
    return pwd_context.verify(plain_password, hashed_password)


def get_password_hash(password: str) -> str:
    """Hash a password."""
    return pwd_context.hash(password)


def create_access_token(data: dict, expires_delta: Optional[timedelta] = None):
    """Create a JWT access token."""
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt


async def authenticate_user(db: AsyncSession, email: str, password: str) -> Optional[User]:
    """Authenticate a user by email and password."""
    result = await db.execute(select(User).where(User.email == email))
    user = result.scalar_one_or_none()
    
    if not user or not verify_password(password, user.password_hash):
        return None
    return user


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


# JWT token authentication dependency
async def get_current_user(
    credentials: HTTPAuthorizationCredentials = Depends(security),
    db: AsyncSession = Depends(get_db)
) -> User:
    """
    Dependency to get the current authenticated user from JWT token.
    
    Args:
        credentials: HTTP Authorization header with Bearer token
        db: Database session
        
    Returns:
        User: The authenticated user
        
    Raises:
        HTTPException: If authentication fails
    """
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )
    
    try:
        token = credentials.credentials
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        email: str = payload.get("sub")
        if email is None:
            raise credentials_exception
    except JWTError:
        raise credentials_exception
    
    result = await db.execute(select(User).where(User.email == email))
    user = result.scalar_one_or_none()
    
    if user is None:
        raise credentials_exception
    
    return user


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


# Authentication endpoints
@app.post("/auth/register", response_model=Token)
async def register_user(
    user_data: UserCreate,
    db: AsyncSession = Depends(get_db)
):
    """
    Register a new user account.
    
    Creates a new user with hashed password and returns a JWT token.
    """
    # Check if user already exists
    result = await db.execute(
        select(User).where(User.email == user_data.email)
    )
    existing_user = result.scalar_one_or_none()
    
    if existing_user:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Email already registered"
        )
    
    # Check if username already exists
    result = await db.execute(
        select(User).where(User.username == user_data.username)
    )
    existing_username = result.scalar_one_or_none()
    
    if existing_username:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Username already taken"
        )
    
    # Create new user
    hashed_password = get_password_hash(user_data.password)
    new_user = User(
        email=user_data.email,
        username=user_data.username,
        password_hash=hashed_password,
        role="individual"
    )
    
    db.add(new_user)
    await db.flush()
    await db.refresh(new_user)
    
    # Create access token
    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = create_access_token(
        data={"sub": new_user.email}, expires_delta=access_token_expires
    )
    
    logger.info(f"New user registered: {new_user.email} (ID: {new_user.user_id})")
    
    return Token(
        access_token=access_token,
        token_type="bearer",
        user=UserResponse.model_validate(new_user)
    )


@app.post("/auth/login", response_model=Token)
async def login_user(
    user_credentials: UserLogin,
    db: AsyncSession = Depends(get_db)
):
    """
    Authenticate user and return JWT token.
    
    Validates email/password and returns access token on success.
    """
    user = await authenticate_user(db, user_credentials.email, user_credentials.password)
    
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Incorrect email or password",
            headers={"WWW-Authenticate": "Bearer"},
        )
    
    # Update last login time
    user.last_login = datetime.utcnow()
    await db.flush()
    
    # Create access token
    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = create_access_token(
        data={"sub": user.email}, expires_delta=access_token_expires
    )
    
    logger.info(f"User logged in: {user.email}")
    
    return Token(
        access_token=access_token,
        token_type="bearer",
        user=UserResponse.model_validate(user)
    )


@app.post("/auth/logout")
async def logout_user():
    """
    Logout endpoint.
    
    In a stateless JWT implementation, logout is handled client-side
    by simply discarding the token. This endpoint is provided for
    compatibility and potential future token blacklisting.
    """
    return {"message": "Successfully logged out"}


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
