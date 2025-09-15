import express from "express";
import cors from "cors";
import pkg from "pg";

const { Pool } = pkg;
const app = express();
app.use(cors());

const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
});

app.get("/data", async (req, res) => {
  const result = await pool.query('SELECT * FROM "environmental_data"');
  res.json(result.rows);
});

app.listen(3000, () => console.log("Backend running on port 3000"));

app.get("/get", async (req, res) => {
  try {
    const response = await fetch("http://sensor.local/get");
    const data = await response.json();
    res.json(data);
  } catch (error) {
    console.error("Error forwarding request to sensor.local:", error);
    res.status(500).send("Error fetching data from sensor.local");
  }
});
