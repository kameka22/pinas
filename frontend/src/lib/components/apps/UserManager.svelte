<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';

	// Types
	interface User {
		id: string;
		username: string;
		description: string;
		role: 'Administrator' | 'User' | 'Guest';
		status: 'Normal' | 'Disabled';
		isCurrentUser: boolean;
	}

	interface UserGroup {
		id: string;
		name: string;
		description: string;
		memberCount: number;
		isSystem: boolean;
	}

	// State
	let activeTab: 'user' | 'group' | 'settings' = 'user';
	let filterQuery = '';
	let showAddDropdown = false;
	let showActionMenu: string | null = null;
	let showPermissionViewer = false;
	let showAddUserModal = false;
	let showEditUserModal = false;
	let showDeleteConfirm = false;
	let selectedUser: User | null = null;

	// Sort state
	let sortColumn: string = 'username';
	let sortDirection: 'asc' | 'desc' = 'asc';

	// Mock data
	const users: User[] = [
		{ id: '1', username: 'admin', description: '-', role: 'Administrator', status: 'Normal', isCurrentUser: true },
		{ id: '2', username: 'john_doe', description: 'John Doe', role: 'Administrator', status: 'Normal', isCurrentUser: false },
		{ id: '3', username: 'jane', description: '-', role: 'User', status: 'Normal', isCurrentUser: false },
		{ id: '4', username: 'bob', description: 'Bob Smith', role: 'User', status: 'Normal', isCurrentUser: false },
		{ id: '5', username: 'guest', description: 'Guest account', role: 'Guest', status: 'Disabled', isCurrentUser: false }
	];

	const groups: UserGroup[] = [
		{ id: '1', name: 'administrators', description: 'System administrators', memberCount: 2, isSystem: true },
		{ id: '2', name: 'users', description: 'Regular users', memberCount: 3, isSystem: true },
		{ id: '3', name: 'guests', description: 'Guest users', memberCount: 1, isSystem: true },
		{ id: '4', name: 'media', description: 'Media access group', memberCount: 2, isSystem: false }
	];

	// Password settings state
	let passwordSettings = {
		noUserNames: false,
		noCommonPasswords: false,
		requireUpperLower: true,
		requireNumber: false,
		requireSpecialChar: false,
		minLength: 6,
		expiryEnabled: false,
		validityDays: 90,
		reminderDays: 7,
		forceChangeAfterExpiry: false
	};

	// Computed
	$: filteredUsers = users.filter(u =>
		u.username.toLowerCase().includes(filterQuery.toLowerCase()) ||
		u.description.toLowerCase().includes(filterQuery.toLowerCase())
	);

	$: filteredGroups = groups.filter(g =>
		g.name.toLowerCase().includes(filterQuery.toLowerCase()) ||
		g.description.toLowerCase().includes(filterQuery.toLowerCase())
	);

	// Functions
	function toggleSort(column: string) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = 'asc';
		}
	}

	function handleAddUser() {
		showAddDropdown = false;
		showAddUserModal = true;
	}

	function handleEditUser(user: User) {
		selectedUser = user;
		showActionMenu = null;
		showEditUserModal = true;
	}

	function handleDeleteUser(user: User) {
		selectedUser = user;
		showActionMenu = null;
		showDeleteConfirm = true;
	}

	function closeAllMenus() {
		showAddDropdown = false;
		showActionMenu = null;
	}
</script>

<svelte:window on:click={closeAllMenus} />

