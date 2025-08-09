# =========================
# Stage 1 — Build environment
# =========================
FROM python:3.12-slim-bookworm AS builder

# Copy uv from its latest image into this stage
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

# Set working directory for the app
WORKDIR /app

# ----------------------------------
# Step 1: Install ONLY production dependencies
# ----------------------------------
COPY uv.lock pyproject.toml ./

RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --locked --no-install-project --no-editable --no-dev

# ----------------------------------
# Step 2: Copy project source and install it into the venv
# ----------------------------------
COPY . .

RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --locked --no-editable --no-dev

# =========================
# Stage 2 — Runtime image
# =========================
FROM python:3.12-slim

# Create a non-root user for security
RUN useradd -m app

# Set working directory
WORKDIR /app


# Finetune python envvars
ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1


# Install PostgreSQL client libs
RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*

# Copy only the virtual environment from builder (keeps image small)
COPY --from=builder --chown=app:app /app/.venv /app/.venv

# Copy only application source code
COPY --from=builder --chown=app:app /app /app

# Put venv first in PATH so "python" and "gunicorn" use it
ENV PATH="/app/.venv/bin:$PATH"

# Switch to non-root user
USER app

# Expose the production port
EXPOSE 8000

#  -w 4 → 4 worker processes (tweak for your CPU cores)
#  -b 0.0.0.0:8000 → bind to all interfaces on port 8000
#  --access-logfile - → log to stdout
CMD ["gunicorn", "app.wsgi:application", "-w", "4", "-b", "0.0.0.0:8000", "--access-logfile", "-"]

