#!/usr/bin/env python3
"""
Convert Umbrel app format to PiNAS manifest.json

Usage:
    python3 convert-umbrel.py <umbrel-app-dir> <output-dir> [--catalog <catalog.json>]

Example:
    python3 convert-umbrel.py umbrel-apps/jellyfin app-catalog/apps/jellyfin
"""

import argparse
import json
import os
import re
import sys
import yaml


# MDI icon mapping for known apps
ICON_MAP = {
    "jellyfin": "mdi:filmstrip",
    "nextcloud": "mdi:cloud",
    "syncthing": "mdi:sync",
    "pi-hole": "mdi:shield-check",
    "pihole": "mdi:shield-check",
    "transmission": "mdi:download",
    "vaultwarden": "mdi:shield-key",
    "filebrowser": "mdi:folder",
    "file-browser": "mdi:folder",
    "photoprism": "mdi:image-multiple",
    "plex": "mdi:plex",
    "qbittorrent": "mdi:download-circle",
    "emby": "mdi:play-network",
    "sonarr": "mdi:television-classic",
    "radarr": "mdi:filmstrip-box",
    "lidarr": "mdi:music",
    "sabnzbd": "mdi:download-box",
    "adguard-home": "mdi:shield-half-full",
    "wireguard": "mdi:vpn",
    "uptime-kuma": "mdi:monitor-dashboard",
    "grafana": "mdi:chart-line",
    "nginx-proxy-manager": "mdi:web",
    "home-assistant": "mdi:home-assistant",
    "homeassistant": "mdi:home-assistant",
    "node-red": "mdi:sitemap",
    "paperless-ngx": "mdi:file-document-multiple",
    "duplicati": "mdi:backup-restore",
    "code-server": "mdi:microsoft-visual-studio-code",
    "immich": "mdi:image-album",
    "gitea": "mdi:git",
    "mealie": "mdi:food-apple",
    "n8n": "mdi:workflow",
    "paperless": "mdi:file-document-multiple",
}

# Gradient mapping for known apps
GRADIENT_MAP = {
    "jellyfin": "from-purple-500 to-indigo-600",
    "nextcloud": "from-blue-500 to-blue-700",
    "syncthing": "from-blue-400 to-cyan-500",
    "pi-hole": "from-red-500 to-rose-600",
    "pihole": "from-red-500 to-rose-600",
    "transmission": "from-red-600 to-red-800",
    "vaultwarden": "from-blue-600 to-indigo-700",
    "filebrowser": "from-teal-500 to-emerald-600",
    "file-browser": "from-teal-500 to-emerald-600",
    "photoprism": "from-amber-500 to-orange-600",
    "plex": "from-amber-500 to-yellow-600",
    "qbittorrent": "from-blue-600 to-blue-800",
    "emby": "from-green-500 to-emerald-600",
    "sonarr": "from-sky-500 to-blue-600",
    "radarr": "from-amber-500 to-yellow-600",
    "lidarr": "from-green-500 to-lime-600",
    "sabnzbd": "from-yellow-500 to-amber-600",
    "adguard-home": "from-green-600 to-teal-700",
    "wireguard": "from-red-500 to-pink-600",
    "uptime-kuma": "from-green-500 to-lime-600",
    "grafana": "from-orange-500 to-amber-600",
    "nginx-proxy-manager": "from-green-600 to-emerald-700",
    "home-assistant": "from-sky-500 to-blue-600",
    "homeassistant": "from-sky-500 to-blue-600",
    "node-red": "from-red-500 to-red-700",
    "paperless-ngx": "from-green-600 to-teal-700",
    "duplicati": "from-blue-500 to-indigo-600",
    "code-server": "from-blue-600 to-blue-800",
    "immich": "from-indigo-500 to-purple-600",
    "gitea": "from-green-600 to-emerald-700",
    "mealie": "from-green-500 to-lime-600",
    "n8n": "from-orange-500 to-red-500",
    "paperless": "from-green-600 to-teal-700",
}

