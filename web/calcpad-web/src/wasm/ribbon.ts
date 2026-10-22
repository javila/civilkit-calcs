// calcpad-rs: mounts the ribbon above the app in the browser build (the desktop build has its native menu).
import { createApp } from 'vue';
import Ribbon from './Ribbon.vue';
import type { WebFiles } from './web-files';

export interface RibbonHost {
    app: unknown;
    bridge: unknown;
    showSidebarTab: (id: string) => void;
}

export function mountRibbon(files: WebFiles, host: RibbonHost): void {
    const el = document.createElement('div');
    el.id = 'cprs-ribbon';
    el.style.flex = 'none';
    document.body.insertBefore(el, document.body.firstChild);
    createApp(Ribbon, {
        ctx: { files, ...host, editor: () => (window as unknown as { calcpadActiveEditor?: unknown }).calcpadActiveEditor },
    }).mount(el);
}
