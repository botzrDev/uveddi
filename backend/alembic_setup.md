# Alembic initialization and migration setup for CodeAtlas backend

## 1. Initialize Alembic

First, initialize Alembic in your backend directory:

```bash
cd backend
alembic init alembic
```

## 2. Configure alembic.ini

Update the `alembic.ini` file to use your database URL:

```ini
# alembic.ini
[alembic]
script_location = alembic
prepend_sys_path = .
version_path_separator = os
sqlalchemy.url = postgresql://codeatlas:codeatlas@localhost:5432/codeatlas

# For environment variable configuration:
# sqlalchemy.url = 

[post_write_hooks]
hooks = black
black.type = console_scripts
black.entrypoint = black
black.options = -l 88 REVISION_SCRIPT_FILENAME

[loggers]
keys = root,sqlalchemy,alembic

[handlers]
keys = console

[formatters]
keys = generic

[logger_root]
level = WARN
handlers = console
qualname =

[logger_sqlalchemy]
level = WARN
handlers =
qualname = sqlalchemy.engine

[logger_alembic]
level = INFO
handlers =
qualname = alembic

[handler_console]
class = StreamHandler
args = (sys.stderr,)
level = NOTSET
formatter = generic

[formatter_generic]
format = %(levelname)-5.5s [%(name)s] %(message)s
datefmt = %H:%M:%S
```

## 3. Configure env.py

Update `alembic/env.py` to use your models and configuration:
