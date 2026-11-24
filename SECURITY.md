# Security policy

PiNAS runs with system privileges and exposes a web interface on your network, so security reports are taken seriously.

## Supported versions

PiNAS has no stable release yet. Fixes go to the `master` branch and ship with the next version.

## Reporting a vulnerability

Please **do not open a public issue** for a security problem.

Report it privately through [GitHub security advisories](https://github.com/kameka22/pinas/security/advisories/new) with:

- the affected version (`VERSION`, or *Control Panel → About*) and platform (Raspberry Pi 5 or x86_64);
- what an attacker can do, and from where (LAN, authenticated user, admin);
- steps to reproduce, or a proof of concept.

You will get an answer within a week. Once a fix is released, the advisory is published with credit to you unless you prefer otherwise.

## Scope

In scope: the PiNAS backend and web interface, the LibreELEC packages in `libreelec/`, the update mechanism (signed archives) and the build scripts.

Out of scope: vulnerabilities in LibreELEC, Kodi or third-party apps from the App Center themselves — report those to their projects.

Past audit findings and their status are in [SECURITY_AUDIT.md](SECURITY_AUDIT.md).
