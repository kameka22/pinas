<script lang="ts">
	import Icon from '@iconify/svelte';

	let activeSection = 'general';

	const sections = [
		{ id: 'general', label: 'General', icon: 'mdi:cog' },
		{ id: 'network', label: 'Network', icon: 'mdi:lan' },
		{ id: 'services', label: 'Services', icon: 'mdi:server' },
		{ id: 'updates', label: 'Updates', icon: 'mdi:update' },
		{ id: 'backup', label: 'Backup', icon: 'mdi:backup-restore' },
		{ id: 'notifications', label: 'Notifications', icon: 'mdi:bell' },
		{ id: 'power', label: 'Power', icon: 'mdi:power' }
	];

	let hostname = 'pinas';
	let timezone = 'Europe/Paris';
	let language = 'en';
	let ipMode = 'dhcp';

	const services = [
		{ id: 'smb', name: 'SMB/CIFS', enabled: true, description: 'Windows file sharing' },
		{ id: 'nfs', name: 'NFS', enabled: true, description: 'Unix/Linux file sharing' },
		{ id: 'ssh', name: 'SSH', enabled: true, description: 'Secure remote access' },
		{ id: 'ftp', name: 'FTP', enabled: false, description: 'File transfer protocol' }
	];
</script>

<div class="settings">
	<aside class="sidebar">
		<h1>Settings</h1>
		<nav class="nav">
			{#each sections as section}
				<button
					class="nav-item"
					class:active={activeSection === section.id}
					on:click={() => activeSection = section.id}
				>
					<Icon icon={section.icon} class="w-5 h-5" />
					<span>{section.label}</span>
				</button>
			{/each}
		</nav>
	</aside>

	<main class="content">
		{#if activeSection === 'general'}
			<section class="section">
				<h2>General Settings</h2>
				<p class="section-desc">Configure basic system settings</p>

				<div class="form">
					<div class="form-group">
						<label for="hostname">Hostname</label>
						<input id="hostname" type="text" bind:value={hostname} />
						<span class="form-hint">The network name for this device</span>
					</div>

					<div class="form-group">
						<label for="timezone">Timezone</label>
						<select id="timezone" bind:value={timezone}>
							<option value="Europe/Paris">Europe/Paris</option>
							<option value="Europe/London">Europe/London</option>
							<option value="America/New_York">America/New York</option>
							<option value="Asia/Tokyo">Asia/Tokyo</option>
						</select>
					</div>

					<div class="form-group">
						<label for="language">Language</label>
						<select id="language" bind:value={language}>
							<option value="en">English</option>
							<option value="fr">Français</option>
							<option value="de">Deutsch</option>
						</select>
					</div>
				</div>

				<div class="form-actions">
					<button class="btn-primary">Save Changes</button>
				</div>
			</section>

		{:else if activeSection === 'network'}
			<section class="section">
				<h2>Network Settings</h2>
				<p class="section-desc">Configure network interfaces</p>

				<div class="form">
					<div class="form-group">
						<label>Network Interface</label>
						<select>
							<option value="eth0">eth0 (Ethernet)</option>
							<option value="wlan0">wlan0 (WiFi)</option>
						</select>
					</div>

					<div class="form-group">
						<label>IP Configuration</label>
						<div class="radio-group">
							<label class="radio-item">
								<input type="radio" name="ipMode" value="dhcp" bind:group={ipMode} />
								<span>DHCP (Automatic)</span>
							</label>
							<label class="radio-item">
								<input type="radio" name="ipMode" value="static" bind:group={ipMode} />
								<span>Static</span>
							</label>
						</div>
					</div>

					{#if ipMode === 'static'}
						<div class="static-config">
							<div class="form-group">
								<label>IP Address</label>
								<input type="text" placeholder="192.168.1.100" />
							</div>
							<div class="form-group">
								<label>Gateway</label>
								<input type="text" placeholder="192.168.1.1" />
							</div>
							<div class="form-group">
								<label>DNS Server</label>
								<input type="text" placeholder="8.8.8.8" />
							</div>
						</div>
					{/if}
				</div>

				<div class="form-actions">
					<button class="btn-primary">Apply Network Settings</button>
				</div>
			</section>

		{:else if activeSection === 'services'}
			<section class="section">
				<h2>Services</h2>
				<p class="section-desc">Enable or disable system services</p>

				<div class="services-list">
					{#each services as service}
						<div class="service-card">
							<div class="service-icon" class:enabled={service.enabled}>
								<Icon icon="mdi:server" class="w-5 h-5" />
							</div>
							<div class="service-info">
								<h4>{service.name}</h4>
								<p>{service.description}</p>
							</div>
							<div class="service-controls">
								<span class="service-status" class:running={service.enabled}>
									{service.enabled ? 'Running' : 'Stopped'}
								</span>
								<label class="toggle">
									<input type="checkbox" bind:checked={service.enabled} />
									<span class="toggle-slider"></span>
								</label>
							</div>
						</div>
					{/each}
				</div>
			</section>

		{:else if activeSection === 'power'}
			<section class="section">
				<h2>Power Management</h2>
				<p class="section-desc">System power options</p>

				<div class="power-buttons">
					<button class="power-btn">
						<Icon icon="mdi:restart" class="w-8 h-8 text-amber-500" />
						<span class="power-label">Restart</span>
						<span class="power-desc">Reboot the system</span>
					</button>
					<button class="power-btn danger">
						<Icon icon="mdi:power" class="w-8 h-8 text-red-500" />
						<span class="power-label">Shutdown</span>
						<span class="power-desc">Power off the system</span>
					</button>
				</div>

				<div class="card">
					<h3>Scheduled Tasks</h3>
					<div class="schedule-list">
						<div class="schedule-item">
							<Icon icon="mdi:power-sleep" class="w-5 h-5 text-slate-400" />
							<div class="schedule-info">
								<span class="schedule-name">Auto Shutdown</span>
								<span class="schedule-status">Disabled</span>
							</div>
							<button class="btn-configure">Configure</button>
						</div>
						<div class="schedule-item">
							<Icon icon="mdi:alarm" class="w-5 h-5 text-slate-400" />
							<div class="schedule-info">
								<span class="schedule-name">Wake on LAN</span>
								<span class="schedule-status">Enabled</span>
							</div>
							<button class="btn-configure">Configure</button>
						</div>
					</div>
				</div>
			</section>

		{:else}
			<div class="empty-state">
				<Icon icon="mdi:wrench" class="w-16 h-16 text-slate-300" />
				<p>This section is under development</p>
			</div>
		{/if}
	</main>
</div>

<style>
	.settings {
		display: flex;
		height: 100%;
		background: white;
	}

	.sidebar {
		width: 220px;
		padding: 20px;
		border-right: 1px solid #e2e8f0;
		background: #f8fafc;
	}

	.sidebar h1 {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 20px;
	}

	.nav {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		border-radius: 8px;
		font-size: 14px;
		color: #64748b;
		text-align: left;
		transition: all 0.15s ease;
	}

	.nav-item:hover { background: #e2e8f0; }
	.nav-item.active { background: #dbeafe; color: #2563eb; }

	.content {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	.section h2 {
		font-size: 20px;
		font-weight: 600;
		color: #1e293b;
	}

	.section-desc {
		font-size: 14px;
		color: #64748b;
		margin-top: 4px;
		margin-bottom: 24px;
	}

	.form {
		max-width: 500px;
	}

	.form-group {
		margin-bottom: 20px;
	}

	.form-group label {
		display: block;
		font-size: 14px;
		font-weight: 500;
		color: #475569;
		margin-bottom: 8px;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 10px 14px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		font-size: 14px;
		color: #1e293b;
		outline: none;
		transition: border-color 0.15s ease;
	}

	.form-group input:focus,
	.form-group select:focus {
		border-color: #3b82f6;
	}

	.form-hint {
		font-size: 12px;
		color: #94a3b8;
		margin-top: 6px;
	}

	.radio-group {
		display: flex;
		gap: 20px;
	}

	.radio-item {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 14px;
		color: #475569;
		cursor: pointer;
	}

	.static-config {
		padding: 16px;
		background: #f8fafc;
		border-radius: 8px;
		margin-top: 16px;
	}

	.form-actions {
		margin-top: 24px;
	}

	.services-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
		max-width: 600px;
	}

	.service-card {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
	}

	.service-icon {
		width: 44px;
		height: 44px;
		background: #e2e8f0;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #94a3b8;
	}

	.service-icon.enabled {
		background: #dbeafe;
		color: #3b82f6;
	}

	.service-info {
		flex: 1;
	}

	.service-info h4 {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
	}

	.service-info p {
		font-size: 12px;
		color: #64748b;
	}

	.service-controls {
		display: flex;
		align-items: center;
		gap: 16px;
	}

	.service-status {
		font-size: 12px;
		padding: 4px 10px;
		border-radius: 20px;
		background: #f1f5f9;
		color: #64748b;
	}

	.service-status.running {
		background: #dcfce7;
		color: #16a34a;
	}

	.toggle {
		position: relative;
		width: 44px;
		height: 24px;
	}

	.toggle input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	.toggle-slider {
		position: absolute;
		cursor: pointer;
		inset: 0;
		background: #cbd5e1;
		border-radius: 24px;
		transition: background 0.2s ease;
	}

	.toggle-slider:before {
		content: '';
		position: absolute;
		width: 18px;
		height: 18px;
		left: 3px;
		bottom: 3px;
		background: white;
		border-radius: 50%;
		transition: transform 0.2s ease;
	}

	.toggle input:checked + .toggle-slider {
		background: #3b82f6;
	}

	.toggle input:checked + .toggle-slider:before {
		transform: translateX(20px);
	}

	.power-buttons {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 16px;
		max-width: 400px;
		margin-bottom: 24px;
	}

	.power-btn {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 24px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		transition: all 0.15s ease;
	}

	.power-btn:hover {
		background: #f1f5f9;
	}

	.power-label {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
	}

	.power-desc {
		font-size: 12px;
		color: #64748b;
	}

	.card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		padding: 20px;
		max-width: 500px;
	}

	.card h3 {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 16px;
	}

	.schedule-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.schedule-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
		background: white;
		border-radius: 8px;
	}

	.schedule-info {
		flex: 1;
	}

	.schedule-name {
		display: block;
		font-size: 14px;
		color: #475569;
	}

	.schedule-status {
		font-size: 12px;
		color: #94a3b8;
	}

	.btn-configure {
		padding: 6px 12px;
		background: #f1f5f9;
		border-radius: 6px;
		font-size: 12px;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.btn-configure:hover {
		background: #e2e8f0;
	}

	.btn-primary {
		padding: 10px 20px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
	}

	.btn-primary:hover {
		background: #2563eb;
	}

	.empty-state {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
	}

	.empty-state p {
		font-size: 14px;
		color: #94a3b8;
	}
</style>
