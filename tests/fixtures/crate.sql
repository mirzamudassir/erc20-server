WITH a AS (INSERT INTO addr(id) VALUES('00000000-0000-0000-0000-00000000000a'::uuid) RETURNING id),
k AS (INSERT INTO key(pub_key) VALUES('\x0279b2f72735c1ffb42532a01c3b063b4e051295cf0cfa4c82479f44faea1d7fd4') RETURNING id),
ak AS (INSERT INTO addr_key(addr_id, key_id) VALUES((SELECT id from a), (SELECT id from k)) RETURNING key_id),
c AS (INSERT INTO crate(id, name) VALUES ('10000000000000000000000000000000', 'test_crate') RETURNING id),
c2 AS (INSERT INTO crate(id, name) VALUES ('20000000000000000000000000000000', 'test_crate') RETURNING id),
c3 AS (INSERT INTO crate(id, name) VALUES ('30000000000000000000000000000000', 'test_crate_3') RETURNING id),
c4 AS (INSERT INTO crate(id, name) VALUES ('40000000000000000000000000000000', 'test_crate_4') RETURNING id),
ca AS (INSERT INTO crate_access(crate_id, addr_id) VALUES ((SELECT id from c), (SELECT id from a)) RETURNING *),
ca3 AS (INSERT INTO crate_access(crate_id, addr_id, type) VALUES ((SELECT id from c2), (SELECT id from a), 'owner') RETURNING *),
ca4 AS (INSERT INTO crate_access(crate_id, addr_id) VALUES ((SELECT id from c4), 'fffffffffffffffffffffffffffffffe') RETURNING *)
SELECT key_id from ak;
