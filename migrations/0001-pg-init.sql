-- Initialize database tables

-- Configuration table
CREATE TABLE configuration (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL,
    value TEXT,
    "order" INTEGER,
    CONSTRAINT configuration_key_unique UNIQUE (key)
);

-- Proxy table
CREATE TABLE proxy (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    protocol TEXT,
    nameserver JSONB,
    configuration JSONB,
    set TEXT,
    create_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT proxy_name_unique UNIQUE (name)
);

CREATE INDEX idx_proxy_set ON proxy(set);

-- Rule table
CREATE TABLE rule (
    id SERIAL PRIMARY KEY,
    mode TEXT NOT NULL,
    value TEXT NOT NULL,
    set TEXT,
    create_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_rule_mode ON rule(mode);
CREATE INDEX idx_rule_set ON rule(set);

-- Match table
CREATE TABLE match (
    id SERIAL PRIMARY KEY,
    proxy_set TEXT NOT NULL,
    rule_set TEXT NOT NULL,
    create_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_match_proxy_set ON match(proxy_set);
CREATE INDEX idx_match_rule_set ON match(rule_set);

-- DNS Record table
CREATE TABLE record (
    id SERIAL PRIMARY KEY,
    domain TEXT NOT NULL,
    type TEXT NOT NULL,
    record TEXT NOT NULL,
    ttl INTEGER,
    create_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_record_domain ON record(domain);
CREATE INDEX idx_record_type ON record(type);

-- Host table
CREATE TABLE host (
    id SERIAL PRIMARY KEY,
    mac_addr TEXT,
    name TEXT,
    hostname TEXT,
    create_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_host_mac_addr ON host(mac_addr);
CREATE INDEX idx_host_name ON host(name);

-- Host IP table
CREATE TABLE host_ip (
    id SERIAL PRIMARY KEY,
    host_id TEXT,
    ip TEXT,
    create_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_host_ip_host_id ON host_ip(host_id);
CREATE INDEX idx_host_ip_ip ON host_ip(ip);

-- Add triggers for update_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_proxy_updated_at
    BEFORE UPDATE ON proxy
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rule_updated_at
    BEFORE UPDATE ON rule
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_match_updated_at
    BEFORE UPDATE ON match
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_record_updated_at
    BEFORE UPDATE ON record
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_host_updated_at
    BEFORE UPDATE ON host
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_host_ip_updated_at
    BEFORE UPDATE ON host_ip
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column(); 