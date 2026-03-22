"""Export GitHub DuckDB data to CSV for the Flight SQL server."""
import duckdb
conn = duckdb.connect("config/seed/github.duckdb", read_only=True)
conn.execute("COPY github_repos TO 'config/seed/github_repos.csv' (HEADER, DELIMITER ',')")
count = conn.execute("SELECT COUNT(*) FROM github_repos").fetchone()[0]
conn.close()
print(f"Exported {count} repos to config/seed/github_repos.csv")