# French description overrides (for popular apps)
FR_DESCRIPTIONS = {
    "jellyfin": "Serveur multimédia libre pour diffuser vos films, séries et musique",
    "nextcloud": "Plateforme de productivité auto-hébergée et synchronisation de fichiers",
    "syncthing": "Synchronisation continue de fichiers entre appareils",
    "pihole": "Blocage de publicités au niveau réseau",
    "pi-hole": "Blocage de publicités au niveau réseau",
    "transmission": "Client BitTorrent léger et efficace",
    "vaultwarden": "Gestionnaire de mots de passe compatible Bitwarden",
    "filebrowser": "Gestionnaire de fichiers web avec interface moderne",
    "file-browser": "Gestionnaire de fichiers web avec interface moderne",
    "photoprism": "Galerie photo auto-hébergée avec IA",
    "plex": "Serveur multimédia pour diffuser vos films, séries et musique",
    "qbittorrent": "Client BitTorrent puissant avec interface web",
    "emby": "Serveur multimédia personnel",
    "sonarr": "Gestionnaire automatique de séries TV",
    "radarr": "Gestionnaire automatique de films",
    "lidarr": "Gestionnaire automatique de musique",
    "sabnzbd": "Téléchargeur Usenet automatisé",
    "adguard-home": "Serveur DNS anti-publicités et anti-trackers",
    "wireguard": "VPN rapide et moderne",
    "uptime-kuma": "Surveillance de disponibilité des services",
    "grafana": "Plateforme de visualisation et monitoring",
    "nginx-proxy-manager": "Gestionnaire de proxy inverse avec certificats SSL",
    "home-assistant": "Plateforme domotique open source",
    "homeassistant": "Plateforme domotique open source",
    "node-red": "Outil de programmation visuelle pour l'IoT",
    "paperless-ngx": "Système de gestion de documents dématérialisés",
    "duplicati": "Sauvegarde chiffrée dans le cloud",
    "code-server": "VS Code dans le navigateur",
    "immich": "Solution de gestion de photos auto-hébergée",
    "gitea": "Service Git léger auto-hébergé",
    "mealie": "Gestionnaire de recettes auto-hébergé",
    "n8n": "Automatisation de workflows",
    "paperless": "Système de gestion de documents dématérialisés",
}

# Umbrel category → PiNAS category mapping
CATEGORY_MAP = {
    "media": "media",
    "files": "utilities",
    "finance": "utilities",
    "networking": "network",
    "social": "utilities",
    "automation": "utilities",
    "developer-tools": "utilities",
    "security": "network",
    "ai": "utilities",
}


