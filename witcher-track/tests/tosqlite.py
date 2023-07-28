import sqlite3
from tqdm import tqdm

con = sqlite3.Connection('f:/movie.db')
con.execute('CREATE TABLE blobs (idx INTEGER NOT NULL, blob BLOB NOT NULL)')

cur = con.cursor()

for i in tqdm(range(1, 21639)):
    with open(f'fixtures/mov/mov{i:06}.png', 'rb') as fp:
        data = fp.read()
    cur.execute('INSERT INTO blobs (idx, blob) VALUES (?, ?)', (i, data))
    if i % 100 == 0:
        con.commit()

con.commit()
