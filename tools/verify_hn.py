import duckdb
conn = duckdb.connect('config/seed/hn.duckdb', read_only=True)
print('Row count:', conn.execute('SELECT COUNT(*) FROM hn_stories').fetchone()[0])
print()
print('Sample (top 5 by points):')
for row in conn.execute('SELECT title, points, author, domain, created_month, story_type FROM hn_stories ORDER BY points DESC LIMIT 5').fetchall():
    print(f'  {row[1]:>5} pts | {row[2]:<15} | {row[4]} | {row[5]:<8} | {row[3][:25]:<25} | {row[0][:55]}')
print()
print('By type:')
for row in conn.execute('SELECT story_type, COUNT(*), AVG(points)::INT FROM hn_stories GROUP BY story_type ORDER BY COUNT(*) DESC').fetchall():
    print(f'  {row[0]:<10} {row[1]:>5} stories  avg {row[2]} pts')
print()
print('By month:')
for row in conn.execute('SELECT created_month, COUNT(*) FROM hn_stories GROUP BY created_month ORDER BY created_month').fetchall():
    print(f'  {row[0]}: {row[1]} stories')
print()
print('Top 10 domains:')
for row in conn.execute("SELECT domain, COUNT(*) as c FROM hn_stories WHERE domain != '' GROUP BY domain ORDER BY c DESC LIMIT 10").fetchall():
    print(f'  {row[0]:<30} {row[1]:>4}')
conn.close()
