# Uveddi Backend Troubleshooting Guide

This document contains solutions for common issues you might encounter when setting up and running the Uveddi backend.

## Docker Compose Issues

### Port 5432 Already in Use

**Problem:** When starting the Docker containers, you see an error like:
```
Error response from daemon: failed to set up container networking: driver failed programming external connectivity on endpoint backend-postgres-1: failed to bind host port for 0.0.0.0:5432:5432/tcp: address already in use
```

**Solution:** 
1. The Docker Compose configuration has been updated to use port 5433 instead of 5432 to avoid conflicts with existing PostgreSQL installations.
2. If you're still having issues, check if something else is using port 5433:
   ```bash
   sudo lsof -i :5433
   ```
3. You can modify the port in `docker-compose.yml` to use a different port if needed:
   ```yaml
   ports:
     - "5434:5432"  # Change 5434 to any available port
   ```
4. Don't forget to update your `.env` file to match the new port.

### Container Fails to Start

**Problem:** The PostgreSQL container fails to start or doesn't become healthy.

**Solution:**
1. Check the container logs:
   ```bash
   docker-compose logs postgres
   ```
2. Make sure the volume permissions are correct:
   ```bash
   sudo chown -R $USER:$USER backend/postgres_data
   ```
3. Try removing the containers and volumes, then start again:
   ```bash
   docker-compose down -v
   docker-compose up -d
   ```

## Database Migration Issues

### Alembic Migration Failures

**Problem:** Alembic migrations fail to apply.

**Solution:**
1. Check the Alembic logs:
   ```bash
   docker-compose exec backend alembic current
   ```
2. Make sure the database is accessible:
   ```bash
   docker-compose exec postgres psql -U uveddi -d uveddi -c "\l"
   ```
3. Try running migrations with more verbose output:
   ```bash
   docker-compose exec backend alembic upgrade head --sql
   ```

### Connection Issues

**Problem:** The backend can't connect to the database.

**Solution:**
1. Inside Docker, use `postgres` as the hostname (not localhost)
2. For local development outside Docker, use `localhost` with port `5433`
3. Verify connection parameters in `.env`
4. Test the connection:
   ```bash
   # Inside Docker:
   docker-compose exec backend python -c "import psycopg2; conn = psycopg2.connect(dbname='uveddi', user='uveddi', password='uveddi', host='postgres', port=5432); print('Connection successful!')"
   
   # From host machine:
   python -c "import psycopg2; conn = psycopg2.connect(dbname='uveddi', user='uveddi', password='uveddi', host='localhost', port=5433); print('Connection successful!')"
   ```

## FastAPI Backend Issues

### Backend Fails to Start

**Problem:** The FastAPI backend fails to start.

**Solution:**
1. Check if all required packages are installed:
   ```bash
   pip install -r requirements.txt
   ```
2. Verify the database connection:
   ```bash
   python test_database.py
   ```
3. Try running with more verbose output:
   ```bash
   uvicorn main:app --host 0.0.0.0 --port 8000 --reload --log-level debug
   ```

### API Endpoints Not Working

**Problem:** API endpoints return errors.

**Solution:**
1. Check the API documentation at http://localhost:8000/docs
2. Verify that the database migrations have been applied:
   ```bash
   alembic current
   ```
3. Check for validation errors in the API request
4. Look for error logs in the backend container:
   ```bash
   docker-compose logs backend
   ```
