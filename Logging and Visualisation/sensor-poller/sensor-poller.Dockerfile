# Use Python slim image
FROM python:3.11-slim

# Faster, cleaner Python in containers: no .pyc files, unbuffered stdout for logs
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

WORKDIR /app

# psycopg2-binary ships prebuilt wheels — no gcc/libpq-dev build deps needed.
# Installing deps before copying the script keeps this layer cached across code edits.
RUN pip install --no-cache-dir requests psycopg2-binary

# Copy the script
COPY poll_sensor.py .

# Run as a non-root user
RUN useradd -l --create-home --shell /usr/sbin/nologin appuser \
    && chown appuser /app
USER appuser

# Print something on container start for visibility
CMD ["python", "-u", "poll_sensor.py"]
