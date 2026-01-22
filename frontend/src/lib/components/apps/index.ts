// App Component Registry
// This file exports all available app components for dynamic loading

import Dashboard from './Dashboard.svelte';
import StorageManager from './StorageManager.svelte';
import ShareManager from './ShareManager.svelte';
import UserManager from './UserManager.svelte';
import Settings from './Settings.svelte';
import ControlPanel from './ControlPanel.svelte';
import FileManager from './FileManager.svelte';
import AppCenter from './AppCenter.svelte';
import DockerApp from './DockerApp.svelte';
import TerminalApp from './TerminalApp.svelte';
import ProcessManager from './ProcessManager.svelte';
import IframeApp from './IframeApp.svelte';
import WebviewApp from './WebviewApp.svelte';
import ServiceApp from './ServiceApp.svelte';

import type { Component } from 'svelte';

// Type for app component props
export interface AppComponentProps {
	config?: Record<string, unknown>;
	name?: string;
	icon?: string;
	gradient?: string;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyComponent = Component<any, any, any>;

// Registry of all available components
export const appComponents: Record<string, AnyComponent> = {
	// Built-in apps
	Dashboard,
	StorageManager,
	ShareManager,
	UserManager,
	Settings,
	ControlPanel,
	FileManager,
	AppCenter,
	DockerApp,
	TerminalApp,
	ProcessManager,

	// Generic app containers
	IframeApp,
	WebviewApp,
	ServiceApp,

	// Aliases for convenience
	Iframe: IframeApp,
	Webview: WebviewApp,
	Service: ServiceApp,

	// Placeholders (map to Dashboard for now)
	NetdiskTools: Dashboard,
	Support: Dashboard,
	TaskManager: ProcessManager
};

// Get a component by name, with fallback to Dashboard
export function getAppComponent(name: string): AnyComponent {
	return appComponents[name] || Dashboard;
}

// Check if a component exists
export function hasAppComponent(name: string): boolean {
	return name in appComponents;
}

// List all available component names
export function listAppComponents(): string[] {
	return Object.keys(appComponents);
}

// Export individual components for direct imports
export {
	Dashboard,
	StorageManager,
	ShareManager,
	UserManager,
	Settings,
	ControlPanel,
	FileManager,
	AppCenter,
	DockerApp,
	TerminalApp,
	ProcessManager,
	IframeApp,
	WebviewApp,
	ServiceApp
};