<div class="user-manager">
	<!-- Tabs -->
	<nav class="tabs">
		<button
			class="tab"
			class:active={activeTab === 'user'}
			on:click={() => activeTab = 'user'}
		>
			{$t.userManager.tabs.user}
		</button>
		<button
			class="tab"
			class:active={activeTab === 'group'}
			on:click={() => activeTab = 'group'}
		>
			{$t.userManager.tabs.userGroup}
		</button>
		<button
			class="tab"
			class:active={activeTab === 'settings'}
			on:click={() => activeTab = 'settings'}
		>
			{$t.userManager.tabs.advancedSettings}
		</button>
	</nav>

	<!-- Content -->
	<div class="content">
		{#if activeTab === 'user'}
			<!-- User Tab -->
			<div class="toolbar">
				<div class="toolbar-left">
					<div class="dropdown-container">
						<button
							class="btn-primary"
							on:click|stopPropagation={() => showAddDropdown = !showAddDropdown}
						>
							{$t.common.add}
							<Icon icon="mdi:chevron-down" class="w-4 h-4" />
						</button>
						{#if showAddDropdown}
							<div class="dropdown-menu">
								<button class="dropdown-item" on:click={handleAddUser}>
									<Icon icon="mdi:account-plus" class="w-4 h-4" />
									{$t.userManager.actions.addUser}
								</button>
							</div>
						{/if}
					</div>
				</div>
				<div class="toolbar-right">
					<button
						class="icon-btn"
						title={$t.userManager.modals.permissionViewer}
						on:click|stopPropagation={() => showPermissionViewer = true}
					>
						<Icon icon="mdi:cog-outline" class="w-5 h-5" />
					</button>
					<div class="filter-input">
						<Icon icon="mdi:filter-variant" class="w-4 h-4 text-gray-400" />
						<input
							type="text"
							placeholder={$t.common.filter}
							bind:value={filterQuery}
						/>
					</div>
				</div>
			</div>

			<!-- Users Table -->
			<div class="table-container">
				<table class="data-table">
					<thead>
						<tr>
							<th class="sortable" on:click={() => toggleSort('username')}>
								{$t.userManager.table.username}
								<Icon icon="mdi:unfold-more-horizontal" class="w-4 h-4" />
							</th>
							<th>{$t.userManager.table.description}</th>
							<th class="sortable" on:click={() => toggleSort('role')}>
								{$t.userManager.table.role}
								<Icon icon="mdi:unfold-more-horizontal" class="w-4 h-4" />
							</th>
							<th class="sortable" on:click={() => toggleSort('status')}>
								{$t.userManager.table.status}
								<Icon icon="mdi:unfold-more-horizontal" class="w-4 h-4" />
							</th>
							<th>{$t.userManager.table.edit}</th>
						</tr>
					</thead>
					<tbody>
						{#each filteredUsers as user}
							<tr class:disabled={user.status === 'Disabled'}>
								<td>
									<div class="user-cell">
										<div class="avatar">
											<Icon icon="mdi:account" class="w-5 h-5" />
										</div>
										<span class="username">{user.username}</span>
										{#if user.isCurrentUser}
											<span class="badge-me">{$t.userManager.messages.me}</span>
										{/if}
									</div>
								</td>
								<td class="text-secondary">{user.description}</td>
								<td>{user.role}</td>
								<td class:text-disabled={user.status === 'Disabled'}>{user.status}</td>
								<td>
									<div class="action-cell">
										<button
											class="action-btn"
											on:click|stopPropagation={() => showActionMenu = showActionMenu === user.id ? null : user.id}
										>
											<Icon icon="mdi:dots-horizontal" class="w-5 h-5" />
										</button>
										{#if showActionMenu === user.id}
											<div class="action-menu">
												<button class="action-item" on:click={() => handleEditUser(user)}>
													{$t.common.edit}
												</button>
												{#if !user.isCurrentUser}
													<button class="action-item danger" on:click={() => handleDeleteUser(user)}>
														{$t.common.delete}
													</button>
												{/if}
											</div>
										{/if}
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

		{:else if activeTab === 'group'}
			<!-- User Group Tab -->
			<div class="toolbar">
				<div class="toolbar-left">
					<div class="dropdown-container">
						<button
							class="btn-primary"
							on:click|stopPropagation={() => showAddDropdown = !showAddDropdown}
						>
							{$t.common.add}
							<Icon icon="mdi:chevron-down" class="w-4 h-4" />
						</button>
						{#if showAddDropdown}
							<div class="dropdown-menu">
								<button class="dropdown-item">
									<Icon icon="mdi:account-group-outline" class="w-4 h-4" />
									{$t.userManager.actions.addGroup}
								</button>
							</div>
						{/if}
					</div>
				</div>
				<div class="toolbar-right">
					<div class="filter-input">
						<Icon icon="mdi:filter-variant" class="w-4 h-4 text-gray-400" />
						<input
							type="text"
							placeholder={$t.common.filter}
							bind:value={filterQuery}
						/>
					</div>
				</div>
			</div>

			<!-- Groups Table -->
			<div class="table-container">
				<table class="data-table">
					<thead>
						<tr>
							<th class="sortable">
								{$t.userManager.table.groupName}
								<Icon icon="mdi:unfold-more-horizontal" class="w-4 h-4" />
							</th>
							<th>{$t.userManager.table.description}</th>
							<th class="sortable">
								{$t.userManager.table.members}
								<Icon icon="mdi:unfold-more-horizontal" class="w-4 h-4" />
							</th>
							<th>{$t.userManager.table.edit}</th>
						</tr>
					</thead>
					<tbody>
						{#each filteredGroups as group}
							<tr>
								<td>
									<span class="group-name">{group.name}</span>
								</td>
								<td class="text-secondary">{group.description || '-'}</td>
								<td>{group.memberCount}</td>
								<td>
									<div class="action-cell">
										<button
											class="action-btn"
											on:click|stopPropagation={() => showActionMenu = showActionMenu === group.id ? null : group.id}
										>
											<Icon icon="mdi:dots-horizontal" class="w-5 h-5" />
										</button>
										{#if showActionMenu === group.id}
											<div class="action-menu">
												<button class="action-item">{$t.common.edit}</button>
												{#if !group.isSystem}
													<button class="action-item danger">{$t.common.delete}</button>
												{/if}
											</div>
										{/if}
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

		{:else if activeTab === 'settings'}
			<!-- Advanced Settings Tab -->
			<div class="settings-content">
				<section class="settings-section">
					<h3>{$t.userManager.advancedSettings.passwordStrength.title}</h3>
					<div class="settings-list">
						<label class="checkbox-row">
							<input type="checkbox" bind:checked={passwordSettings.noUserNames} />
							<span>{$t.userManager.advancedSettings.passwordStrength.noUserNames}</span>
						</label>
						<label class="checkbox-row">
							<input type="checkbox" bind:checked={passwordSettings.noCommonPasswords} />
							<span>{$t.userManager.advancedSettings.passwordStrength.noCommonPasswords}</span>
						</label>
						<label class="checkbox-row">
							<input type="checkbox" bind:checked={passwordSettings.requireUpperLower} />
							<span>{$t.userManager.advancedSettings.passwordStrength.requireUpperLower}</span>
						</label>
						<label class="checkbox-row">
							<input type="checkbox" bind:checked={passwordSettings.requireNumber} />
							<span>{$t.userManager.advancedSettings.passwordStrength.requireNumber}</span>
						</label>
						<label class="checkbox-row">
							<input type="checkbox" bind:checked={passwordSettings.requireSpecialChar} />
							<span>{$t.userManager.advancedSettings.passwordStrength.requireSpecialChar}</span>
						</label>
						<label class="checkbox-row with-input">
							<input type="checkbox" checked disabled />
							<span class="text-disabled">{$t.userManager.advancedSettings.passwordStrength.minLength}</span>
							<div class="number-input">
								<input type="number" bind:value={passwordSettings.minLength} min="4" max="32" />
								<span class="suffix">{$t.userManager.advancedSettings.passwordStrength.digits}</span>
							</div>
						</label>
					</div>
				</section>

				<section class="settings-section">
					<h3>{$t.userManager.advancedSettings.passwordExpiry.title}</h3>
					<div class="settings-list">
						<div class="toggle-row">
							<span>{$t.userManager.advancedSettings.passwordExpiry.enabled}</span>
							<label class="toggle">
								<input type="checkbox" bind:checked={passwordSettings.expiryEnabled} />
								<span class="toggle-slider"></span>
							</label>
						</div>

						{#if passwordSettings.expiryEnabled}
							<div class="input-row">
								<span>{$t.userManager.advancedSettings.passwordExpiry.validityPeriod}</span>
								<div class="number-input">
									<input type="number" bind:value={passwordSettings.validityDays} min="1" max="365" />
									<span class="suffix">{$t.userManager.advancedSettings.passwordExpiry.days}</span>
								</div>
							</div>
							<div class="input-row">
								<span>{$t.userManager.advancedSettings.passwordExpiry.reminderDays}</span>
								<div class="number-input">
									<input type="number" bind:value={passwordSettings.reminderDays} min="1" max="30" />
									<span class="suffix">{$t.userManager.advancedSettings.passwordExpiry.days}</span>
								</div>
							</div>
							<div class="toggle-row">
								<span>{$t.userManager.advancedSettings.passwordExpiry.forceChange}</span>
								<label class="toggle">
									<input type="checkbox" bind:checked={passwordSettings.forceChangeAfterExpiry} />
									<span class="toggle-slider"></span>
								</label>
							</div>
						{/if}
					</div>
				</section>

				<div class="settings-footer">
					<button class="btn-primary">{$t.common.apply}</button>
				</div>
			</div>
		{/if}
	</div>
</div>

<!-- Permission Viewer Modal -->
{#if showPermissionViewer}
	<div class="modal-overlay" on:click={() => showPermissionViewer = false}>
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>{$t.userManager.modals.permissionViewer}</h2>
				<button class="modal-close" on:click={() => showPermissionViewer = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-body">
				<div class="permission-filters">
					<div class="select-group">
						<label>{$t.userManager.permissions.user}</label>
						<select>
							<option value="">{$t.common.search}</option>
							{#each users as user}
								<option value={user.id}>{user.username}</option>
							{/each}
						</select>
					</div>
					<div class="select-group">
						<label>{$t.userManager.permissions.sharedFolder}</label>
						<select>
							<option value="">{$t.common.search}</option>
							<option value="media">media</option>
							<option value="documents">documents</option>
							<option value="docker">docker</option>
						</select>
					</div>
					<button class="btn-secondary">{$t.common.reset}</button>
				</div>
				<table class="data-table">
					<thead>
						<tr>
							<th>{$t.userManager.table.username}</th>
							<th>media</th>
							<th>documents</th>
							<th>docker</th>
						</tr>
					</thead>
					<tbody>
						{#each users.slice(0, 4) as user}
							<tr>
								<td>
									<div class="user-cell">
										<div class="avatar">
											<Icon icon="mdi:account" class="w-5 h-5" />
										</div>
										<span>{user.username}</span>
									</div>
								</td>
								<td>{$t.userManager.permissions.readWrite}</td>
								<td>{$t.userManager.permissions.readWrite}</td>
								<td>{$t.userManager.permissions.readWrite}</td>
							</tr>
						{/each}
					</tbody>
				</table>
				<div class="pagination">
					<span>{$t.userManager.messages.userDataShown}: 1-{Math.min(users.length, 12)}</span>
					<div class="pagination-controls">
						<button disabled><Icon icon="mdi:chevron-left" class="w-4 h-4" /></button>
						<span class="page-number">1</span>
						<button disabled><Icon icon="mdi:chevron-right" class="w-4 h-4" /></button>
					</div>
				</div>
			</div>
		</div>
	</div>
{/if}

<!-- Add User Modal -->
{#if showAddUserModal}
	<div class="modal-overlay" on:click={() => showAddUserModal = false}>
		<div class="modal modal-sm" on:click|stopPropagation>
			<div class="modal-header">
				<h2>{$t.userManager.modals.addUserTitle}</h2>
				<button class="modal-close" on:click={() => showAddUserModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-body">
				<div class="form-group">
					<label>{$t.userManager.fields.username} <span class="required">*</span></label>
					<input type="text" placeholder="" />
				</div>
				<div class="form-group">
					<label>{$t.userManager.fields.description}</label>
					<input type="text" placeholder="" />
				</div>
				<div class="form-group">
					<label>{$t.userManager.fields.password} <span class="required">*</span></label>
					<input type="password" placeholder="" />
				</div>
				<div class="form-group">
					<label>{$t.userManager.fields.confirmPassword} <span class="required">*</span></label>
					<input type="password" placeholder="" />
				</div>
				<div class="form-group">
					<label>{$t.userManager.fields.role} <span class="required">*</span></label>
					<select>
						<option value="User">{$t.userManager.roles.user}</option>
						<option value="Administrator">{$t.userManager.roles.administrator}</option>
						<option value="Guest">{$t.userManager.roles.guest}</option>
					</select>
				</div>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" on:click={() => showAddUserModal = false}>{$t.common.cancel}</button>
				<button class="btn-primary">{$t.common.create}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Edit User Modal -->
{#if showEditUserModal && selectedUser}
	<div class="modal-overlay" on:click={() => showEditUserModal = false}>
		<div class="modal modal-sm" on:click|stopPropagation>
			<div class="modal-header">
				<h2>{$t.userManager.modals.editUserTitle}</h2>
				<button class="modal-close" on:click={() => showEditUserModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-body">
				<div class="form-group">
					<label>{$t.userManager.fields.username}</label>
					<input type="text" value={selectedUser.username} disabled />
				</div>
				<div class="form-group">
					<label>{$t.userManager.fields.description}</label>
					<input type="text" value={selectedUser.description === '-' ? '' : selectedUser.description} placeholder="" />
				</div>
				<div class="form-group">
					<label>{$t.userManager.fields.role}</label>
					<select value={selectedUser.role}>
						<option value="User">{$t.userManager.roles.user}</option>
						<option value="Administrator">{$t.userManager.roles.administrator}</option>
						<option value="Guest">{$t.userManager.roles.guest}</option>
					</select>
				</div>
				<div class="form-group">
					<label>{$t.userManager.fields.status}</label>
					<select value={selectedUser.status}>
						<option value="Normal">{$t.userManager.statuses.normal}</option>
						<option value="Disabled">{$t.userManager.statuses.disabled}</option>
					</select>
				</div>
				<label class="checkbox-row">
					<input type="checkbox" />
					<span>{$t.userManager.fields.changePassword}</span>
				</label>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" on:click={() => showEditUserModal = false}>{$t.common.cancel}</button>
				<button class="btn-primary">{$t.common.save}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete Confirmation Modal -->
{#if showDeleteConfirm && selectedUser}
	<div class="modal-overlay" on:click={() => showDeleteConfirm = false}>
		<div class="modal modal-sm" on:click|stopPropagation>
			<div class="modal-header">
				<h2>{$t.userManager.modals.deleteUserTitle}</h2>
				<button class="modal-close" on:click={() => showDeleteConfirm = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-body">
				<p>{$t.userManager.messages.deleteConfirm.replace('{username}', selectedUser.username)}</p>
				<p class="text-secondary">{$t.userManager.messages.cannotBeUndone}</p>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" on:click={() => showDeleteConfirm = false}>{$t.common.cancel}</button>
				<button class="btn-danger">{$t.common.delete}</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.user-manager {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: white;
	}

	/* Tabs */
	.tabs {
		display: flex;
		gap: 8px;
		padding: 0 20px;
		border-bottom: 1px solid #e5e7eb;
	}

	.tab {
		padding: 14px 16px;
		font-size: 14px;
		color: #6b7280;
		border-bottom: 2px solid transparent;
		margin-bottom: -1px;
		transition: all 0.15s ease;
	}

	.tab:hover {
		color: #3b82f6;
	}

	.tab.active {
		color: #3b82f6;
		border-bottom-color: #3b82f6;
	}

	/* Content */
	.content {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* Toolbar */
	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
	}

	.toolbar-left, .toolbar-right {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	/* Dropdown */
	.dropdown-container {
		position: relative;
	}

	.dropdown-menu {
		position: absolute;
		top: 100%;
		left: 0;
		margin-top: 4px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
		min-width: 160px;
		z-index: 50;
	}

	.dropdown-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 10px 14px;
		font-size: 14px;
		color: #374151;
		text-align: left;
	}

	.dropdown-item:hover {
		background: #f3f4f6;
	}

	/* Filter Input */
	.filter-input {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
	}

	.filter-input input {
		border: none;
		background: transparent;
		font-size: 14px;
		width: 120px;
		outline: none;
	}

	.filter-input input::placeholder {
		color: #9ca3af;
	}

	/* Icon Button */
	.icon-btn {
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 8px;
		color: #6b7280;
		transition: all 0.15s ease;
	}

	.icon-btn:hover {
		background: #f3f4f6;
		color: #374151;
	}

	/* Table */
	.table-container {
		flex: 1;
		overflow: auto;
		padding: 0 20px 20px;
	}

	.data-table {
		width: 100%;
		border-collapse: collapse;
	}

	.data-table th {
		padding: 12px 16px;
		text-align: left;
		font-size: 13px;
		font-weight: 500;
		color: #6b7280;
		background: #f9fafb;
		border-bottom: 1px solid #e5e7eb;
	}

	.data-table th.sortable {
		cursor: pointer;
		user-select: none;
	}

	.data-table th.sortable:hover {
		color: #374151;
	}

	.data-table td {
		padding: 12px 16px;
		font-size: 14px;
		color: #374151;
		border-bottom: 1px solid #f3f4f6;
	}

	.data-table tr:hover {
		background: #f9fafb;
	}

	.data-table tr.disabled td {
		color: #9ca3af;
	}

	/* User Cell */
	.user-cell {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.avatar {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: #e5e7eb;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #6b7280;
	}

	.username {
		font-weight: 500;
	}

	.badge-me {
		padding: 2px 8px;
		background: #3b82f6;
		color: white;
		font-size: 11px;
		font-weight: 500;
		border-radius: 10px;
	}

	.group-name {
		font-weight: 500;
	}

	.text-secondary {
		color: #6b7280;
	}

	.text-disabled {
		color: #9ca3af;
	}

	/* Action Cell */
	.action-cell {
		position: relative;
	}

	.action-btn {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 6px;
		color: #6b7280;
	}

	.action-btn:hover {
		background: #e5e7eb;
	}

	.action-menu {
		position: absolute;
		top: 100%;
		right: 0;
		margin-top: 4px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
		min-width: 100px;
		z-index: 50;
		overflow: hidden;
	}

	.action-item {
		display: block;
		width: 100%;
		padding: 10px 14px;
		font-size: 14px;
		color: #374151;
		text-align: left;
	}

	.action-item:hover {
		background: #f3f4f6;
	}

	.action-item.danger {
		color: #ef4444;
	}

	.action-item.danger:hover {
		background: #fef2f2;
	}

	/* Settings */
	.settings-content {
		flex: 1;
		overflow-y: auto;
		padding: 20px;
	}

	.settings-section {
		margin-bottom: 32px;
	}

	.settings-section h3 {
		font-size: 15px;
		font-weight: 600;
		color: #1f2937;
		margin-bottom: 16px;
		padding-bottom: 12px;
		border-bottom: 1px solid #e5e7eb;
	}

	.settings-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.checkbox-row {
		display: flex;
		align-items: center;
		gap: 12px;
		cursor: pointer;
	}

	.checkbox-row input[type="checkbox"] {
		width: 18px;
		height: 18px;
		accent-color: #3b82f6;
	}

	.checkbox-row span {
		font-size: 14px;
		color: #374151;
	}

	.checkbox-row.with-input {
		justify-content: space-between;
	}

	.toggle-row, .input-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 0;
	}

	.toggle-row span, .input-row span {
		font-size: 14px;
		color: #374151;
	}

	/* Toggle Switch */
	.toggle {
		position: relative;
		width: 48px;
		height: 24px;
	}

	.toggle input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	.toggle-slider {
		position: absolute;
		cursor: pointer;
		inset: 0;
		background: #d1d5db;
		border-radius: 24px;
		transition: 0.3s;
	}

	.toggle-slider::before {
		content: '';
		position: absolute;
		height: 18px;
		width: 18px;
		left: 3px;
		bottom: 3px;
		background: white;
		border-radius: 50%;
		transition: 0.3s;
	}

	.toggle input:checked + .toggle-slider {
		background: #3b82f6;
	}

	.toggle input:checked + .toggle-slider::before {
		transform: translateX(24px);
	}

	/* Number Input */
	.number-input {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.number-input input {
		width: 80px;
		padding: 8px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 6px;
		font-size: 14px;
		text-align: right;
	}

	.number-input .suffix {
		font-size: 14px;
		color: #6b7280;
	}

	.settings-footer {
		display: flex;
		justify-content: flex-end;
		padding: 16px 0;
		border-top: 1px solid #e5e7eb;
		margin-top: 16px;
	}

	/* Buttons */
	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
	}

	.btn-primary:hover {
		background: #2563eb;
	}

	.btn-secondary {
		padding: 8px 16px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		color: #374151;
		transition: all 0.15s ease;
	}

	.btn-secondary:hover {
		background: #f9fafb;
	}

	.btn-danger {
		padding: 8px 16px;
		background: #ef4444;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
	}

	.btn-danger:hover {
		background: #dc2626;
	}

	/* Modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.modal {
		background: white;
		border-radius: 12px;
		width: 90%;
		max-width: 700px;
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
	}

	.modal.modal-sm {
		max-width: 450px;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid #e5e7eb;
	}

	.modal-header h2 {
		font-size: 16px;
		font-weight: 600;
		color: #1f2937;
	}

	.modal-close {
		color: #6b7280;
		padding: 4px;
		border-radius: 4px;
	}

	.modal-close:hover {
		background: #f3f4f6;
		color: #374151;
	}

	.modal-body {
		flex: 1;
		padding: 20px;
		overflow-y: auto;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 20px;
		border-top: 1px solid #e5e7eb;
	}

	/* Permission Filters */
	.permission-filters {
		display: flex;
		align-items: flex-end;
		gap: 16px;
		margin-bottom: 20px;
	}

	.select-group {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.select-group label {
		font-size: 13px;
		color: #6b7280;
	}

	.select-group select {
		padding: 8px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 6px;
		font-size: 14px;
		min-width: 150px;
	}

	/* Pagination */
	.pagination {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 0;
		font-size: 13px;
		color: #6b7280;
	}

	.pagination-controls {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.pagination-controls button {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid #e5e7eb;
		border-radius: 4px;
		color: #6b7280;
	}

	.pagination-controls button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.page-number {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid #3b82f6;
		border-radius: 4px;
		color: #3b82f6;
		font-weight: 500;
	}

	/* Form */
	.form-group {
		margin-bottom: 16px;
	}

	.form-group label {
		display: block;
		margin-bottom: 6px;
		font-size: 14px;
		color: #374151;
	}

	.form-group .required {
		color: #ef4444;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
	}

	.form-group input:focus,
	.form-group select:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.form-group input:disabled {
		background: #f3f4f6;
		color: #9ca3af;
	}

	.modal-body p {
		font-size: 14px;
		color: #374151;
		margin-bottom: 8px;
	}
</style>
