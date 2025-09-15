import time
import requests
import psycopg2
import os
import logging

# ----------------------
# Logging configuration
# ----------------------
logging.basicConfig(
    level=logging.INFO,  # default level
    format="%(asctime)s [%(levelname)s] %(message)s",
)
logger = logging.getLogger()

# Reduce non-error messages to DEBUG
logger.setLevel(logging.DEBUG)

# ----------------------
# Environment variables
# ----------------------
DB_HOST = os.getenv("DB_HOST")
DB_NAME = os.getenv("POSTGRES_DB")
DB_USER = os.getenv("POSTGRES_USER")
DB_PASS = os.getenv("POSTGRES_PASSWORD")
SENSOR_URL = os.getenv("SENSOR_URL", "http://sensor.local/get")
INTERVAL = int(os.getenv("INTERVAL", "300"))  # 5 minutes

# ----------------------
# Polling function
# ----------------------
def poll_and_insert():
    try:
        resp = requests.get(SENSOR_URL, timeout=10)
        data = resp.json()
        temp = data.get("Temp")
        humid = data.get("Humid")
        if temp is not None and humid is not None:
            conn = psycopg2.connect(
                host=DB_HOST,
                dbname=DB_NAME,
                user=DB_USER,
                password=DB_PASS,
                port=5432
            )
            cur = conn.cursor()
            cur.execute(
                "INSERT INTO environmental_data (temp, humid) VALUES (%s, %s)",
                (temp, humid),
            )
            conn.commit()
            cur.close()
            conn.close()

            # Normal info messages at DEBUG level
            logger.debug(f"Inserted: Temp={temp}, Humid={humid}")
        else:
            logger.warning(f"Invalid data from sensor: {data}")
    except Exception as e:
        logger.error("Error during polling/inserting", exc_info=True)

# ----------------------
# Main loop
# ----------------------
if __name__ == "__main__":
    while True:
        logger.debug("Polling sensor...")
        poll_and_insert()
        time.sleep(INTERVAL)