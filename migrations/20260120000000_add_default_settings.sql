-- Add default settings for key TTL and cron schedule
INSERT OR IGNORE INTO setting (key, value) VALUES ('TTL_KEY_WARNING', '90');
INSERT OR IGNORE INTO setting (key, value) VALUES ('TTL_KEY_LIMIT', '365');
INSERT OR IGNORE INTO setting (key, value) VALUES ('CRON_SYNC', '0 0 0 * * *');
