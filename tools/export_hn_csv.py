"""Export HN DuckDB data to CSV for the Flight SQL server."""
import duckdb

conn = duckdb.connect("config/seed/hn.duckdb", read_only=True)
conn.execute("COPY hn_stories TO 'config/seed/hn_stories.csv' (HEADER, DELIMITER ',')")
count = conn.execute("SELECT COUNT(*) FROM hn_stories").fetchone()[0]
conn.close()
print(f"Exported {count} stories to config/seed/hn_stories.csv")
