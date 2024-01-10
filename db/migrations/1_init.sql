DROP TABLE IF EXISTS addr CASCADE;
CREATE TABLE addr (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    name TEXT,
    comment TEXT,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
DROP INDEX IF EXISTS idx_addr_name_unique;
CREATE UNIQUE INDEX idx_addr_name_unique ON addr(name);
INSERT INTO addr(id, name) VALUES
    ('ffffffffffffffffffffffffffffffff', 'public'),
    ('fffffffffffffffffffffffffffffffe', 'registered'),
    ('00000000000000000000000000000000', 'config'),
    ('000000000000000000000000000000cc', 'comncoin');

DROP TYPE IF EXISTS KEY_ALGOS CASCADE;
CREATE TYPE KEY_ALGOS AS ENUM ('ECDSA');

DROP TABLE IF EXISTS key CASCADE;
CREATE TABLE key (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    pub_key BYTEA NOT NULL,
    algo KEY_ALGOS NOT NULL DEFAULT 'ECDSA',
    comment TEXT,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
DROP INDEX IF EXISTS idx_key_pub_key_unique;
CREATE UNIQUE INDEX idx_key_pub_key_unique ON key(pub_key);

DROP TYPE IF EXISTS ADDR_KEY_TYPE CASCADE;
CREATE TYPE ADDR_KEY_TYPE AS ENUM ('operation', 'recovery');

DROP TABLE IF EXISTS addr_key;
CREATE TABLE addr_key (
    addr_id UUID REFERENCES addr(id) ON DELETE CASCADE ON UPDATE CASCADE,
    key_id UUID REFERENCES key(id) ON DELETE CASCADE ON UPDATE CASCADE,
    key_type ADDR_KEY_TYPE NOT NULL DEFAULT 'operation',
    comment TEXT,
    expires TIMESTAMPTZ,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
DROP INDEX IF EXISTS idx_addr_key_addr_id_key_id_unique;
CREATE UNIQUE INDEX idx_addr_key_addr_id_key_id_unique ON addr_key(addr_id, key_id);

DROP TABLE IF EXISTS crate CASCADE;
CREATE TABLE crate (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    name TEXT NOT NULL,
    comment TEXT,
    expires TIMESTAMPTZ,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO crate(id, name) VALUES
    ('ffffffffffffffffffffffffffffffff', 'public'),
    ('00000000000000000000000000000000', 'config'),
    ('000000000000000000000000000000cc', 'comncoin'),
    ('000000000000000000000000000001cc', '≈6D');


DROP TYPE IF EXISTS ACCESS_TYPE CASCADE;
CREATE TYPE ACCESS_TYPE AS ENUM ('owner', 'admin', 'editor', 'reader', 'writer');


DROP TABLE IF EXISTS item_type CASCADE;
CREATE TABLE item_type (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    description TEXT,
    media_type TEXT NOT NULL,
    extension TEXT NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
DROP INDEX IF EXISTS idx_item_type_media_type;
CREATE INDEX idx_item_type_media_type ON item_type(media_type);

DROP TABLE IF EXISTS item_type_access CASCADE;
CREATE TABLE item_type_access (
    item_type_id uuid REFERENCES item_type(id) ON DELETE CASCADE ON UPDATE CASCADE,
    addr_id uuid REFERENCES addr(id) ON DELETE CASCADE ON UPDATE CASCADE,
    type ACCESS_TYPE NOT NULL DEFAULT 'reader',
    expires TIMESTAMPTZ,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
DROP INDEX IF EXISTS idx_item_type_access_unique;
CREATE UNIQUE INDEX idx_item_type_access_unique ON item_type_access(item_type_id, addr_id);

WITH it AS(
    INSERT INTO item_type(description, media_type, extension) VALUES
    ('Binary Unknown', 'application/octet-stream', ''),
    ('JavaScript Object Notation (JSON)', 'application/json', 'json'),
    ('Portable Document Format (PDF)', 'application/pdf', 'pdf'),
    ('Compressed File (ZIP)', 'application/zip', 'zip'),
    ('Plain Text (TXT)', 'text/plain', 'txt'),
    ('Comma Separated Values Text', 'text/csv', 'csv'),
    ('Hyper Text Markup Language Text', 'text/html', 'html'),
    ('Mpeg Audio', 'audio/mpeg', 'mpeg'),
    ('Web Media Audio', 'audio/webm', 'webm'),
    ('Web Media Video', 'video/webm', 'webm'),
    ('Vorbis Audio', 'audio/vorbis', 'vorbis'),
    ('True Type Font', 'font/ttf', 'ttf'),
    ('Open Type Font', 'font/otf', 'otf'),
    ('Web Open Font Format', 'font/woff', 'woff'),
    ('Web Open Font Format 2', 'font/woff2', 'woff2'),
    ('Joint Photographic Expert Group image (JPEG)', 'image/jpeg', 'jpeg'),
    ('Portable Network Graphics (PNG)', 'image/png', 'png'),
    ('Scalable Vector Graphics (SVG)', 'image/svg+xml', 'svg'),
    ('Animated Portable Network Graphics (APNG)', 'image/apng', 'apng'),
    ('AV1 Image File Format (AVIF)', 'image/avif', 'avif'),
    ('Graphics Interchange Format (GIF)', 'image/gif', 'gif'),
    ('Web Picture format (WEBP)', 'image/webp', 'webp'),
    ('Mpeg4 Video', 'video/mp4', 'mp4') RETURNING id),
ita AS(
    INSERT INTO item_type_access(item_type_id, addr_id)
    SELECT id, 'ffffffffffffffffffffffffffffffff' FROM it
)
SELECT id FROM it;

DROP TABLE IF EXISTS scope CASCADE;
CREATE TABLE scope (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    scope_type TEXT NOT NULL,
    description TEXT,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
DROP INDEX IF EXISTS idx_scope_type;
CREATE INDEX idx_scope_type ON scope(scope_type);

INSERT INTO scope(scope_type) VALUES
    ('login_producer'),
    ('notification'),
    ('system'),
    ('storage'),
    ('transaction_history'),
    ('custom')
    RETURNING id;
INSERT INTO scope(scope_type, description) VALUES
    ('login_approve', 'Accepting the login requested by application.'),
    ('login_reject', 'Rejecting the login requested by application.')
    RETURNING id;


DROP TYPE IF EXISTS CrateItemStorage CASCADE;
CREATE TYPE CrateItemStorage AS ENUM ('Json', 'Text', 'Bytes', 'File');


DROP TABLE IF EXISTS crate_item CASCADE;
CREATE TABLE crate_item (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    added_by UUID NOT NULL REFERENCES addr(id) ON DELETE CASCADE ON UPDATE CASCADE,
    crate_id UUID NOT NULL REFERENCES crate(id) ON DELETE CASCADE ON UPDATE CASCADE,
    scope_id UUID NOT NULL REFERENCES scope(id) ON DELETE CASCADE ON UPDATE CASCADE,
    type_id UUID NOT NULL REFERENCES item_type(id) ON DELETE CASCADE ON UPDATE CASCADE,
    item_path TEXT NOT NULL,
    item_storage CrateItemStorage NOT NULL,
    data_json JSONB,
    data_text TEXT,
    data_bytes BYTEA,
    size_hectobyte INTEGER NOT NULL,
    complete BOOL NOT NULL DEFAULT FALSE,
    expires TIMESTAMPTZ,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
DROP INDEX IF EXISTS idx_crate_item_crate_id;
CREATE INDEX idx_crate_item_crate_id ON crate_item(crate_id);
DROP INDEX IF EXISTS idx_crate_item_crate_id_item_path_unique;
CREATE UNIQUE INDEX idx_crate_item_crate_id_item_path_unique ON crate_item(crate_id, item_path);

INSERT INTO crate_item(id, added_by, crate_id, scope_id, item_path, item_storage, type_id, data_json, data_text, size_hectobyte) VALUES
    ('ffffffffffffffffffffffffffffffff', 
        'ffffffffffffffffffffffffffffffff', 'ffffffffffffffffffffffffffffffff',
        (SELECT id from scope where scope_type = 'system'),
        '/README.txt', 'Text', (SELECT id from item_type where media_type = 'text/plain'), null, 'Readme for Comn Backend', 1),
    ('00000000000000000000000000000000',
        '00000000000000000000000000000000', '00000000000000000000000000000000',
        (SELECT id from scope where scope_type = 'system'),
        '/Config', 'Json', (SELECT id from item_type where media_type = 'application/json'), '{"host_names": ["comn.opus.ai"]
        ,"data_size": {"max": 1000000, "max_chunk": 1000000, "max_db": 20}
        ,"chunks_location": "/tmp"}', null, 1),
    ('000000000000000000000000000001cc',
        '000000000000000000000000000000cc', '000000000000000000000000000000cc',
        (SELECT id from scope where scope_type = 'system'),
        '/ComnCoin', 'Json', (SELECT id from item_type where media_type = 'application/json'),
         '{"total": 100000000000, "00000000-0000-0000-0000-0000000000cc": 100000000000}', null, 1),
    ('000000000000000000000000000000cc',
        '000000000000000000000000000000cc', '000000000000000000000000000000cc',
        (SELECT id from scope where scope_type = 'system'),
        '/Details', 'Json', (SELECT id from item_type where media_type = 'application/json'),
         '{"name": "comncoin","max": 100000000000, "symbol": "cc", "freeze": false
         ,"image":"cc.png"}', null, 1),
    ('000000000000000000000000000011cc',
        '000000000000000000000000000000cc', '000000000000000000000000000000cc',
        (SELECT id from scope where scope_type = 'system'),
        '/nonce', 'Json', (SELECT id from item_type where media_type = 'application/json'),
         '{}', null, 1);


DROP TABLE IF EXISTS crate_item_chunk CASCADE;
CREATE TABLE crate_item_chunk (
    id SMALLINT NOT NULL DEFAULT 0 PRIMARY KEY,
    crate_item_id UUID NOT NULL,
    sha2_hash BYTEA NOT NULL,
    size_hectobyte SMALLINT NOT NULL
);

DROP INDEX IF EXISTS idx_crate_item_chunk_id_crate_item_id_unique;
CREATE UNIQUE INDEX idx_crate_item_chunk_id_crate_item_id_unique ON crate_item_chunk(id, crate_item_id);


DROP TABLE IF EXISTS crate_access CASCADE;
CREATE TABLE crate_access (
    crate_id uuid REFERENCES crate(id) ON DELETE CASCADE ON UPDATE CASCADE,
    addr_id uuid REFERENCES addr(id) ON DELETE CASCADE ON UPDATE CASCADE,
    type ACCESS_TYPE NOT NULL DEFAULT 'reader',
    expires TIMESTAMPTZ,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
DROP INDEX IF EXISTS idx_crate_access_unique;
CREATE UNIQUE INDEX idx_crate_access_unique ON crate_access(crate_id, addr_id, type);

INSERT INTO crate_access(crate_id, addr_id, type) VALUES
    ('ffffffffffffffffffffffffffffffff', 'ffffffffffffffffffffffffffffffff', 'reader'),
    ('00000000000000000000000000000000', '00000000000000000000000000000000', 'owner'),
    ('000000000000000000000000000000cc', '000000000000000000000000000000cc', 'owner'),
    ('000000000000000000000000000000cc', 'fffffffffffffffffffffffffffffffe', 'reader'),
    ('000000000000000000000000000001cc', '000000000000000000000000000000cc', 'owner');
