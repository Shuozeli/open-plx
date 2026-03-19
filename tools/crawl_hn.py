#!/usr/bin/env python3
"""
Crawl Hacker News stories via Algolia Search API and load into DuckDB.

Fetches notable stories (> min_points) from the last year by chunking
requests by month to avoid Algolia's 1000-result cap.

Usage:
    uv run --with httpx --with duckdb tools/crawl_hn.py [--db PATH] [--min-points N] [--months N]
"""

import asyncio
import argparse
import time
from datetime import datetime, timezone, timedelta
from urllib.parse import urlparse

import httpx
import duckdb


API_BASE = "https://hn.algolia.com/api/v1/search_by_date"
HITS_PER_PAGE = 1000
MAX_PAGES_PER_CHUNK = 10


async def fetch_page(
    client: httpx.AsyncClient,
    created_after: int,
    created_before: int,
    min_points: int,
    page: int,
) -> list[dict]:
    """Fetch one page of results from Algolia."""
    params = {
        "tags": "story",
        "numericFilters": f"created_at_i>{created_after},created_at_i<{created_before},points>{min_points}",
        "hitsPerPage": HITS_PER_PAGE,
        "page": page,
    }

    for attempt in range(3):
        try:
            resp = await client.get(API_BASE, params=params)
            resp.raise_for_status()
            data = resp.json()
            return data.get("hits", [])
        except (httpx.HTTPStatusError, httpx.ReadTimeout) as e:
            if attempt < 2:
                await asyncio.sleep(2 ** attempt)
            else:
                print(f"  FAILED page {page}: {e}")
                return []


async def crawl_month_chunk(
    client: httpx.AsyncClient,
    month_start: datetime,
    month_end: datetime,
    min_points: int,
) -> list[dict]:
    """Crawl stories for one month chunk."""
    created_after = int(month_start.timestamp())
    created_before = int(month_end.timestamp())
    label = month_start.strftime("%Y-%m")

    # Get total count for this chunk
    params = {
        "tags": "story",
        "numericFilters": f"created_at_i>{created_after},created_at_i<{created_before},points>{min_points}",
        "hitsPerPage": 1,
    }
    resp = await client.get(API_BASE, params=params)
    resp.raise_for_status()
    data = resp.json()
    total = data["nbHits"]
    pages = min(data["nbPages"], MAX_PAGES_PER_CHUNK)

    if total == 0:
        return []

    hits: list[dict] = []
    for page in range(pages):
        page_hits = await fetch_page(client, created_after, created_before, min_points, page)
        hits.extend(page_hits)
        await asyncio.sleep(0.3)

    print(f"  {label}: {len(hits)} stories (of {total} total)")
    return hits


async def crawl_stories(months: int, min_points: int) -> list[dict]:
    """Crawl stories by month chunks to bypass the 1000-result cap."""
    now = datetime.now(timezone.utc)
    all_hits: list[dict] = []

    async with httpx.AsyncClient(timeout=30) as client:
        for i in range(months):
            month_end = now - timedelta(days=30 * i)
            month_start = now - timedelta(days=30 * (i + 1))

            hits = await crawl_month_chunk(client, month_start, month_end, min_points)
            all_hits.extend(hits)

            await asyncio.sleep(0.5)

    print(f"Crawled {len(all_hits)} stories total")
    return all_hits


def extract_domain(url: str | None) -> str:
    if not url:
        return ""
    try:
        parsed = urlparse(url)
        domain = parsed.netloc
        if domain.startswith("www."):
            domain = domain[4:]
        return domain
    except Exception:
        return ""


def load_into_duckdb(stories: list[dict], db_path: str) -> None:
    conn = duckdb.connect(db_path)
    conn.execute("BEGIN TRANSACTION")

    conn.execute("DROP TABLE IF EXISTS hn_stories")
    conn.execute("""
        CREATE TABLE hn_stories (
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
        )
    """)

    weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
    rows = []
    seen_ids: set[int] = set()

    for story in stories:
        story_id = int(story.get("objectID", 0))
        if story_id == 0 or story_id in seen_ids:
            continue
        seen_ids.add(story_id)

        title = story.get("title") or ""
        url = story.get("url") or ""
        author = story.get("author") or ""
        points = story.get("points") or 0
        num_comments = story.get("num_comments") or 0
        created_at_i = story.get("created_at_i") or 0

        dt = datetime.fromtimestamp(created_at_i, tz=timezone.utc)
        title_lower = title.lower()
        if title_lower.startswith("ask hn"):
            story_type = "Ask HN"
        elif title_lower.startswith("show hn"):
            story_type = "Show HN"
        elif title_lower.startswith("tell hn"):
            story_type = "Tell HN"
        elif not url:
            story_type = "Text"
        else:
            story_type = "Link"

        rows.append((
            story_id, title, url, extract_domain(url), author, points, num_comments,
            dt.strftime("%Y-%m-%d %H:%M:%S"), dt.strftime("%Y-%m-%d"),
            dt.strftime("%Y-%m"), weekdays[dt.weekday()], dt.hour, story_type,
        ))

    conn.executemany(
        "INSERT INTO hn_stories VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rows,
    )

    count = conn.execute("SELECT COUNT(*) FROM hn_stories").fetchone()[0]
    conn.execute("COMMIT")
    conn.close()
    print(f"Loaded {count} unique stories into {db_path}")


async def main() -> None:
    parser = argparse.ArgumentParser(description="Crawl HN stories into DuckDB")
    parser.add_argument("--db", default="config/seed/hn.duckdb", help="DuckDB file path")
    parser.add_argument("--min-points", type=int, default=50, help="Minimum points threshold")
    parser.add_argument("--months", type=int, default=12, help="Months of history")
    args = parser.parse_args()

    print(f"Crawling HN stories from last {args.months} months with > {args.min_points} points")
    start = time.monotonic()

    stories = await crawl_stories(args.months, args.min_points)
    load_into_duckdb(stories, args.db)

    print(f"Done in {time.monotonic() - start:.1f}s")


if __name__ == "__main__":
    asyncio.run(main())
