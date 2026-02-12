-- Packages and Docker management schema

-- Installed packages table
CREATE TABLE IF NOT EXISTS installed_packages (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    package_type TEXT NOT NULL CHECK(package_type IN ('binary', 'docker', 'service')),
    manifest_url TEXT,
    manifest_data TEXT, -- Cached manifest JSON
    status TEXT NOT NULL DEFAULT 'installed' CHECK(status IN ('installing', 'installed', 'updating', 'removing', 'error')),
    error_message TEXT,
    installed_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Package files (tracks files created during installation)
CREATE TABLE IF NOT EXISTS package_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id TEXT NOT NULL REFERENCES installed_packages(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    file_type TEXT NOT NULL CHECK(file_type IN ('binary', 'config', 'data', 'symlink', 'service')),
    created_at TEXT NOT NULL
);

-- Docker containers managed by PiNAS
CREATE TABLE IF NOT EXISTS docker_containers (
    id TEXT PRIMARY KEY NOT NULL, -- Docker container ID
    package_id TEXT REFERENCES installed_packages(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    image TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'created',
    config TEXT, -- JSON configuration used to create container
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Package installation tasks (for progress tracking)
CREATE TABLE IF NOT EXISTS package_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    package_id TEXT NOT NULL,
    task_type TEXT NOT NULL CHECK(task_type IN ('install', 'update', 'uninstall')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'running', 'completed', 'failed')),
    progress INTEGER DEFAULT 0,
    total_steps INTEGER DEFAULT 0,
    current_step TEXT,
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_package_files_package_id ON package_files(package_id);
CREATE INDEX IF NOT EXISTS idx_docker_containers_package_id ON docker_containers(package_id);
CREATE INDEX IF NOT EXISTS idx_package_tasks_package_id ON package_tasks(package_id);
CREATE INDEX IF NOT EXISTS idx_package_tasks_status ON package_tasks(status);
