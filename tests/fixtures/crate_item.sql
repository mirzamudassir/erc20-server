BEGIN;
WITH k AS (INSERT INTO key(id, pub_key) VALUES
	('00000000000000000000000000000001', '\x03ba2c0e05c00185b2a793ee99476789572c558c532c62ffbed46e53b2b9a237ab'::bytea),
	('00000000000000000000000000000002', '\x02478a9a811b32520794f6fc8f6794dae10d63c6e8e63478c8c7bb4f4110806ba6'::bytea),
	('00000000000000000000000000000003', '\x0267bbd7cb74c1c0690f2fcd7c237342e6c5b9a249f6a7c3e9078eb66cd2487285'::bytea)

RETURNING id),
a AS (INSERT INTO addr(id, name) VALUES
	('0000000000000000000000000000000a', 'addr1'),
	('0000000000000000000000000000000b', 'addr2'),
	('0000000000000000000000000000000c', 'addr3'),
	('0000000000000000000000000000000d', 'addr4')
RETURNING id),
ak AS (INSERT INTO addr_key(addr_id, key_id) VALUES
	('0000000000000000000000000000000a', '00000000000000000000000000000001'),
	('0000000000000000000000000000000b', '00000000000000000000000000000002'),
	('0000000000000000000000000000000c', '00000000000000000000000000000001'),
	('0000000000000000000000000000000d', '00000000000000000000000000000002'),
	('0000000000000000000000000000000a', '00000000000000000000000000000003')
RETURNING key_id),
b AS (INSERT INTO crate(id, name) VALUES
	('10000000000000000000000000000000', 'b1 anon read, 0a owner'),
	('20000000000000000000000000000000', 'b2 0b reader'),
	('30000000000000000000000000000000', 'b3 0c editor')
RETURNING id),
bi AS (INSERT INTO crate_item(id, added_by, crate_id, scope_id, item_path, item_storage, data_text, type_id, size_hectobyte) VALUES
	('a0000000000000000000000000000000', '0000000000000000000000000000000a', '10000000000000000000000000000000', (SELECT id from scope where scope_type = 'storage'),
		'/', 'Text', 'this is text string 1 for test_crate\nyes it is!\nb1 anon read, 0a owner', (SELECT id from item_type where media_type = 'text/plain'), 1),
	('b0000000000000000000000000000000', '0000000000000000000000000000000d', '20000000000000000000000000000000', (SELECT id from scope where scope_type = 'storage'),
		'/', 'Text', 'this is a text string for rest_cucket\nres it is!\nb2 0b reader', (SELECT id from item_type where media_type = 'text/plain'), 1),
	('c0000000000000000000000000000000', '0000000000000000000000000000000c', '30000000000000000000000000000000', (SELECT id from scope where scope_type = 'storage'),
		'/nested', 'Text', 'this is a text string for jest_hucket\nmes it is!\nb3 0c editor', (SELECT id from item_type where media_type = 'text/plain'), 1),
	('d0000000000000000000000000000000', '0000000000000000000000000000000d', '20000000000000000000000000000000', (SELECT id from scope where scope_type = 'storage'),
		'/bested/nested', 'Text', 'this is a text string for rest_cucket\nyes it is!\nb2 0b reader', (SELECT id from item_type where media_type = 'text/plain'), 1),
	('e0000000000000000000000000000000', '0000000000000000000000000000000a', '10000000000000000000000000000000', (SELECT id from scope where scope_type = 'storage'),
		'/lested', 'Text', 'this is anon text string for test_crate\nyes it is!\nb1 anon read, 0a owner', (SELECT id from item_type where media_type = 'text/plain'), 1)

RETURNING id),
ba AS (INSERT INTO crate_access(crate_id, addr_id, type) VALUES
	('10000000000000000000000000000000', '0000000000000000000000000000000a', 'owner'),
	('20000000000000000000000000000000', '0000000000000000000000000000000b', 'reader'),
	('30000000000000000000000000000000', '0000000000000000000000000000000c', 'editor'),
	('10000000000000000000000000000000', 'ffffffffffffffffffffffffffffffff', 'reader'),
	('20000000000000000000000000000000', 'fffffffffffffffffffffffffffffffe', 'writer')
)
SELECT id from bi;
COMMIT;
