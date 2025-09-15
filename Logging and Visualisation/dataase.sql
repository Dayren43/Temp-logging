CREATE TABLE environmental_data (
    id SERIAL PRIMARY KEY,
    temp DOUBLE PRECISION NOT NULL,
    humid DOUBLE PRECISION NOT NULL,
    perceived_temp DOUBLE PRECISION
);