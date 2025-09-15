import express from "express";
import cors from "cors";
import pkg from "pg";

const { Pool } = pkg;
const app = express();
app.use(cors());

// Simple request logging middleware
app.use((req, res, next) => {
  console.log(`[${new Date().toISOString()}] ${req.method} ${req.url}`);
  next();
});

const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
});

// /data endpoint
app.get("/data", async (req, res) => {
  try {
    console.log("Querying environmental_data...");
    const result = await pool.query('SELECT * FROM "environmental_data"');
    console.log(`Returned ${result.rows.length} rows`);
    res.json(result.rows);
  } catch (err) {
    console.error("Database query error:", err);
    res.status(500).send("Database error");
  }
});

// /get endpoint (proxy to sensor)
app.get("/get", async (req, res) => {
  try {
    console.log("Forwarding request to sensor.local/get");
    const response = await fetch("http://sensor.local/get");
    const data = await response.json();
    console.log("Received data from sensor.local:", data);
    res.json(data);
  } catch (error) {
    console.error("Error forwarding request to sensor.local:", error);
    res.status(500).send("Error fetching data from sensor.local");
  }
});

// Start server
app.listen(3000, () => console.log(`[${new Date().toISOString()}] Backend running on port 3000`));
