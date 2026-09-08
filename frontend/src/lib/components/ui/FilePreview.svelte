<script lang="ts">
	import Icon from '@iconify/svelte';
	import { createEventDispatcher, onMount } from 'svelte';
	import { t } from '$lib/i18n';

	export let name: string;
	export let url: string;
	export let downloadUrl: string;
	export let mimeType: string | undefined = undefined;
	export let size: number | undefined = undefined;

	const dispatch = createEventDispatcher<{ close: void }>();
	const TEXT_LIMIT = 1024 * 1024; // 1 MB

	let text: string | null = null;
	let textError: string | null = null;
	let textLoading = false;

	$: kind = detectKind(mimeType, name);

	function detectKind(mime: string | undefined, filename: string): 'image' | 'video' | 'audio' | 'pdf' | 'text' | 'none' {
		const m = (mime || '').toLowerCase();
		const ext = filename.split('.').pop()?.toLowerCase() || '';
		if (m.startsWith('image/')) return 'image';
		if (m.startsWith('video/')) return 'video';
		if (m.startsWith('audio/')) return 'audio';
		if (m === 'application/pdf' || ext === 'pdf') return 'pdf';
		if (m.startsWith('text/') || ['md', 'json', 'yml', 'yaml', 'toml', 'ini', 'conf', 'log', 'sh', 'rs', 'ts', 'js', 'svelte', 'py', 'xml', 'csv'].includes(ext)) return 'text';
		return 'none';
	}

	async function loadText() {
		if (size !== undefined && size > TEXT_LIMIT) {
			textError = $t.fileManager.preview.tooLarge;
			return;
		}
		textLoading = true;
		try {
			const res = await fetch(url, { credentials: 'include' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			text = await res.text();
		} catch (e) {
			textError = e instanceof Error ? e.message : $t.common.errors.generic;
		} finally {
			textLoading = false;
		}
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') dispatch('close');
	}

	onMount(() => {
		if (kind === 'text') loadText();
	});
</script>

<svelte:window on:keydown={onKey} />

<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
<div class="preview-overlay" on:click={() => dispatch('close')}>
	<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
	<div class="preview-dialog" on:click|stopPropagation role="dialog" aria-label={name}>
		<header class="preview-header">
			<span class="preview-name" title={name}>{name}</span>
			<div class="preview-actions">
				<a class="preview-btn" href={downloadUrl} download={name} title={$t.fileManager.preview.download}>
					<Icon icon="mdi:download" class="w-5 h-5" />
				</a>
				<button class="preview-btn" on:click={() => dispatch('close')} title={$t.common.close}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
		</header>

		<div class="preview-body">
			{#if kind === 'image'}
				<img src={url} alt={name} />
			{:else if kind === 'video'}
				<!-- svelte-ignore a11y_media_has_caption -->
				<video src={url} controls autoplay></video>
			{:else if kind === 'audio'}
				<div class="preview-audio">
					<Icon icon="mdi:music-circle" class="w-20 h-20 text-slate-300" />
					<audio src={url} controls autoplay></audio>
				</div>
			{:else if kind === 'pdf'}
				<iframe src={url} title={name}></iframe>
			{:else if kind === 'text'}
				{#if textLoading}
					<div class="preview-empty"><Icon icon="mdi:loading" class="w-8 h-8 animate-spin" /></div>
				{:else if textError}
					<div class="preview-empty"><Icon icon="mdi:alert-circle-outline" class="w-10 h-10 text-slate-300" /><p>{textError}</p></div>
				{:else}
					<pre>{text}</pre>
				{/if}
			{:else}
				<div class="preview-empty">
					<Icon icon="mdi:file-question-outline" class="w-16 h-16 text-slate-300" />
					<p>{$t.fileManager.preview.noPreview}</p>
					<a class="preview-download" href={downloadUrl} download={name}>
						<Icon icon="mdi:download" class="w-4 h-4" />
						{$t.fileManager.preview.download}
					</a>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.preview-overlay {
		position: fixed;
		inset: 0;
		z-index: 9000;
		background: rgb(15 23 42 / 0.75);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 2rem;
	}
	.preview-dialog {
		width: min(1100px, 100%);
		height: min(85vh, 100%);
		background: #0f172a;
		border-radius: 1rem;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		box-shadow: 0 30px 80px rgb(0 0 0 / 0.5);
	}
	.preview-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		padding: 0.625rem 1rem;
		color: #e2e8f0;
		background: rgb(255 255 255 / 0.04);
	}
	.preview-name { font-size: 0.875rem; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.preview-actions { display: flex; gap: 0.25rem; }
	.preview-btn { padding: 0.375rem; border-radius: 0.5rem; color: #cbd5e1; display: inline-flex; }
	.preview-btn:hover { background: rgb(255 255 255 / 0.1); color: white; }
	.preview-body { flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center; overflow: auto; }
	img, video { max-width: 100%; max-height: 100%; object-fit: contain; }
	iframe { width: 100%; height: 100%; border: 0; background: white; }
	pre {
		align-self: stretch;
		width: 100%;
		margin: 0;
		padding: 1rem 1.25rem;
		font-size: 0.8125rem;
		line-height: 1.4;
		color: #e2e8f0;
		white-space: pre-wrap;
		word-break: break-word;
		text-align: left;
	}
	.preview-audio { display: flex; flex-direction: column; align-items: center; gap: 1.5rem; }
	.preview-empty { display: flex; flex-direction: column; align-items: center; gap: 0.75rem; color: #94a3b8; font-size: 0.875rem; }
	.preview-download {
		display: inline-flex; align-items: center; gap: 0.375rem;
		padding: 0.5rem 0.875rem; border-radius: 0.5rem;
		background: #3b82f6; color: white; font-size: 0.8125rem;
	}
</style>
