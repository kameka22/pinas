import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import { toasts, errorMessage } from './toasts';

describe('toasts store', () => {
	it('pushes and dismisses toasts by id', () => {
		toasts.clear();
		const id = toasts.error('boom', 0);
		expect(get(toasts)).toHaveLength(1);
		expect(get(toasts)[0]).toMatchObject({ id, kind: 'error', message: 'boom' });
		toasts.dismiss(id);
		expect(get(toasts)).toHaveLength(0);
	});

	it('auto-dismisses after the kind timeout', () => {
		vi.useFakeTimers();
		toasts.clear();
		toasts.success('saved');
		expect(get(toasts)).toHaveLength(1);
		vi.advanceTimersByTime(4000);
		expect(get(toasts)).toHaveLength(0);
		vi.useRealTimers();
	});

	it('keeps sticky toasts until cleared', () => {
		vi.useFakeTimers();
		toasts.clear();
		toasts.warning('careful', 0);
		vi.advanceTimersByTime(60_000);
		expect(get(toasts)).toHaveLength(1);
		toasts.clear();
		expect(get(toasts)).toHaveLength(0);
		vi.useRealTimers();
	});

	it('assigns increasing ids', () => {
		toasts.clear();
		const a = toasts.info('a', 0);
		const b = toasts.info('b', 0);
		expect(b).toBeGreaterThan(a);
		toasts.clear();
	});
});

describe('errorMessage', () => {
	it('prefers the Error message, then strings, then the fallback', () => {
		expect(errorMessage(new Error('nope'), 'fb')).toBe('nope');
		expect(errorMessage('plain', 'fb')).toBe('plain');
		expect(errorMessage(new Error(''), 'fb')).toBe('fb');
		expect(errorMessage(undefined, 'fb')).toBe('fb');
		expect(errorMessage({ weird: true }, 'fb')).toBe('fb');
	});
});
