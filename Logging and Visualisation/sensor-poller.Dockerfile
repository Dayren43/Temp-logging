FROM python:3.11-slim

WORKDIR /app
COPY poll_sensor.py .

RUN pip install requests psycopg2-binary

CMD ["python", "poll_sensor.py"]