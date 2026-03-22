-- Company financials test data for dark launch validation.
CREATE TABLE IF NOT EXISTS company_financials (
    company VARCHAR NOT NULL,
    quarter VARCHAR NOT NULL,
    revenue DOUBLE NOT NULL,
    profit DOUBLE NOT NULL,
    eps DOUBLE NOT NULL,
    market_cap DOUBLE NOT NULL
);

INSERT INTO company_financials VALUES
    ('AAPL', 'Q1', 94.8, 25.0, 1.53, 2840),
    ('AAPL', 'Q2', 81.8, 19.4, 1.26, 2900),
    ('AAPL', 'Q3', 89.5, 22.9, 1.46, 3100),
    ('AAPL', 'Q4', 119.6, 33.9, 2.18, 3400),
    ('TSLA', 'Q1', 21.3, 2.5, 0.73, 560),
    ('TSLA', 'Q2', 24.9, 2.3, 0.78, 620),
    ('TSLA', 'Q3', 25.2, 1.9, 0.58, 680),
    ('TSLA', 'Q4', 25.2, 7.9, 2.29, 790),
    ('AVGO', 'Q1', 8.9, 3.7, 2.83, 550),
    ('AVGO', 'Q2', 8.7, 3.5, 2.65, 580),
    ('AVGO', 'Q3', 9.3, 3.8, 2.93, 640),
    ('AVGO', 'Q4', 14.1, 5.3, 4.11, 780),
    ('AMZN', 'Q1', 143.3, 10.4, 0.98, 1870),
    ('AMZN', 'Q2', 148.0, 13.5, 1.26, 1950),
    ('AMZN', 'Q3', 158.9, 15.3, 1.43, 2020),
    ('AMZN', 'Q4', 170.0, 20.0, 1.86, 2200);

-- Hacker News stories (12 months, > 50 points)
CREATE TABLE IF NOT EXISTS hn_stories (
    id BIGINT PRIMARY KEY,
    title VARCHAR NOT NULL,
    url VARCHAR,
    domain VARCHAR,
    author VARCHAR NOT NULL,
    points INTEGER NOT NULL,
    num_comments INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    created_date DATE NOT NULL,
    created_month VARCHAR NOT NULL,
    created_weekday VARCHAR NOT NULL,
    created_hour INTEGER NOT NULL,
    story_type VARCHAR NOT NULL
);

COPY hn_stories FROM '/opt/flight_sql/seed/hn_stories.csv' (HEADER, DELIMITER ',');

-- GitHub repos (top 500 by stars, recently active)
CREATE TABLE IF NOT EXISTS github_repos (
    name VARCHAR NOT NULL,
    full_name VARCHAR NOT NULL PRIMARY KEY,
    description VARCHAR,
    language VARCHAR,
    stars INTEGER NOT NULL,
    forks INTEGER NOT NULL,
    open_issues INTEGER NOT NULL,
    watchers INTEGER NOT NULL,
    size_kb INTEGER NOT NULL,
    created_date DATE NOT NULL,
    updated_date DATE NOT NULL,
    topics VARCHAR,
    license VARCHAR,
    is_fork BOOLEAN NOT NULL,
    owner VARCHAR NOT NULL
);

COPY github_repos FROM '/opt/flight_sql/seed/github_repos.csv' (HEADER, DELIMITER ',');
