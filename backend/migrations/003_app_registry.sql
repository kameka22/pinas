-- App registry schema for frontend integration

-- Add frontend configuration to installed packages
ALTER TABLE installed_packages ADD COLUMN frontend_config TEXT; -- JSON FrontendConfig
ALTER TABLE installed_packages ADD COLUMN has_window INTEGER DEFAULT 0;

-- App translations table (stores per-app translations)
CREATE TABLE IF NOT EXISTS app_translations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id TEXT NOT NULL REFERENCES installed_packages(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    translations TEXT NOT NULL, -- JSON object with translation keys
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(package_id, locale)
);

-- Create index for fast locale lookups
CREATE INDEX IF NOT EXISTS idx_app_translations_package_locale ON app_translations(package_id, locale);
