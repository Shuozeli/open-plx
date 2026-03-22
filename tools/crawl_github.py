#!/usr/bin/env python3
"""
Crawl GitHub trending repositories and load into DuckDB.

Fetches top repos by stars that were recently active, collecting
language, stars, forks, issues, watchers, topics, and creation dates.

Usage:
    uv run --with httpx --with duckdb tools/crawl_github.py [--db PATH] [--count N]
"""

import asyncio
import argparse
import time
from datetime import datetime, timezone

import httpx
import duckdb


SEARCH_URL = "https://api.github.com/search/repositories"


async def fetch_repos(count: int) -> list[dict]:
    """Fetch top repos via GitHub Search API, paginated."""
    all_repos: list[dict] = []
    per_page = min(count, 100)
    pages = (count + per_page - 1) // per_page

    async with httpx.AsyncClient(timeout=30) as client:
        for page in range(1, pages + 1):
            params = {
                "q": "stars:>500 pushed:>2025-01-01",
                "sort": "stars",
                "order": "desc",
                "per_page": per_page,
                "page": page,
            }
            resp = await client.get(SEARCH_URL, params=params)
            if resp.status_code == 403:
                print(f"  rate limited at page {page}, stopping")
                break
            resp.raise_for_status()
            data = resp.json()
            items = data.get("items", [])
            all_repos.extend(items)
            print(f"  page {page}: {len(items)} repos (total {len(all_repos)})")

            if len(items) < per_page:
                break
            await asyncio.sleep(2)  # respect rate limits

    return all_repos[:count]


def load_into_duckdb(repos: list[dict], db_path: str) -> None:
    conn = duckdb.connect(db_path)
    conn.execute("BEGIN TRANSACTION")

    conn.execute("DROP TABLE IF EXISTS github_repos")
    conn.execute("""
        CREATE TABLE github_repos (
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
        )
    """)

    rows = []
    seen: set[str] = set()
    for repo in repos:
        full_name = repo.get("full_name", "")
        if not full_name or full_name in seen:
            continue
        seen.add(full_name)

        created = repo.get("created_at", "")[:10]
        updated = repo.get("updated_at", "")[:10]
        topics = ", ".join(repo.get("topics", [])[:5])
        license_name = ""
        if repo.get("license") and repo["license"].get("spdx_id"):
            license_name = repo["license"]["spdx_id"]

        rows.append((
            repo.get("name", ""),
            full_name,
            (repo.get("description") or "")[:200],
            repo.get("language") or "Other",
            repo.get("stargazers_count", 0),
            repo.get("forks_count", 0),
            repo.get("open_issues_count", 0),
            repo.get("watchers_count", 0),
            repo.get("size", 0),
            created,
            updated,
            topics,
            license_name,
            repo.get("fork", False),
            repo.get("owner", {}).get("login", ""),
        ))

    conn.executemany(
        "INSERT INTO github_repos VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rows,
    )

    count = conn.execute("SELECT COUNT(*) FROM github_repos").fetchone()[0]
    conn.execute("COMMIT")
    conn.close()
    print(f"Loaded {count} repos into {db_path}")


async def main() -> None:
    parser = argparse.ArgumentParser(description="Crawl GitHub repos into DuckDB")
    parser.add_argument("--db", default="config/seed/github.duckdb", help="DuckDB file path")
    parser.add_argument("--count", type=int, default=500, help="Number of repos to fetch")
    args = parser.parse_args()

    print(f"Crawling top {args.count} GitHub repos")
    start = time.monotonic()

    repos = await fetch_repos(args.count)
    load_into_duckdb(repos, args.db)

    print(f"Done in {time.monotonic() - start:.1f}s")


if __name__ == "__main__":
    asyncio.run(main())
