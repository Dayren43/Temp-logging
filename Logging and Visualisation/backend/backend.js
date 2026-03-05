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

/**
 * /data endpoint with pagination and modern aggregation
 */
app.get("/data", async (req, res) => {
  try {
    const { limit, offset, aggregate, range } = req.query;
    
    let params = [];
    let paramIndex = 1;
    let whereClause = '';
    
    // 1. Calculate the start date if a range is provided
function parseDuration(str) {
  const regex = /(\d+)([dhm])/g;
  let ms = 0;
  let match;
  while ((match = regex.exec(str)) !== null) {
    const val = parseInt(match[1]);
    const unit = match[2];
    if (unit === 'd') ms += val * 24 * 60 * 60 * 1000;
    if (unit === 'h') ms += val * 60 * 60 * 1000;
    if (unit === 'm') ms += val * 60 * 1000;
  }
  return ms;
}

// Inside app.get("/data")...
if (range && range !== 'all') {
  const durationMs = parseDuration(range);
  
  // Fallback to 24h if parsing fails
  const finalDuration = durationMs > 0 ? durationMs : (24 * 60 * 60 * 1000);
  
  const startDate = new Date(Date.now() - finalDuration);
  
  whereClause = ` WHERE timestamp >= $${paramIndex}`;
  params.push(startDate.toISOString());
  paramIndex++;
}
    
    let query = '';
    
    // 2. Aggregation Logic using date_bin (Postgres 14+)
    if (aggregate) {
      const validIntervals = {
        '1m': '1 minute',
        '5m': '5 minutes',
        '15m': '15 minutes',
        '1h': '1 hour'
      };
      const interval = validIntervals[aggregate] || '5 minutes';
      
      // date_bin(stride, source, origin)
      // origin '2025-01-01' covers your existing 2025/2026 data perfectly
      query = `
        SELECT 
          date_bin(CAST('${interval}' AS INTERVAL), timestamp, TIMESTAMP '2025-01-01') AS time_bucket,
          ROUND(AVG(temp)::numeric, 2) as temp,
          ROUND(AVG(humid)::numeric, 2) as humid,
          COUNT(*) as data_points
        FROM "environmental_data"
        ${whereClause}
        GROUP BY time_bucket
        ORDER BY time_bucket DESC
      `;
    } else {
      query = `SELECT * FROM "environmental_data"${whereClause} ORDER BY timestamp DESC`;
    }
    
    // 3. Handle pagination
    if (limit) {
      const limitNum = parseInt(limit);
      if (!isNaN(limitNum) && limitNum > 0) {
        query += ` LIMIT $${paramIndex}`;
        params.push(limitNum);
        paramIndex++;
        
        if (offset) {
          const offsetNum = parseInt(offset);
          if (!isNaN(offsetNum) && offsetNum >= 0) {
            query += ` OFFSET $${paramIndex}`;
            params.push(offsetNum);
          }
        }
      }
    }
    
    const result = await pool.query(query, params);
    
    res.json({
      data: result.rows,
      metadata: {
        total_rows: result.rows.length,
        has_aggregation: !!aggregate,
        time_range: range || 'all',
        aggregation_interval: aggregate || 'none'
      }
    });
  } catch (err) {
    console.error("Database query error:", err);
    res.status(500).send("Database error");
  }
});

/**
 * /get endpoint (proxy to sensor)
 */
app.get("/get", async (req, res) => {
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000);

    const response = await fetch("http://sensor.local/get", { signal: controller.signal });
    clearTimeout(timeoutId);

    const data = await response.json();
    res.json(data);
  } catch (error) {
    console.error("Error forwarding request to sensor.local:", error.message);
    res.status(502).send("Bad Gateway: Could not reach sensor");
  }
});

/**
 * /average endpoint with corrected interval logic
 */
app.get("/average", async (req, res) => {
  let { period, start } = req.query;
  
  if (!period || !start) return res.status(400).send("Missing period or start query parameters");
  
  const startDate = new Date(start);
  if (isNaN(startDate)) return res.status(400).send("Invalid start date");

  const intervals = {
    "day": "1 day",
    "week": "7 days",
    "month": "30 days"
  };
  
  const interval = intervals[period];
  if (!interval) return res.status(400).send("Invalid period. Use 'day', 'week', or 'month'.");

  try {
    const query = `
      SELECT 
        ROUND(AVG(temp)::numeric, 2) as avg_temp, 
        ROUND(AVG(humid)::numeric, 2) as avg_humid,
        MIN(temp) as min_temp,
        MAX(temp) as max_temp,
        COUNT(*) as total_readings
      FROM "environmental_data"
      WHERE timestamp >= $1 AND timestamp < ($1::timestamp + INTERVAL '${interval}')
    `;
    const result = await pool.query(query, [startDate.toISOString()]);
    res.json(result.rows[0]); 
  } catch (err) {
    console.error("Database query error:", err);
    res.status(500).send("Database error");
  }
});
  
app.listen(3000, () => console.log(`[${new Date().toISOString()}] Backend running on port 3000`));