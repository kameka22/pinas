<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { t } from '$lib/i18n';

	interface TerminalLine {
		type: 'input' | 'output' | 'error';
		content: string;
	}

	// Terminal state
	let lines: TerminalLine[] = [];
	let currentInput = '';
	let commandHistory: string[] = [];
	let historyIndex = -1;
	let isExecuting = false;
	let outputElement: HTMLDivElement;
	let inputElement: HTMLInputElement;

	// Current working directory
	let cwd = '/storage';

	// Prompt configuration (reactive based on cwd)
	$: displayPath = cwd === '/storage' ? '~' : cwd.replace('/storage', '~');
	$: prompt = `pinas@host:${displayPath}$`;

	// Add welcome message on mount
	onMount(() => {
		lines = [
			{ type: 'output', content: 'PiNAS Terminal v1.0' },
			{ type: 'output', content: 'Type "help" for available commands.' }
		];
		inputElement?.focus();
	});

	// Auto-scroll to bottom when lines change
	$: if (lines.length > 0) {
		scrollToBottom();
	}

	async function scrollToBottom() {
		await tick();
		if (outputElement) {
			outputElement.scrollTop = outputElement.scrollHeight;
		}
	}

	// Handle keyboard input
	function handleKeyDown(e: KeyboardEvent) {
		switch (e.key) {
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

	// Focus input element
	async function focusInput() {
		await tick();
		inputElement?.focus();
	}

	// Execute current command
	async function executeCommand() {
		const command = currentInput.trim();

		// Add command line to output
		lines = [...lines, { type: 'input', content: `${prompt} ${command}` }];

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

		// Execute via API
		isExecuting = true;
		try {
			const response = await fetch('/api/terminal/exec', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ command, cwd })
			});

			const data = await response.json();

			// Update cwd from response
			if (data.cwd) {
				cwd = data.cwd;
			}

			if (data.output) {
				// Add output as a single block (trim trailing newlines)
				const output = data.output.replace(/\n+$/, '');
				if (output) {
					lines = [...lines, {
						type: data.exit_code === 0 ? 'output' : 'error',
						content: output
					}];
				}
			}
		} catch (error) {
			lines = [...lines, {
				type: 'error',
				content: `Error: ${error instanceof Error ? error.message : 'Connection failed'}`
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
				{ type: 'output', content: '  Ctrl+C   - Cancel current command' },
				{ type: 'output', content: '  Ctrl+L   - Clear screen' },
				{ type: 'output', content: '  Up/Down  - Navigate command history' }
			];
			return true;
		}

		if (cmd === 'history') {
			lines = [...lines, { type: 'output', content: '' }];
			commandHistory.forEach((cmd, i) => {
				lines = [...lines, { type: 'output', content: `  ${i + 1}  ${cmd}` }];
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
			lines = [...lines, { type: 'input', content: `${prompt} ${currentInput}^C` }];
			currentInput = '';
		}
	}

	// Focus input when clicking on terminal
	function handleTerminalClick() {
		inputElement?.focus();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="terminal" on:click={handleTerminalClick}>
	<div class="output" bind:this={outputElement}>
		{#each lines as line}
			<div class="line {line.type}">
				{#if line.content}
					<pre>{line.content}</pre>
				{:else}
					<br />
				{/if}
			</div>
		{/each}
	</div>

	<div class="input-line">
		<span class="prompt">{prompt}</span>
		<input
			type="text"
			bind:value={currentInput}
			bind:this={inputElement}
			on:keydown={handleKeyDown}
			disabled={isExecuting}
			autocomplete="off"
			autocorrect="off"
			autocapitalize="off"
			spellcheck="false"
		/>
		{#if isExecuting}
			<span class="spinner"></span>
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
		color: #00ff00;
		overflow: hidden;
	}

	.output {
		flex: 1;
		overflow-y: auto;
		padding: 12px 16px;
		padding-bottom: 0;
	}

	.output::-webkit-scrollbar {
		width: 8px;
	}

	.output::-webkit-scrollbar-track {
		background: #1a1a1a;
	}

	.output::-webkit-scrollbar-thumb {
		background: #333;
		border-radius: 4px;
	}

	.output::-webkit-scrollbar-thumb:hover {
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

	.line.input {
		color: #00ff00;
	}

	.line.output {
		color: #e0e0e0;
	}

	.line.error {
		color: #ff6b6b;
	}

	.input-line {
		display: flex;
		align-items: center;
		padding: 8px 16px 12px;
		gap: 8px;
		background: #1a1a1a;
		border-top: 1px solid #333;
	}

	.prompt {
		color: #00ff00;
		font-weight: 600;
		white-space: nowrap;
	}

	.input-line input {
		flex: 1;
		background: transparent;
		border: none;
		outline: none;
		color: #00ff00;
		font-family: inherit;
		font-size: inherit;
		caret-color: #00ff00;
	}

	.input-line input:disabled {
		color: #666;
	}

	.input-line input::placeholder {
		color: #444;
	}

	.spinner {
		width: 12px;
		height: 12px;
		border: 2px solid #333;
		border-top-color: #00ff00;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* Selection styling */
	::selection {
		background: #00ff00;
		color: #1a1a1a;
	}
</style>
