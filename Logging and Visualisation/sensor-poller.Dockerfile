# Use Python slim image
FROM python:3.11-slim

WORKDIR /app

# Copy the script
COPY poll_sensor.py .

# Install system dependencies for psycopg2
RUN apt-get update && \
    apt-get install -y gcc libpq-dev && \
    rm -rf /var/lib/apt/lists/*

# Install Python dependencies
RUN pip install --no-cache-dir requests psycopg2-binary

# Print something on container start for visibility
CMD ["python", "-u", "poll_sensor.py"]