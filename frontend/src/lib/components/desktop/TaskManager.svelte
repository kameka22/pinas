<script lang="ts">
	import Icon from '@iconify/svelte';
	import { fileTasks, hasActiveTask, activeTaskCount } from '$stores/taskManager';
	import type { FileTask, FileTaskType } from '$stores/taskManager';
	import { t } from '$lib/i18n';
	import { createEventDispatcher } from 'svelte';

	export let visible = false;

	const dispatch = createEventDispatcher();

	function close() {
		dispatch('close');
	}

	function getTaskIcon(type: FileTaskType): string {
		switch (type) {
			case 'upload': return 'mdi:upload';
			case 'copy': return 'mdi:content-copy';
			case 'move': return 'mdi:file-move';
			case 'delete': return 'mdi:delete';
			case 'create_folder': return 'mdi:folder-plus';
			case 'create_file': return 'mdi:file-plus';
			default: return 'mdi:file';
		}
	}

	function getTaskLabel(type: FileTaskType): string {
		const labels: Record<string, string> = {
			upload: $t.taskManager?.types?.upload || 'Upload',
			copy: $t.taskManager?.types?.copy || 'Copy',
			move: $t.taskManager?.types?.move || 'Move',
			delete: $t.taskManager?.types?.delete || 'Delete',
			create_folder: $t.taskManager?.types?.createFolder || 'Create folder',
			create_file: $t.taskManager?.types?.createFile || 'Create file'
		};
		return labels[type] || type;
	}

	function getStatusColor(task: FileTask): string {
		switch (task.status) {
			case 'completed': return '#22c55e';
			case 'error': return '#ef4444';
			case 'in_progress': return '#3b82f6';
			default: return '#94a3b8';
		}
	}
</script>

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="task-backdrop" on:click={close}></div>
	<div class="task-panel">
		<div class="task-header">
			<h3>{$t.taskManager?.title || 'Tasks'}</h3>
			<button class="clear-btn" on:click={() => fileTasks.clearCompleted()} title={$t.taskManager?.clear || 'Clear completed'}>
				<Icon icon="mdi:notification-clear-all" class="w-4 h-4" />
			</button>
		</div>

		<div class="task-list">
			{#if $fileTasks.length === 0}
				<div class="task-empty">
					<Icon icon="mdi:check-circle-outline" class="w-8 h-8" />
					<span>{$t.taskManager?.empty || 'No tasks'}</span>
				</div>
			{:else}
				{#each $fileTasks as task (task.id)}
					<div class="task-item" class:completed={task.status === 'completed'} class:error={task.status === 'error'}>
						<div class="task-icon" style="color: {getStatusColor(task)}">
							{#if task.status === 'in_progress'}
								<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
							{:else if task.status === 'completed'}
								<Icon icon="mdi:check-circle" class="w-4 h-4" />
							{:else if task.status === 'error'}
								<Icon icon="mdi:alert-circle" class="w-4 h-4" />
							{:else}
								<Icon icon={getTaskIcon(task.type)} class="w-4 h-4" />
							{/if}
						</div>
						<div class="task-info">
							<div class="task-name">
								<span class="task-type-label">{getTaskLabel(task.type)}</span>
								<span class="task-file-name">{task.fileName}</span>
							</div>
							{#if task.status === 'in_progress' && task.progress > 0}
								<div class="task-progress">
									<div class="task-progress-fill" style="width: {task.progress}%"></div>
								</div>
							{:else if task.status === 'in_progress'}
								<div class="task-progress indeterminate">
									<div class="task-progress-fill"></div>
								</div>
							{/if}
							{#if task.error}
								<span class="task-error">{task.error}</span>
							{/if}
						</div>
					</div>
				{/each}
			{/if}
		</div>
	</div>
{/if}

<style>
	.task-backdrop {
		position: fixed;
		inset: 0;
		z-index: 200;
	}

	.task-panel {
		position: fixed;
		top: 44px;
		left: 48px;
		width: 360px;
		max-height: 420px;
		background: rgba(255, 255, 255, 0.95);
		backdrop-filter: blur(20px);
		border-radius: 12px;
		box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2), 0 0 0 1px rgba(0, 0, 0, 0.05);
		z-index: 201;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		animation: panel-in 0.15s ease-out;
	}

	@keyframes panel-in {
		from { opacity: 0; transform: translateY(-8px); }
		to { opacity: 1; transform: translateY(0); }
	}

	.task-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid rgba(0, 0, 0, 0.08);
	}

	.task-header h3 {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
		margin: 0;
	}

	.clear-btn {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 6px;
		color: #64748b;
		transition: all 0.15s;
	}

	.clear-btn:hover {
		background: rgba(0, 0, 0, 0.06);
		color: #1e293b;
	}

	.task-list {
		overflow-y: auto;
		max-height: 360px;
		padding: 4px 0;
	}

	.task-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 32px 16px;
		color: #94a3b8;
	}

	.task-item {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 10px 16px;
		transition: background 0.1s;
	}

	.task-item:hover {
		background: rgba(0, 0, 0, 0.03);
	}

	.task-item.completed {
		opacity: 0.6;
	}

	.task-icon {
		flex-shrink: 0;
		margin-top: 2px;
	}

	.task-info {
		flex: 1;
		min-width: 0;
	}

	.task-name {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
	}

	.task-type-label {
		color: #64748b;
		flex-shrink: 0;
	}

	.task-file-name {
		color: #1e293b;
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.task-progress {
		height: 3px;
		background: #e2e8f0;
		border-radius: 2px;
		margin-top: 6px;
		overflow: hidden;
	}

	.task-progress-fill {
		height: 100%;
		background: #3b82f6;
		border-radius: 2px;
		transition: width 0.3s ease;
	}

	.task-progress.indeterminate .task-progress-fill {
		width: 40%;
		animation: indeterminate 1.5s ease-in-out infinite;
	}

	@keyframes indeterminate {
		0% { transform: translateX(-100%); }
		100% { transform: translateX(350%); }
	}

	.task-error {
		font-size: 11px;
		color: #ef4444;
		margin-top: 4px;
		display: block;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	:global(.animate-spin) {
		animation: spin 1s linear infinite;
	}
</style>
