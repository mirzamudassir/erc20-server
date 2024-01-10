WITH a AS (SELECT a.id from addr a WHERE a.id = '000000000000000000000000000000cc'::uuid),
k AS (INSERT INTO key(pub_key) VALUES('\x0279b2f72735c1ffb42532a01c3b063b4e051295cf0cfa4c82479f44faea1d7fd4') RETURNING id),
ak AS (INSERT INTO addr_key(addr_id, key_id) VALUES((SELECT id from a), (SELECT id from k)) RETURNING key_id),
a_receiver AS (INSERT INTO addr(id, name) VALUES('00000000-0000-0000-0000-00000000000a'::uuid, 'onion') RETURNING id),
k_receiver AS (INSERT INTO key(pub_key) VALUES('\x03ba2c0e05c00185b2a793ee99476789572c558c532c62ffbed46e53b2b9a237ab'::bytea) RETURNING id),
ak_receiver AS (INSERT INTO addr_key(addr_id, key_id) VALUES((SELECT id from a_receiver), (SELECT id from k_receiver)) RETURNING key_id)

SELECT key_id from ak;