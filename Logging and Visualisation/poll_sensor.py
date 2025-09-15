import time
import requests
import psycopg2
import os

DB_HOST = os.getenv("DB_HOST")
DB_NAME = os.getenv("DB_NAME")
DB_USER = os.getenv("DB_USER")
DB_PASS = os.getenv("DB_PASS")
SENSOR_URL = os.getenv("SENSOR_URL", "http://sensor.local/get")
INTERVAL = int(os.getenv("INTERVAL", "300"))  # 5 minutes

def poll_and_insert():
    try:
        resp = requests.get(SENSOR_URL, timeout=10)
        data = resp.json()
        temp = data.get("Temp")
        humid = data.get("Humid")
        if temp is not None and humid is not None:
            conn = psycopg2.connect(
                host=DB_HOST, dbname=DB_NAME, user=DB_USER, password=DB_PASS
            )
            cur = conn.cursor()
            cur.execute(
                "INSERT INTO environmental_data (temp, humid) VALUES (%s, %s)",
                (temp, humid),
            )
            conn.commit()
            cur.close()
            conn.close()
            print(f"Inserted: Temp={temp}, Humid={humid}")
        else:
            print("Invalid data from sensor:", data)
    except Exception as e:
        print("Error:", e)

if __name__ == "__main__":
    while True:
        poll_and_insert()
        time.sleep(INTERVAL)