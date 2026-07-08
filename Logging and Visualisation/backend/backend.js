import express from "express";
import cors from "cors";
import pkg from "pg";

const { Pool } = pkg;
const app = express();
app.use(cors());
app.use(express.json());

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
 * /monthly endpoint — per-month aggregates for the last N months
 */
app.get("/monthly", async (req, res) => {
  const months = Math.min(parseInt(req.query.months) || 12, 60);
  try {
    const query = `
      SELECT
        date_trunc('month', timestamp) AS month,
        ROUND(AVG(temp)::numeric, 2)   AS avg_temp,
        ROUND(MIN(temp)::numeric, 2)   AS min_temp,
        ROUND(MAX(temp)::numeric, 2)   AS max_temp,
        ROUND(AVG(humid)::numeric, 1)  AS avg_humid,
        COUNT(*)                       AS data_points,
        ROUND(
          100.0 * COUNT(*) FILTER (
            WHERE temp >= 20 AND temp <= 24
              AND humid >= 35 AND humid <= 60
          ) / NULLIF(COUNT(*), 0)
        , 1) AS comfort_pct
      FROM "environmental_data"
      WHERE timestamp >= date_trunc('month', NOW()) - ($1 - 1) * INTERVAL '1 month'
      GROUP BY month
      ORDER BY month ASC
    `;
    const result = await pool.query(query, [months]);
    res.json({ data: result.rows, months });
  } catch (err) {
    console.error("Monthly query error:", err);
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
  
/**
 * Home Assistant AC control
 *
 * The frontend never talks to Home Assistant directly — it goes through here so
 * the long-lived token stays server-side (same reasoning as the DB creds) and
 * we avoid CORS. Configure via .env:
 *   HA_URL       e.g. http://homeassistant.local:8123
 *   HA_TOKEN     Long-Lived Access Token (HA → profile → Security)
 *   AC_ENTITY_ID e.g. climate.living_room
 */
const HA_URL = (process.env.HA_URL || "").replace(/\/$/, "");
const HA_TOKEN = process.env.HA_TOKEN || "";
const AC_ENTITY_ID = process.env.AC_ENTITY_ID || "";
const HA_CONFIGURED = Boolean(HA_URL && HA_TOKEN && AC_ENTITY_ID);

async function haFetch(path, options = {}) {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 5000);
  try {
    return await fetch(`${HA_URL}${path}`, {
      ...options,
      signal: controller.signal,
      headers: {
        Authorization: `Bearer ${HA_TOKEN}`,
        "Content-Type": "application/json",
        ...(options.headers || {}),
      },
    });
  } finally {
    clearTimeout(timeoutId);
  }
}

// Call a HA service, e.g. callService("climate", "set_temperature", { temperature: 21 })
function callService(domain, service, data = {}) {
  return haFetch(`/api/services/${domain}/${service}`, {
    method: "POST",
    body: JSON.stringify({ entity_id: AC_ENTITY_ID, ...data }),
  });
}

// Reduce HA's verbose state object down to what the card needs.
function shapeState(s) {
  const a = s.attributes || {};
  return {
    entity_id: s.entity_id,
    mode: s.state,                          // "off" | "cool" | "heat" | "auto" | ...
    on: s.state !== "off" && s.state !== "unavailable",
    available: s.state !== "unavailable",
    hvac_action: a.hvac_action ?? null,     // "cooling" | "idle" | "off" | ...
    current_temp: a.current_temperature ?? null,
    target_temp: a.temperature ?? null,
    min_temp: a.min_temp ?? 16,
    max_temp: a.max_temp ?? 30,
    step: a.target_temp_step ?? 0.5,
    modes: a.hvac_modes ?? [],
    name: a.friendly_name ?? s.entity_id,
  };
}

/**
 * GET /ac — current AC state (shaped for the card)
 */
app.get("/ac", async (req, res) => {
  if (!HA_CONFIGURED) {
    return res.status(503).json({ error: "not_configured" });
  }
  try {
    const r = await haFetch(`/api/states/${AC_ENTITY_ID}`);
    if (!r.ok) {
      console.error("HA state error:", r.status);
      return res.status(502).json({ error: "ha_error", status: r.status });
    }
    res.json(shapeState(await r.json()));
  } catch (err) {
    console.error("HA /ac error:", err.message);
    res.status(502).json({ error: "unreachable" });
  }
});

/**
 * POST /ac/power  { on: boolean }
 */
app.post("/ac/power", async (req, res) => {
  if (!HA_CONFIGURED) return res.status(503).json({ error: "not_configured" });
  const on = Boolean(req.body?.on);
  try {
    const r = await callService("climate", on ? "turn_on" : "turn_off");
    if (!r.ok) return res.status(502).json({ error: "ha_error", status: r.status });
    res.json({ ok: true, on });
  } catch (err) {
    console.error("HA /ac/power error:", err.message);
    res.status(502).json({ error: "unreachable" });
  }
});

/**
 * POST /ac/temp  { temperature: number }
 */
app.post("/ac/temp", async (req, res) => {
  if (!HA_CONFIGURED) return res.status(503).json({ error: "not_configured" });
  const temperature = Number(req.body?.temperature);
  if (!Number.isFinite(temperature)) {
    return res.status(400).json({ error: "invalid_temperature" });
  }
  try {
    const r = await callService("climate", "set_temperature", { temperature });
    if (!r.ok) return res.status(502).json({ error: "ha_error", status: r.status });
    res.json({ ok: true, temperature });
  } catch (err) {
    console.error("HA /ac/temp error:", err.message);
    res.status(502).json({ error: "unreachable" });
  }
});

app.listen(3000, () => console.log(`[${new Date().toISOString()}] Backend running on port 3000`));