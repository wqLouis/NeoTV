import { browser } from '$app/environment';
import { invoke } from '@tauri-apps/api/core';

export interface ModalInfo {
	title: string;
	content: string;
	confirmText?: string;
	cancelText?: string;
	onConfirm?: () => void;
	onCancel?: () => void;
}

interface GstLibavInfo {
	installed: boolean;
	distro: string;
	distro_name: string;
	install_command: string;
	plugin_version: string | null;
}

let showModal = $state(false);
let modalInfo = $state<ModalInfo | null>(null);

export function getModalShow(): boolean {
	return showModal;
}

export function getModalInfo(): ModalInfo | null {
	return modalInfo;
}

export function showModalDialog(info: ModalInfo) {
	modalInfo = info;
	showModal = true;
}

export function hideModalDialog() {
	showModal = false;
	modalInfo = null;
}

export async function checkGstLibavStatus(): Promise<void> {
	if (!browser) return;

	try {
		const info = await invoke<GstLibavInfo>('get_gst_libav_status');
		if (!info.installed) {
			showModalDialog({
				title: '缺少 gst-libav 插件',
				content: `您的 Linux 发行版 "${info.distro_name}" 需要安装 gst-libav 插件才能播放视频。\n\n请运行以下命令安装：\n${info.install_command}`,
				confirmText: '知道了'
			});
		}
	} catch (e) {
		console.error('[ModalStore] Failed to check gst-libav status:', e);
	}
}

export function initModalListener() {
	if (!browser) return;

	import('@tauri-apps/api/event').then(({ listen }) => {
		listen<{ title: string; content: string }>('show-modal', (event) => {
			console.log('[ModalStore] Received show-modal event:', event.payload);
			showModalDialog({
				title: event.payload.title,
				content: event.payload.content
			});
		});
	});
}

export const modalStore = {
	get showModal() {
		return showModal;
	},
	get info() {
		return modalInfo;
	},
	show: showModalDialog,
	hide: hideModalDialog,
	init: initModalListener,
	checkGstLibav: checkGstLibavStatus
};
