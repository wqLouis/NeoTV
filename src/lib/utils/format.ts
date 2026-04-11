export function formatDuration(seconds: number): string {
	if (isNaN(seconds) || seconds < 0) return '00:00';
	const h = Math.floor(seconds / 3600);
	const m = Math.floor((seconds % 3600) / 60);
	const s = Math.floor(seconds % 60);
	if (h > 0) {
		return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
	}
	return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

export function formatRelativeTime(timestamp: number): string {
	const date = new Date(timestamp);
	const now = new Date();
	const diff = now.getTime() - date.getTime();

	if (diff < 60000) return '刚刚';
	if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
	if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
	if (diff < 604800000) return `${Math.floor(diff / 86400000)}天前`;
	return date.toLocaleDateString();
}
