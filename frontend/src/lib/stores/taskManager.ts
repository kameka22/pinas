import { writable, derived } from 'svelte/store';

export type FileTaskType = 'upload' | 'copy' | 'move' | 'delete' | 'create_folder' | 'create_file';
export type FileTaskStatus = 'pending' | 'in_progress' | 'completed' | 'error';

export interface FileTask {
	id: string;
	type: FileTaskType;
	fileName: string;
	status: FileTaskStatus;
	progress: number; // 0-100
	error?: string;
	createdAt: Date;
}

const MAX_TASKS = 10;

function createTaskManagerStore() {
	const { subscribe, update, set } = writable<FileTask[]>([]);

	return {
		subscribe,

		addTask(type: FileTaskType, fileName: string, id?: string): string {
			const taskId = id || (crypto.randomUUID?.() ?? Math.random().toString(36).slice(2) + Date.now().toString(36));
			update(tasks => {
				const newTask: FileTask = {
					id: taskId,
					type,
					fileName,
					status: 'in_progress',
					progress: 0,
					createdAt: new Date()
				};
				const updated = [newTask, ...tasks];
				// Keep only the last MAX_TASKS
				return updated.slice(0, MAX_TASKS);
			});
			return taskId;
		},

		updateTask(taskId: string, updates: Partial<Pick<FileTask, 'status' | 'progress' | 'error'>>) {
			update(tasks =>
				tasks.map(t =>
					t.id === taskId ? { ...t, ...updates } : t
				)
			);
		},

		/** Create task if not found, otherwise update it */
		upsertTask(taskId: string, type: FileTaskType, fileName: string, updates: Partial<Pick<FileTask, 'status' | 'progress' | 'error'>>) {
			update(tasks => {
				const existing = tasks.find(t => t.id === taskId);
				if (existing) {
					return tasks.map(t =>
						t.id === taskId ? { ...t, ...updates } : t
					);
				}
				// Create new task with the given state
				const newTask: FileTask = {
					id: taskId,
					type,
					fileName,
					status: updates.status || 'in_progress',
					progress: updates.progress || 0,
					error: updates.error,
					createdAt: new Date()
				};
				return [newTask, ...tasks].slice(0, MAX_TASKS);
			});
		},

		clearCompleted() {
			update(tasks => tasks.filter(t => t.status === 'in_progress' || t.status === 'pending'));
		},

		reset() {
			set([]);
		}
	};
}

export const fileTasks = createTaskManagerStore();

/** True if at least one task is pending or in_progress */
export const hasActiveTask = derived(fileTasks, $tasks =>
	$tasks.some(t => t.status === 'pending' || t.status === 'in_progress')
);

/** Number of active (pending + in_progress) tasks */
export const activeTaskCount = derived(fileTasks, $tasks =>
	$tasks.filter(t => t.status === 'pending' || t.status === 'in_progress').length
);
