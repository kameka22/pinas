// Maps Tailwind gradient class strings to inline CSS background styles.
// This is needed because dynamic gradient classes from the API are not in source files,
// so Tailwind's JIT compiler cannot detect and generate them at build time.

const tailwindColors: Record<string, string> = {
	// Slate
	'slate-400': '#94a3b8', 'slate-500': '#64748b', 'slate-600': '#475569', 'slate-700': '#334155',
	// Gray
	'gray-400': '#9ca3af', 'gray-500': '#6b7280', 'gray-600': '#4b5563', 'gray-700': '#374151', 'gray-800': '#1f2937',
	// Zinc
	'zinc-500': '#71717a', 'zinc-600': '#52525b', 'zinc-700': '#3f3f46',
	// Red
	'red-400': '#f87171', 'red-500': '#ef4444', 'red-600': '#dc2626', 'red-700': '#b91c1c', 'red-800': '#991b1b',
	// Orange
	'orange-400': '#fb923c', 'orange-500': '#f97316', 'orange-600': '#ea580c',
	// Amber
	'amber-400': '#fbbf24', 'amber-500': '#f59e0b', 'amber-600': '#d97706',
	// Yellow
	'yellow-400': '#facc15', 'yellow-500': '#eab308', 'yellow-600': '#ca8a04', 'yellow-700': '#a16207',
	// Lime
	'lime-500': '#84cc16', 'lime-600': '#65a30d',
	// Green
	'green-400': '#4ade80', 'green-500': '#22c55e', 'green-600': '#16a34a',
	// Emerald
	'emerald-400': '#34d399', 'emerald-500': '#10b981', 'emerald-600': '#059669', 'emerald-700': '#047857',
	// Teal
	'teal-400': '#2dd4bf', 'teal-500': '#14b8a6', 'teal-600': '#0d9488', 'teal-700': '#0f766e',
	// Cyan
	'cyan-400': '#22d3ee', 'cyan-500': '#06b6d4', 'cyan-600': '#0891b2',
	// Sky
	'sky-400': '#38bdf8', 'sky-500': '#0ea5e9', 'sky-600': '#0284c7',
	// Blue
	'blue-400': '#60a5fa', 'blue-500': '#3b82f6', 'blue-600': '#2563eb', 'blue-700': '#1d4ed8', 'blue-800': '#1e40af',
	// Indigo
	'indigo-400': '#818cf8', 'indigo-500': '#6366f1', 'indigo-600': '#4f46e5', 'indigo-700': '#4338ca',
	// Violet
	'violet-500': '#8b5cf6', 'violet-600': '#7c3aed',
	// Purple
	'purple-400': '#c084fc', 'purple-500': '#a855f7', 'purple-600': '#9333ea',
	// Fuchsia
	'fuchsia-500': '#d946ef', 'fuchsia-600': '#c026d3',
	// Pink
	'pink-400': '#f472b6', 'pink-500': '#ec4899', 'pink-600': '#db2777',
	// Rose
	'rose-500': '#f43f5e', 'rose-600': '#e11d48',
};

/**
 * Convert a Tailwind gradient class string (e.g., "from-cyan-500 to-blue-500")
 * to an inline CSS background style string.
 * Returns empty string if the gradient can't be parsed.
 */
export function gradientStyle(gradient: string | undefined): string {
	if (!gradient) return '';

	const parts = gradient.split(/\s+/);
	let fromColor = '';
	let toColor = '';

	for (const part of parts) {
		if (part.startsWith('from-')) {
			fromColor = tailwindColors[part.slice(5)] || '';
		} else if (part.startsWith('to-')) {
			toColor = tailwindColors[part.slice(3)] || '';
		}
	}

	if (fromColor && toColor) {
		return `background: linear-gradient(to bottom right, ${fromColor}, ${toColor})`;
	}

	return '';
}
