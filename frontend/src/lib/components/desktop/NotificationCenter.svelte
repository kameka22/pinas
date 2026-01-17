<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';

	export let visible = false;

	interface Notification {
		id: string;
		icon: string;
		iconGradient: string;
		title: string;
		message: string;
		time: string;
		unread: boolean;
	}

	const notifications: Notification[] = [
		{
			id: '1',
			icon: 'mdi:download',
			iconGradient: 'from-green-400 to-green-500',
			title: 'Download Center',
			message: 'Download complete',
			time: 'Now',
			unread: true
		},
		{
			id: '2',
			icon: 'mdi:harddisk',
			iconGradient: 'from-slate-400 to-slate-500',
			title: 'Storage Manager(2)',
			message: 'Hard disk 6 is not in use yet. Please enter [Hard disk manager] to manage it.',
			time: 'Now',
			unread: true
		},
		{
			id: '3',
			icon: 'mdi:folder',
			iconGradient: 'from-blue-400 to-blue-500',
			title: 'File Manager (12)',
			message: 'File deduplication has been completed. A total of 849 images, 137 videos, and ...',
			time: '1 hour ago',
			unread: true
		},
		{
			id: '4',
			icon: 'mdi:apps',
			iconGradient: 'from-purple-400 to-purple-500',
			title: 'App Center',
			message: 'All apps have been upgraded to the latest version. Come and experience it.',
			time: 'February 21st 13:51',
			unread: false
		}
	];

	function dismissNotification(id: string) {
		// Would remove from store in real implementation
		console.log('Dismiss:', id);
	}

	function clearAll() {
		console.log('Clear all notifications');
	}
</script>

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="notification-overlay" on:click={() => (visible = false)}></div>
{/if}

<aside class="notification-center" class:visible>
	<header class="notification-header">
		<div class="flex items-center gap-2">
			<Icon icon="mdi:home" class="w-5 h-5 text-slate-600" />
			<h2 class="text-sm font-semibold text-slate-800">{$t.notifications.title}</h2>
		</div>
		<button class="settings-btn" title={$t.topBar.userMenu.settings}>
			<Icon icon="mdi:cog" class="w-4 h-4" />
		</button>
	</header>

	<div class="notification-list">
		{#each notifications as notification}
			<div class="notification-item" class:unread={notification.unread}>
				<div class="notification-icon bg-gradient-to-br {notification.iconGradient}">
					<Icon icon={notification.icon} class="w-5 h-5 text-white" />
				</div>
				<div class="notification-content">
					<div class="notification-top">
						<h4 class="notification-title">{notification.title}</h4>
						<span class="notification-time">{notification.time}</span>
					</div>
					<p class="notification-message">{notification.message}</p>
				</div>
				<button
					class="dismiss-btn"
					on:click={() => dismissNotification(notification.id)}
					title="Dismiss"
				>
					<Icon icon="mdi:close" class="w-4 h-4" />
				</button>
			</div>
		{/each}
	</div>

	<footer class="notification-footer">
		<button class="view-all-btn" on:click={clearAll}>
			{$t.notifications.clearAll}
		</button>
	</footer>
</aside>

<style>
	.notification-overlay {
		position: fixed;
		inset: 0;
		z-index: 199;
	}

	.notification-center {
		position: fixed;
		top: 48px;
		right: -340px;
		width: 320px;
		max-height: calc(100vh - 60px);
		background: rgba(255, 255, 255, 0.95);
		backdrop-filter: blur(20px);
		border-radius: 16px;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
		z-index: 200;
		display: flex;
		flex-direction: column;
		transition: right 0.3s ease;
		overflow: hidden;
	}

	.notification-center.visible {
		right: 12px;
	}

	.notification-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px;
		border-bottom: 1px solid rgba(0, 0, 0, 0.08);
	}

	.settings-btn {
		width: 28px;
		height: 28px;
		border-radius: 6px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.settings-btn:hover {
		background: rgba(0, 0, 0, 0.08);
		color: #334155;
	}

	.notification-list {
		flex: 1;
		overflow-y: auto;
		padding: 8px;
	}

	.notification-item {
		display: flex;
		gap: 12px;
		padding: 12px;
		border-radius: 12px;
		transition: all 0.15s ease;
		position: relative;
	}

	.notification-item:hover {
		background: rgba(0, 0, 0, 0.04);
	}

	.notification-item.unread {
		background: rgba(59, 130, 246, 0.08);
	}

	.notification-icon {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.notification-content {
		flex: 1;
		min-width: 0;
	}

	.notification-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 4px;
	}

	.notification-title {
		font-size: 13px;
		font-weight: 600;
		color: #1e293b;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.notification-time {
		font-size: 11px;
		color: #94a3b8;
		white-space: nowrap;
	}

	.notification-message {
		font-size: 12px;
		color: #64748b;
		line-height: 1.4;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.dismiss-btn {
		position: absolute;
		top: 8px;
		right: 8px;
		width: 24px;
		height: 24px;
		border-radius: 6px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #94a3b8;
		opacity: 0;
		transition: all 0.15s ease;
	}

	.notification-item:hover .dismiss-btn {
		opacity: 1;
	}

	.dismiss-btn:hover {
		background: rgba(0, 0, 0, 0.08);
		color: #64748b;
	}

	.notification-footer {
		padding: 12px 16px;
		border-top: 1px solid rgba(0, 0, 0, 0.08);
	}

	.view-all-btn {
		width: 100%;
		padding: 8px 16px;
		border-radius: 8px;
		background: rgba(0, 0, 0, 0.06);
		font-size: 13px;
		font-weight: 500;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.view-all-btn:hover {
		background: rgba(0, 0, 0, 0.1);
		color: #334155;
	}
</style>