class UmbrelConverter:
    """Converts an Umbrel app to a PiNAS manifest."""

    def __init__(self, umbrel_dir: str):
        self.umbrel_dir = umbrel_dir
        self.app_yml = None
        self.compose = None

    def load(self):
        """Load umbrel-app.yml and docker-compose.yml."""
        app_yml_path = os.path.join(self.umbrel_dir, "umbrel-app.yml")
        compose_path = os.path.join(self.umbrel_dir, "docker-compose.yml")

        if not os.path.exists(app_yml_path):
            raise FileNotFoundError(f"umbrel-app.yml not found in {self.umbrel_dir}")
        if not os.path.exists(compose_path):
            raise FileNotFoundError(f"docker-compose.yml not found in {self.umbrel_dir}")

        with open(app_yml_path, "r") as f:
            self.app_yml = yaml.safe_load(f)

        with open(compose_path, "r") as f:
            self.compose = yaml.safe_load(f)

    def get_services(self):
        """Get docker-compose services, filtering out app_proxy."""
        services = self.compose.get("services", {})
        # Filter out Umbrel-specific proxy service
        return {k: v for k, v in services.items() if k != "app_proxy"}

    def is_multi_service(self):
        """Check if app has multiple services."""
        return len(self.get_services()) > 1

    def map_variables(self, value: str) -> str:
        """Map Umbrel variables to PiNAS variables."""
        if not isinstance(value, str):
            return str(value)
        result = value
        result = result.replace("${APP_DATA_DIR}", "${APP_DATA_DIR}")
        result = result.replace("${DEVICE_HOSTNAME}", "${DEVICE_HOSTNAME}")
        result = result.replace("${APP_PASSWORD}", "${APP_PASSWORD}")
        # Remove Umbrel-specific variables that don't have PiNAS equivalents
        result = re.sub(r'\$\{APP_SEED\}', 'pinas-default-seed', result)
        result = re.sub(r'\$\{APP_HIDDEN_SERVICE\}', '', result)
        result = re.sub(r'\$\{APP_DOMAIN\}', '${DEVICE_HOSTNAME}', result)
        # Map Umbrel storage paths to PiNAS equivalents
        result = re.sub(r'\$\{UMBREL_ROOT\}/data/storage', '${DATA_DIR}/storage', result)
        result = re.sub(r'\$\{UMBREL_ROOT\}', '${DATA_DIR}', result)
        # Remove remaining Umbrel-specific per-app variables (APP_*_PORT, NETWORK_IP, etc.)
        # These are dynamically generated by Umbrel and have no PiNAS equivalent
        result = re.sub(r'\$\{NETWORK_IP\}', '0.0.0.0', result)
        # Strip any remaining ${APP_*} variables not already mapped (leave as-is if APP_DATA_DIR/APP_PASSWORD)
        result = re.sub(r'\$\{APP_(?!DATA_DIR|PASSWORD)[A-Z_]+\}', '', result)
        return result

    def parse_ports(self, service: dict) -> list:
        """Parse ports from a compose service."""
        ports = []
        for port_def in service.get("ports", []):
            port_str = str(port_def)
            # Handle "host:container/protocol" format
            match = re.match(r'(\d+):(\d+)(?:/(\w+))?', port_str)
            if match:
                host_port = int(match.group(1))
                container_port = int(match.group(2))
                protocol = match.group(3) or "tcp"
                ports.append({
                    "host": host_port,
                    "container": container_port,
                    "protocol": protocol,
                })
        return ports

    def parse_volumes(self, service: dict, app_id: str) -> tuple:
        """Parse volumes from a compose service. Returns (volume_mappings, mkdir_paths)."""
        volumes = []
        mkdirs = []

        for vol in service.get("volumes", []):
            vol_str = str(vol) if not isinstance(vol, dict) else None
            if vol_str is None:
                continue

            # Parse "host:container[:ro]" format
            parts = vol_str.split(":")
            if len(parts) >= 2:
                host_path = self.map_variables(parts[0])
                container_path = parts[1]
                readonly = len(parts) > 2 and parts[2] == "ro"

                # Convert ${APP_DATA_DIR}/... paths to ${APP_DATA_DIR}/...
                volumes.append({
                    "host": host_path,
                    "container": container_path,
                    "readonly": readonly,
                })

                # Create mkdir step for data directories
                if "${APP_DATA_DIR}" in host_path or "${DATA_DIR}" in host_path:
                    mkdirs.append(host_path)

        return volumes, mkdirs

    def parse_environment(self, service: dict) -> list:
        """Parse environment variables from a compose service."""
        env = service.get("environment", {})
        env_list = []

        if isinstance(env, dict):
            for k, v in env.items():
                env_list.append({
                    "name": k,
                    "value": self.map_variables(str(v) if v is not None else ""),
                })
        elif isinstance(env, list):
            for item in env:
                if "=" in str(item):
                    k, v = str(item).split("=", 1)
                    env_list.append({
                        "name": k,
                        "value": self.map_variables(v),
                    })

        return env_list

    @staticmethod
    def strip_image_digest(image: str) -> str:
        """Strip @sha256:... digest from image reference (arch-specific)."""
        return re.sub(r'@sha256:[a-f0-9]+', '', image)

    def convert_single_service(self, app_id: str, service_name: str, service: dict) -> dict:
        """Convert a single-service app to PiNAS install steps."""
        image = self.strip_image_digest(service.get("image", ""))
        ports = self.parse_ports(service)
        volumes, mkdirs = self.parse_volumes(service, app_id)
        environment = self.parse_environment(service)
        restart = service.get("restart", "unless-stopped")
        network_mode = service.get("network_mode", None)
        cap_add = service.get("cap_add", [])
        cap_drop = service.get("cap_drop", [])
        privileged = service.get("privileged", False)
        user = service.get("user", None)
        command = service.get("command", None)
        entrypoint = service.get("entrypoint", None)
        devices = service.get("devices", [])
        dns = service.get("dns", [])
        extra_hosts = service.get("extra_hosts", [])
        tmpfs = service.get("tmpfs", [])

        if isinstance(dns, str):
            dns = [dns]
        if isinstance(tmpfs, str):
            tmpfs = [tmpfs]
        if isinstance(extra_hosts, str):
            extra_hosts = [extra_hosts]

        # Parse command to list if string
        if isinstance(command, str):
            command = command.split()
        if isinstance(entrypoint, str):
            entrypoint = [entrypoint]

        steps = []

        # Create data directories
        for mkdir_path in mkdirs:
            steps.append({"action": "mkdir", "path": mkdir_path})

        # Pull image
        steps.append({"action": "docker_pull", "image": image})

        # Create container config
        container_config = {
            "name": app_id,
            "hostname": app_id,
            "image": image,
            "restart": restart,
            "ports": ports if network_mode != "host" else [],
            "volumes": volumes,
            "environment": environment,
            "labels": {
                "com.pinas.managed": "true",
                "com.pinas.app": app_id,
            },
        }

        if network_mode:
            container_config["network"] = network_mode
        if cap_add:
            container_config["cap_add"] = cap_add
        if cap_drop:
            container_config["cap_drop"] = cap_drop
        if privileged:
            container_config["privileged"] = True
        if user:
            container_config["user"] = str(user)
        if command:
            container_config["command"] = command
        if entrypoint:
            container_config["entrypoint"] = entrypoint
        if devices:
            container_config["devices"] = devices
        if dns:
            container_config["dns"] = dns
        if extra_hosts:
            container_config["extra_hosts"] = extra_hosts
        if tmpfs:
            container_config["tmpfs"] = tmpfs

        steps.append({"action": "docker_create", "config": container_config})
        steps.append({"action": "docker_start", "container": app_id})

        # Uninstall steps
        uninstall_steps = [
            {"action": "docker_stop", "container": app_id},
            {"action": "docker_rm", "container": app_id},
        ]

        return {
            "type": "docker",
            "image": image,
            "container": container_config,
            "steps": steps,
        }, {
            "steps": uninstall_steps,
        }

    def convert_compose_content(self, app_id: str) -> str:
        """Generate a cleaned docker-compose.yml content for multi-service apps."""
        services = self.get_services()

        # Build a clean compose dict
        compose = {"services": {}}

        for svc_name, svc in services.items():
            clean_svc = {}

            if "image" in svc:
                clean_svc["image"] = self.strip_image_digest(svc["image"])
            if "build" in svc:
                continue  # Skip build-only services

            if "restart" in svc:
                clean_svc["restart"] = svc["restart"]
            if "ports" in svc:
                clean_svc["ports"] = svc["ports"]
            if "volumes" in svc:
                # Map variables in volumes
                clean_volumes = []
                for v in svc["volumes"]:
                    if isinstance(v, str):
                        clean_volumes.append(self.map_variables(v))
                    else:
                        clean_volumes.append(v)
                clean_svc["volumes"] = clean_volumes
            if "environment" in svc:
                env = svc["environment"]
                if isinstance(env, dict):
                    clean_svc["environment"] = {
                        k: self.map_variables(str(v) if v is not None else "")
                        for k, v in env.items()
                    }
                elif isinstance(env, list):
                    clean_svc["environment"] = [
                        self.map_variables(str(e)) for e in env
                    ]
            if "depends_on" in svc:
                deps = svc["depends_on"]
                # Filter out app_proxy
                if isinstance(deps, list):
                    deps = [d for d in deps if d != "app_proxy"]
                elif isinstance(deps, dict):
                    deps = {k: v for k, v in deps.items() if k != "app_proxy"}
                if deps:
                    clean_svc["depends_on"] = deps
            if "network_mode" in svc:
                clean_svc["network_mode"] = svc["network_mode"]
            if "cap_add" in svc:
                clean_svc["cap_add"] = svc["cap_add"]
            if "cap_drop" in svc:
                clean_svc["cap_drop"] = svc["cap_drop"]
            if "privileged" in svc:
                clean_svc["privileged"] = svc["privileged"]
            if "user" in svc:
                clean_svc["user"] = svc["user"]
            if "command" in svc:
                clean_svc["command"] = svc["command"]
            if "entrypoint" in svc:
                clean_svc["entrypoint"] = svc["entrypoint"]
            if "devices" in svc:
                clean_svc["devices"] = svc["devices"]
            if "dns" in svc:
                clean_svc["dns"] = svc["dns"]
            if "extra_hosts" in svc:
                clean_svc["extra_hosts"] = svc["extra_hosts"]
            if "tmpfs" in svc:
                clean_svc["tmpfs"] = svc["tmpfs"]
            if "healthcheck" in svc:
                clean_svc["healthcheck"] = svc["healthcheck"]
            if "labels" in svc:
                clean_svc["labels"] = svc["labels"]
            if "stop_grace_period" in svc:
                clean_svc["stop_grace_period"] = svc["stop_grace_period"]
            if "shm_size" in svc:
                clean_svc["shm_size"] = svc["shm_size"]
            if "security_opt" in svc:
                clean_svc["security_opt"] = svc["security_opt"]

            compose["services"][svc_name] = clean_svc

        # Copy networks and volumes if present
        if "networks" in self.compose:
            compose["networks"] = self.compose["networks"]
        orig_volumes = self.compose.get("volumes", {})
        if orig_volumes:
            compose["volumes"] = orig_volumes

        return yaml.dump(compose, default_flow_style=False, sort_keys=False)

    def convert_multi_service(self, app_id: str) -> tuple:
        """Convert a multi-service app using ComposeUp."""
        compose_content = self.convert_compose_content(app_id)
        compose_dest = f"${{APP_DATA_DIR}}/docker-compose.yml"

        # Collect mkdir paths from all services
        mkdirs = set()
        for svc_name, svc in self.get_services().items():
            _, svc_mkdirs = self.parse_volumes(svc, app_id)
            mkdirs.update(svc_mkdirs)

        steps = []
        for path in sorted(mkdirs):
            steps.append({"action": "mkdir", "path": path})

        steps.append({
            "action": "compose_up",
            "dest": compose_dest,
            "content": compose_content,
            "project_name": app_id,
        })

        uninstall_steps = [
            {
                "action": "compose_down",
                "compose_file": compose_dest,
                "remove_volumes": False,
                "project_name": app_id,
            },
        ]

        # Determine main image for metadata
        services = self.get_services()
        first_svc = next(iter(services.values()))
        main_image = self.strip_image_digest(first_svc.get("image", ""))

        return {
            "type": "docker",
            "image": main_image,
            "steps": steps,
        }, {
            "steps": uninstall_steps,
        }

    def get_web_port_and_path(self) -> tuple:
        """Get the web port and path from umbrel-app.yml."""
        port = self.app_yml.get("port")
        path = self.app_yml.get("path", "/")
        return port, path

    def build_frontend_config(self, app_id: str) -> dict | None:
        """Build frontend configuration."""
        port, path = self.get_web_port_and_path()

        if not port:
            # No web UI - use ServiceApp
            return {
                "component": "ServiceApp",
                "icon": ICON_MAP.get(app_id, "mdi:application"),
                "gradient": GRADIENT_MAP.get(app_id, "from-gray-500 to-gray-700"),
                "window": {
                    "width": 900,
                    "height": 600,
                    "min_width": 600,
                    "min_height": 400,
                },
                "config": {},
                "i18n": self.build_i18n(app_id),
            }

        return {
            "component": "IframeApp",
            "icon": ICON_MAP.get(app_id, "mdi:application"),
            "gradient": GRADIENT_MAP.get(app_id, "from-gray-500 to-gray-700"),
            "window": {
                "width": 1200,
                "height": 800,
                "min_width": 900,
                "min_height": 600,
            },
            "config": {
                "port": port,
                "path": path,
                "title": self.app_yml.get("name", app_id),
            },
            "i18n": self.build_i18n(app_id),
        }

    def build_i18n(self, app_id: str) -> dict:
        """Build i18n section."""
        en_desc = self.app_yml.get("tagline", "")
        fr_desc = FR_DESCRIPTIONS.get(app_id, en_desc)

        return {
            "en": {"description": en_desc},
            "fr": {"description": fr_desc},
        }

    def convert(self) -> dict:
        """Convert the Umbrel app to a PiNAS manifest."""
        self.load()

        app_id = self.app_yml.get("id", "")
        name = self.app_yml.get("name", app_id)
        version = self.app_yml.get("version", "1.0.0")
        tagline = self.app_yml.get("tagline", "")
        developer = self.app_yml.get("developer", "")
        website = self.app_yml.get("website", "")
        fr_desc = FR_DESCRIPTIONS.get(app_id, tagline)

        # Map dependencies
        umbrel_deps = self.app_yml.get("dependencies", [])
        deps = ["docker"]
        for dep in umbrel_deps:
            if dep not in deps and dep != "app_proxy":
                deps.append(dep)

        # Convert services
        if self.is_multi_service():
            install, uninstall = self.convert_multi_service(app_id)
        else:
            services = self.get_services()
            svc_name = next(iter(services))
            svc = services[svc_name]
            install, uninstall = self.convert_single_service(app_id, svc_name, svc)

        manifest = {
            "id": app_id,
            "name": name,
            "version": version,
            "description": {
                "en": tagline,
                "fr": fr_desc,
            },
            "author": developer,
            "license": None,
            "website": website,
            "icon": ICON_MAP.get(app_id, "mdi:application"),
            "requirements": {
                "min_ram": 256,
                "min_disk": 200,
                "arch": ["aarch64", "x86_64"],
                "dependencies": deps,
            },
            "install": install,
            "uninstall": uninstall,
            "files": {},
            "config": {},
            "frontend": self.build_frontend_config(app_id),
        }

        return manifest

    def build_catalog_entry(self) -> dict:
        """Build a catalog.json entry for this app."""
        app_id = self.app_yml.get("id", "")
        category = self.app_yml.get("category", "utilities")
        pinas_category = CATEGORY_MAP.get(category, "utilities")

        return {
            "id": app_id,
            "name": self.app_yml.get("name", app_id),
            "version": self.app_yml.get("version", "1.0.0"),
            "category": pinas_category,
            "icon": ICON_MAP.get(app_id, "mdi:application"),
            "dependencies": ["docker"],
            "description": {
                "en": self.app_yml.get("tagline", ""),
                "fr": FR_DESCRIPTIONS.get(app_id, self.app_yml.get("tagline", "")),
            },
            "manifest": f"https://raw.githubusercontent.com/kameka22/pinas-app-catalog/master/apps/{app_id}/manifest.json",
        }


