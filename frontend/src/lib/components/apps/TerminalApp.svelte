<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { t } from '$lib/i18n';
	import { auth } from '$stores/api';
	import { systemInfo } from '$stores/system';
	import { get } from 'svelte/store';

	function getAuthToken(): string | null {
		return get(auth).token;
	}

	interface PromptLine {
		type: 'prompt';
		user: string;
		host: string;
		path: string;
		command: string;
	}

	interface OutputLine {
		type: 'output' | 'error';
		content: string;
	}

	type TerminalLine = PromptLine | OutputLine;

	// Terminal state
	let lines: TerminalLine[] = [];
	let currentInput = '';
	let commandHistory: string[] = [];
	let historyIndex = -1;
	let isExecuting = false;
	let terminalElement: HTMLDivElement;
	let hiddenInput: HTMLInputElement;
	let hasFocus = true;

	// Current working directory
	let cwd = '/storage';

	// Tab completion state
	let showCompletions = false;
	let completionDisplay = '';
	let isCompleting = false;

	// Prompt display values
	$: username = $auth.user?.username || 'root';
	$: hostname = $systemInfo?.hostname || 'pinas';
	$: displayPath = cwd === '/storage' ? '~' : cwd.replace('/storage', '~');

	// i18n
	$: tTerm = ($t as any).terminalApp || {};
	$: tErrors = tTerm.errors || {};

	// Add welcome message on mount
	onMount(() => {
		const tr = get(t) as any;
		const term = tr.terminalApp || {};
		lines = [
			{ type: 'output', content: term.welcome || 'PiNAS Terminal v1.0' },
			{ type: 'output', content: term.helpHint || 'Type "help" for available commands.' },
			{ type: 'output', content: term.tabHint || 'Use Tab for path auto-completion.' }
		];
		hiddenInput?.focus();
	});

	// Auto-scroll to bottom when lines change
	$: if (lines.length > 0 || currentInput !== undefined) {
		scrollToBottom();
	}

	async function scrollToBottom() {
		await tick();
		if (terminalElement) {
			terminalElement.scrollTop = terminalElement.scrollHeight;
		}
	}

	// Handle click on terminal - focus hidden input (but not if selecting text)
	function handleTerminalClick() {
		const sel = window.getSelection();
		if (sel && !sel.isCollapsed) return;
		hiddenInput?.focus();
	}

	function handleFocus() {
		hasFocus = true;
	}

	function handleBlur() {
		hasFocus = false;
	}

	// Handle keyboard input
	function handleKeyDown(e: KeyboardEvent) {
		// Dismiss completions on any key except Tab
		if (e.key !== 'Tab' && showCompletions) {
			showCompletions = false;
			completionDisplay = '';
		}

		switch (e.key) {
			case 'Tab':
				e.preventDefault();
				handleTabCompletion();
				return;
			case 'Enter':
				e.preventDefault();
				executeCommand();
				break;
			case 'ArrowUp':
				e.preventDefault();
				navigateHistory(-1);
				break;
			case 'ArrowDown':
				e.preventDefault();
				navigateHistory(1);
				break;
			case 'c':
				if (e.ctrlKey) {
					e.preventDefault();
					cancelCommand();
				}
				break;
			case 'l':
				if (e.ctrlKey) {
					e.preventDefault();
					clearTerminal();
				}
				break;
		}
	}

	// Navigate command history
	function navigateHistory(direction: number) {
		if (commandHistory.length === 0) return;

		const newIndex = historyIndex + direction;
		if (newIndex < -1) return;
		if (newIndex >= commandHistory.length) {
			historyIndex = -1;
			currentInput = '';
			return;
		}

		historyIndex = newIndex;
		if (historyIndex === -1) {
			currentInput = '';
		} else {
			currentInput = commandHistory[commandHistory.length - 1 - historyIndex];
		}
	}

	// Focus hidden input
	async function focusInput() {
		await tick();
		hiddenInput?.focus();
	}

	// Execute current command
	async function executeCommand() {
		const command = currentInput.trim();

		// Add prompt line to history
		lines = [...lines, {
			type: 'prompt' as const,
			user: username,
			host: hostname,
			path: displayPath,
			command: command
		}];

		// Clear input
		currentInput = '';
		historyIndex = -1;

		if (!command) {
			await focusInput();
			return;
		}

		// Add to history
		commandHistory = [...commandHistory, command];

		// Handle built-in commands
		if (handleBuiltinCommand(command)) {
			await focusInput();
			return;
		}

		// Execute via API (authenticated, admin-only)
		isExecuting = true;
		try {
			const response = await fetch('/api/terminal/exec', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					...(getAuthToken() ? { 'Authorization': `Bearer ${getAuthToken()}` } : {})
				},
				body: JSON.stringify({ command, cwd })
			});

			const data = await response.json();

			if (response.status === 401) {
				lines = [...lines, { type: 'error', content: tErrors.authRequired || 'Authentication required. Please log in again.' }];
			} else if (response.status === 403 && !data.output) {
				lines = [...lines, { type: 'error', content: tErrors.adminRequired || 'Admin access required to use the terminal.' }];
			} else {
				// Update cwd from response
				if (data.cwd) {
					cwd = data.cwd;
				}

				if (data.output) {
					const output = data.output.replace(/\n+$/, '');
					if (output) {
						lines = [...lines, {
							type: data.exit_code === 0 ? 'output' : 'error',
							content: output
						}];
					}
				}
			}
		} catch (error) {
			lines = [...lines, {
				type: 'error',
				content: `Error: ${error instanceof Error ? error.message : (tErrors.connectionFailed || 'Connection failed')}`
			}];
		} finally {
			isExecuting = false;
			await focusInput();
		}
	}

	// Handle built-in terminal commands
	function handleBuiltinCommand(command: string): boolean {
		const cmd = command.toLowerCase();

		if (cmd === 'clear' || cmd === 'cls') {
			clearTerminal();
			return true;
		}

		if (cmd === 'help') {
			lines = [...lines,
				{ type: 'output', content: 'Built-in commands:' },
				{ type: 'output', content: '  help     - Show this help message' },
				{ type: 'output', content: '  clear    - Clear the terminal screen' },
				{ type: 'output', content: '  history  - Show command history' },
				{ type: 'output', content: '' },
				{ type: 'output', content: 'Navigation:' },
				{ type: 'output', content: '  cd <dir> - Change directory' },
				{ type: 'output', content: '  pwd      - Print working directory' },
				{ type: 'output', content: '' },
				{ type: 'output', content: 'Keyboard shortcuts:' },
				{ type: 'output', content: '  Tab      - Path auto-completion' },
				{ type: 'output', content: '  Ctrl+C   - Cancel current command' },
				{ type: 'output', content: '  Ctrl+L   - Clear screen' },
				{ type: 'output', content: '  Up/Down  - Navigate command history' }
			];
			return true;
		}

		if (cmd === 'history') {
			lines = [...lines, { type: 'output', content: '' }];
			commandHistory.forEach((c, i) => {
				lines = [...lines, { type: 'output', content: `  ${i + 1}  ${c}` }];
			});
			lines = [...lines, { type: 'output', content: '' }];
			return true;
		}

		return false;
	}

	// Clear terminal
	function clearTerminal() {
		lines = [];
	}

	// Cancel current command (Ctrl+C)
	function cancelCommand() {
		if (isExecuting) {
			lines = [...lines, { type: 'error', content: '^C' }];
			isExecuting = false;
		} else {
			lines = [...lines, {
				type: 'prompt' as const,
				user: username,
				host: hostname,
				path: displayPath,
				command: currentInput + '^C'
			}];
			currentInput = '';
		}
	}

	// --- Tab completion ---

	/** Parse input into tokens, respecting quotes */
	function parseTokens(input: string): string[] {
		const tokens: string[] = [];
		let current = '';
		let inSingle = false;
		let inDouble = false;

		for (let i = 0; i < input.length; i++) {
			const ch = input[i];
			if (ch === "'" && !inDouble) {
				inSingle = !inSingle;
				current += ch;
			} else if (ch === '"' && !inSingle) {
				inDouble = !inDouble;
				current += ch;
			} else if (ch === ' ' && !inSingle && !inDouble) {
				if (current.length > 0) {
					tokens.push(current);
					current = '';
				}
			} else {
				current += ch;
			}
		}
		if (current.length > 0) {
			tokens.push(current);
		}
		return tokens;
	}

	/** Replace the last token in input */
	function replaceLastToken(input: string, replacement: string): string {
		// Find the start of the last token
		let lastSpaceIdx = -1;
		let inSingle = false;
		let inDouble = false;

		for (let i = 0; i < input.length; i++) {
			const ch = input[i];
			if (ch === "'" && !inDouble) inSingle = !inSingle;
			else if (ch === '"' && !inSingle) inDouble = !inDouble;
			else if (ch === ' ' && !inSingle && !inDouble) lastSpaceIdx = i;
		}

		if (lastSpaceIdx === -1) {
			return replacement;
		}
		return input.substring(0, lastSpaceIdx + 1) + replacement;
	}

	async function handleTabCompletion() {
		if (isCompleting || isExecuting) return;

		const tokens = parseTokens(currentInput);
		// The partial is the last token, or empty if input ends with space
		const endsWithSpace = currentInput.endsWith(' ') && currentInput.trim().length > 0;
		const partial = endsWithSpace ? '' : (tokens.length > 0 ? tokens[tokens.length - 1] : '');

		isCompleting = true;
		try {
			const response = await fetch('/api/terminal/complete', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					...(getAuthToken() ? { 'Authorization': `Bearer ${getAuthToken()}` } : {})
				},
				body: JSON.stringify({ partial, cwd })
			});

			if (!response.ok) return;

			const data: { matches: { name: string; is_dir: boolean }[]; common_prefix: string } = await response.json();

			if (data.matches.length === 0) {
				// No matches - do nothing
				return;
			}

			if (data.matches.length === 1) {
				// Single match: replace token and add / or space
				const match = data.matches[0];
				const suffix = match.is_dir ? '/' : ' ';

				// Build the replacement preserving path prefix
				let replacement: string;
				if (partial.includes('/')) {
					const lastSlash = partial.lastIndexOf('/');
					replacement = partial.substring(0, lastSlash + 1) + match.name + suffix;
				} else {
					replacement = match.name + suffix;
				}

				if (endsWithSpace) {
					currentInput = currentInput + replacement;
				} else {
					currentInput = replaceLastToken(currentInput, replacement);
				}
				showCompletions = false;
			} else {
				// Multiple matches: fill common prefix if longer, show matches
				if (data.common_prefix.length > 0) {
					const currentPrefix = partial.includes('/') ? partial.substring(partial.lastIndexOf('/') + 1) : partial;
					if (data.common_prefix.length > currentPrefix.length) {
						let replacement: string;
						if (partial.includes('/')) {
							const lastSlash = partial.lastIndexOf('/');
							replacement = partial.substring(0, lastSlash + 1) + data.common_prefix;
						} else {
							replacement = data.common_prefix;
						}

						if (endsWithSpace) {
							currentInput = currentInput + replacement;
						} else {
							currentInput = replaceLastToken(currentInput, replacement);
						}
					}
				}

				// Display matches
				const display = data.matches.map(m => m.is_dir ? m.name + '/' : m.name).join('  ');
				completionDisplay = display;
				showCompletions = true;
			}
		} catch {
			// Silently ignore completion errors
		} finally {
			isCompleting = false;
			await scrollToBottom();
		}
	}

	// --- URL formatting ---

	/** Escape HTML and make URLs clickable (safe: escape first, then match URLs) */
	function formatOutput(text: string): string {
		const escaped = text
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;')
			.replace(/"/g, '&quot;')
			.replace(/'/g, '&#39;');
		// Strict URL regex: only allow safe characters in URLs (no quotes, angle brackets, backticks)
		const urlRegex = /(https?:\/\/[a-zA-Z0-9\-._~:/?#\[\]@!$&()*+,;=%]+)/g;
		return escaped.replace(
			urlRegex,
			(url) => `<a href="${url}" target="_blank" rel="noopener noreferrer" class="terminal-link">${url}</a>`
		);
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="terminal" on:click={handleTerminalClick}>
	<!-- Hidden input captures all keyboard input -->
	<input
		type="text"
		class="hidden-input"
		bind:this={hiddenInput}
		bind:value={currentInput}
		on:keydown={handleKeyDown}
		on:focus={handleFocus}
		on:blur={handleBlur}
		disabled={isExecuting}
		autocomplete="off"
		autocorrect="off"
		autocapitalize="off"
		spellcheck="false"
	/>

	<div class="terminal-content" bind:this={terminalElement}>
		<!-- History lines -->
		{#each lines as line}
			{#if line.type === 'prompt'}
				<div class="line prompt-line">
					<pre><span class="c-user">{line.user}@{line.host}</span><span class="c-sep">:</span><span class="c-path">{line.path}</span><span class="c-sep">$ </span>{line.command}</pre>
				</div>
			{:else if line.type === 'error'}
				<div class="line error">
					{#if line.content}
						<pre>{@html formatOutput(line.content)}</pre>
					{:else}
						<br />
					{/if}
				</div>
			{:else}
				<div class="line output">
					{#if line.content}
						<pre>{@html formatOutput(line.content)}</pre>
					{:else}
						<br />
					{/if}
				</div>
			{/if}
		{/each}

		<!-- Active prompt (inline in flow) -->
		<div class="line prompt-line active">
			<pre><span class="c-user">{username}@{hostname}</span><span class="c-sep">:</span><span class="c-path">{displayPath}</span><span class="c-sep">$ </span>{currentInput}<span class="cursor" class:blink={hasFocus && !isExecuting}>█</span>{#if isExecuting}<span class="spinner-inline">⠋</span>{/if}</pre>
		</div>

		<!-- Completion suggestions (temporary) -->
		{#if showCompletions}
			<div class="line output completions">
				<pre>{completionDisplay}</pre>
			</div>
		{/if}
	</div>
</div>

<style>
	.terminal {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: #1a1a1a;
		font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Consolas', monospace;
		font-size: 14px;
		line-height: 1.4;
		color: #e0e0e0;
		overflow: hidden;
		position: relative;
	}

	.hidden-input {
		position: absolute;
		left: -9999px;
		opacity: 0;
		width: 0;
		height: 0;
	}

	.terminal-content {
		flex: 1;
		overflow-y: auto;
		padding: 8px 12px;
	}

	.terminal-content::-webkit-scrollbar {
		width: 8px;
	}

	.terminal-content::-webkit-scrollbar-track {
		background: #1a1a1a;
	}

	.terminal-content::-webkit-scrollbar-thumb {
		background: #333;
		border-radius: 4px;
	}

	.terminal-content::-webkit-scrollbar-thumb:hover {
		background: #444;
	}

	.line {
		min-height: 1.4em;
	}

	.line pre {
		margin: 0;
		white-space: pre-wrap;
		word-wrap: break-word;
		font-family: inherit;
	}

	.line.output {
		color: #e0e0e0;
	}

	.line.error {
		color: #ff6b6b;
	}

	.line.completions {
		color: #999;
	}

	/* Prompt colors */
	.c-user {
		color: #4ade80;
		font-weight: 600;
	}

	.c-path {
		color: #60a5fa;
		font-weight: 600;
	}

	.c-sep {
		color: #e0e0e0;
	}

	.prompt-line pre {
		color: #e0e0e0;
	}

	/* Cursor */
	.cursor {
		color: #e0e0e0;
		font-weight: 400;
	}

	.cursor.blink {
		animation: blink-cursor 1s step-end infinite;
	}

	@keyframes blink-cursor {
		0%, 100% { opacity: 1; }
		50% { opacity: 0; }
	}

	/* Spinner for executing state */
	.spinner-inline {
		color: #4ade80;
		margin-left: 4px;
		animation: spin-chars 0.8s steps(6) infinite;
	}

	@keyframes spin-chars {
		0% { content: '⠋'; }
		16% { content: '⠙'; }
		33% { content: '⠹'; }
		50% { content: '⠸'; }
		66% { content: '⠼'; }
		83% { content: '⠴'; }
	}

	/* Clickable URLs */
	:global(.terminal-link) {
		color: #60a5fa;
		text-decoration: underline;
		text-decoration-style: dotted;
	}

	:global(.terminal-link:hover) {
		color: #93bbfc;
		text-decoration-style: solid;
	}

	/* Selection styling */
	::selection {
		background: #4ade80;
		color: #1a1a1a;
	}
</style>
