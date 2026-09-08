<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t, locale } from '$lib/i18n';
	import { auth } from '$stores/api';
	import { gradientStyle } from '$lib/utils/gradient';
	import {
		notifications,
		unreadCount,
		loadNotifications,
		markNotificationRead,
		markAllNotificationsRead,
		dismissNotification,
		clearNotifications,
		timeAgo,
		notificationVisual
	} from '$stores/notifications';

	export let visible = false;

	$: isAdmin = $auth.user?.role === 'admin';
	$: if (visible && isAdmin) loadNotifications();

	function sourceLabel(source: string): string {
		const sources = $t.notifications.sources as Record<string, string>;
		return sources[source] || source;
	}
</script>

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="notification-overlay" on:click={() => (visible = false)}></div>
{/if}

<aside class="notification-center" class:visible>
	<header class="notification-header">
		<div class="flex items-center gap-2">
			<Icon icon="mdi:bell-outline" class="w-5 h-5 text-slate-600" />
			<h2 class="text-sm font-semibold text-slate-800">{$t.notifications.title}</h2>
			{#if $unreadCount > 0}
				<span class="unread-pill">{$unreadCount}</span>
			{/if}
		</div>
		{#if $unreadCount > 0}
			<button class="header-action" on:click={markAllNotificationsRead} title={$t.notifications.markAllRead}>
				<Icon icon="mdi:check-all" class="w-4 h-4" />
			</button>
		{/if}
	</header>

	<div class="notification-list">
		{#if !isAdmin}
			<p class="notification-empty">{$t.notifications.adminOnly}</p>
		{:else if $notifications.length === 0}
			<p class="notification-empty">{$t.notifications.empty}</p>
		{:else}
			{#each $notifications as notification (notification.id)}
				{@const visual = notificationVisual(notification)}
				<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
				<div
					class="notification-item"
					class:unread={!notification.read}
					on:click={() => !notification.read && markNotificationRead(notification.id)}
				>
					<div class="notification-icon" style={gradientStyle(visual.gradient)}>
						<Icon icon={visual.icon} class="w-5 h-5 text-white" />
					</div>
					<div class="notification-content">
						<div class="notification-top">
							<h4 class="notification-title">{notification.title}</h4>
							<span class="notification-time" title={new Date(notification.created_at).toLocaleString($locale)}>
								{timeAgo(notification.created_at, $locale)}
							</span>
						</div>
						<p class="notification-message">{notification.message}</p>
						<span class="notification-source">{sourceLabel(notification.source)}</span>
					</div>
					<button
						class="dismiss-btn"
						on:click|stopPropagation={() => dismissNotification(notification.id)}
						title={$t.common.dismiss}
					>
						<Icon icon="mdi:close" class="w-4 h-4" />
					</button>
				</div>
			{/each}
		{/if}
	</div>

	{#if isAdmin && $notifications.length > 0}
		<footer class="notification-footer">
			<button class="view-all-btn" on:click={clearNotifications}>
				{$t.notifications.clearAll}
			</button>
		</footer>
	{/if}
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

	.notification-empty {
		padding: 2rem 1rem;
		text-align: center;
		font-size: 0.8125rem;
		color: #64748b;
	}
	.notification-source {
		display: inline-block;
		margin-top: 0.25rem;
		font-size: 0.6875rem;
		color: #94a3b8;
	}
	.unread-pill {
		font-size: 0.6875rem;
		font-weight: 600;
		padding: 0 0.4rem;
		border-radius: 999px;
		background: #ef4444;
		color: white;
		line-height: 1.1rem;
	}
	.header-action {
		padding: 0.25rem;
		border-radius: 0.375rem;
		color: #475569;
	}
	.header-action:hover { background: rgb(15 23 42 / 0.06); }
	.notification-item { cursor: pointer; }
</style>