def main():
    parser = argparse.ArgumentParser(description="Convert Umbrel app to PiNAS manifest")
    parser.add_argument("umbrel_dir", help="Path to Umbrel app directory")
    parser.add_argument("output_dir", help="Output directory for manifest.json")
    parser.add_argument("--catalog", help="Path to catalog.json to update (adds entry)")
    parser.add_argument("--dry-run", action="store_true", help="Print manifest without writing")

    args = parser.parse_args()

    converter = UmbrelConverter(args.umbrel_dir)

    try:
        manifest = converter.convert()
    except Exception as e:
        print(f"Error converting {args.umbrel_dir}: {e}", file=sys.stderr)
        sys.exit(1)

    manifest_json = json.dumps(manifest, indent=2, ensure_ascii=False)

    if args.dry_run:
        print(manifest_json)
        return

    # Write manifest
    os.makedirs(args.output_dir, exist_ok=True)
    output_path = os.path.join(args.output_dir, "manifest.json")
    with open(output_path, "w") as f:
        f.write(manifest_json)
        f.write("\n")

    print(f"Written: {output_path}")
    app_id = manifest["id"]
    multi = "multi-service (compose)" if converter.is_multi_service() else "single-service"
    print(f"  App: {manifest['name']} ({app_id}) - {multi}")

    # Update catalog if requested
    if args.catalog and os.path.exists(args.catalog):
        entry = converter.build_catalog_entry()

        with open(args.catalog, "r") as f:
            catalog = json.load(f)

        # Remove existing entry if present
        catalog["apps"] = [a for a in catalog["apps"] if a["id"] != entry["id"]]
        catalog["apps"].append(entry)

        with open(args.catalog, "w") as f:
            json.dump(catalog, f, indent=2, ensure_ascii=False)
            f.write("\n")

        print(f"  Updated catalog: {args.catalog}")


if __name__ == "__main__":
    main()
