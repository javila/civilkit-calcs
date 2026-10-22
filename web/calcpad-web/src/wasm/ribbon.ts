// calcpad-rs: mounts the ribbon above the app in the browser build (the desktop build has its native menu).
import { createApp } from 'vue';
import Ribbon from './Ribbon.vue';
import type { WebFiles } from './web-files';

export function mountRibbon(files: WebFiles, app: unknown): void {
    const host = document.createElement('div');
    host.id = 'cprs-ribbon';
    host.style.flex = 'none';
    document.body.insertBefore(host, document.body.firstChild);
    createApp(Ribbon, {
        ctx: { files, app, editor: () => (window as unknown as { calcpadActiveEditor?: unknown }).calcpadActiveEditor },
    }).mount(host);
}
