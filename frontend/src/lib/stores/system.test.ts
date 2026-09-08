import { describe, it, expect } from 'vitest';
import { formatBytes, formatUptime, formatRate } from './system';

describe('formatBytes', () => {
	it('formats with one decimal and the right unit', () => {
		expect(formatBytes(0)).toBe('0 B');
		expect(formatBytes(1024)).toBe('1 KB');
		expect(formatBytes(1536)).toBe('1.5 KB');
		expect(formatBytes(8 * 1024 ** 3)).toBe('8 GB');
		expect(formatBytes(2.5 * 1024 ** 4)).toBe('2.5 TB');
	});

	it('keeps sub-kilobyte values in bytes', () => {
		expect(formatBytes(1023)).toBe('1023 B');
		expect(formatBytes(1)).toBe('1 B');
	});
});

describe('formatUptime', () => {
	it('shows hours/minutes and days when relevant', () => {
		expect(formatUptime(0)).toBe('00h 00m');
		expect(formatUptime(3600 + 15 * 60)).toBe('01h 15m');
		expect(formatUptime(3 * 86400 + 2 * 3600 + 5 * 60 + 59)).toBe('3d 02h 05m');
	});
});

describe('formatRate', () => {
	it('formats throughput per second with the right unit', () => {
		expect(formatRate(0)).toBe('0 B/s');
		expect(formatRate(1536)).toBe('1.5 KB/s');
		expect(formatRate(12 * 1024 ** 2)).toBe('12 MB/s');
	});
});
