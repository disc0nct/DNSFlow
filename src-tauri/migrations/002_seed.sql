INSERT INTO dns_servers (name, address, secondary_address, protocol, is_default) VALUES
    ('Google DNS', '8.8.8.8', '8.8.4.4', 'udp', 0),
    ('Cloudflare', '1.1.1.1', '1.0.0.1', 'udp', 1),
    ('Quad9', '9.9.9.9', '149.112.112.112', 'udp', 0);

INSERT INTO config (key, value) VALUES
    ('proxy_port', '5353'),
    ('log_enabled', 'true'),
    ('auto_start', 'false');
