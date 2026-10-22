import { BaseMessageBridge, type ExportRequest } from 'calcpad-frontend/services/message-bridge/base';
import { buildApiSettings, getDefaultSettings } from 'calcpad-frontend/types/settings';
import type { CalcpadSettings } from 'calcpad-frontend/types/settings';
import type { ExportVariant } from 'calcpad-frontend/types/api';
import { mimeFromExtension, bytesToBase64 } from 'calcpad-frontend';
import { setAppTheme, coerceAppTheme } from '../editor/app-theme';
import { serverOnlyMessage } from '../wasm/server-only';
import { printHtml } from '../wasm/print';

const SETTINGS_KEY = 'calcpad-settings';

function camelToKebab(s: string): string {
    return s.replace(/([A-Z])/g, '-$1').toLowerCase();
}

function extraKey(key: string): string {
    return 'calcpad-' + camelToKebab(key);
}

function triggerBlobDownload(blob: Blob, filename: string): void {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
}

/** Pop a hidden `<input type="file">` to let the user choose an image; resolve with the File. */
function pickImageViaInput(): Promise<File | null> {
    return new Promise(resolve => {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = 'image/png,image/jpeg,image/gif,image/webp,image/svg+xml';
        input.style.display = 'none';
        let settled = false;
        const cleanup = () => { if (input.parentNode) input.parentNode.removeChild(input); };
        input.onchange = () => {
            settled = true;
            const file = input.files?.[0] ?? null;
            cleanup();
            resolve(file);
        };
        // If the user cancels, no event fires reliably across browsers; clean up on next tick
        // when focus returns to the window.
        const onFocus = () => {
            window.removeEventListener('focus', onFocus);
            setTimeout(() => {
                if (!settled) { cleanup(); resolve(null); }
            }, 200);
        };
        window.addEventListener('focus', onFocus);
        document.body.appendChild(input);
        input.click();
    });
}

/**
 * In-process message bridge for the web platform. Persists settings to
 * localStorage and uses browser blob-downloads for exported files.
 */
export class MessageBridge extends BaseMessageBridge {
    constructor(serverUrl: string) {
        super(serverUrl);
        this.settings = this.loadSettings();
    }

    getExtraSetting(key: string): string | undefined {
        return localStorage.getItem(extraKey(key)) ?? undefined;
    }

    setExtraSetting(key: string, value: string): void {
        localStorage.setItem(extraKey(key), value);
    }

    protected persistSettings(settings: CalcpadSettings): void {
        localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    }

    protected resetSettingsBackend(): void {
        this.settings = getDefaultSettings();
        localStorage.setItem(SETTINGS_KEY, JSON.stringify(this.settings));
    }

    protected coerceColorTheme(raw: string | undefined | null): string {
        return coerceAppTheme(raw);
    }

    protected applyColorTheme(theme: string): void {
        setAppTheme(coerceAppTheme(theme));
    }

    protected async pickImageSrc(): Promise<string | null> {
        const file = await pickImageViaInput();
        if (!file) return null;
        const data = new Uint8Array(await file.arrayBuffer());
        const mimeType = file.type || mimeFromExtension(file.name);
        return `data:${mimeType};base64,${bytesToBase64(data)}`;
    }

    /** calcpad-rs: exports that need Calcpad.Server say so (see wasm/server-only.ts). */
    protected handlePlatformMessage(message: any): boolean {
        if (message?.type === 'saveXlsx') {
            void this.saveXlsx();
            return true;
        }
        const notice = serverOnlyMessage(message?.type);
        if (notice == null) return false;
        void this.showAlert('Not available in the browser', notice, 'info');
        return true;
    }

    /**
     * calcpad-rs: File > Export Excel. The report's variables, calculated in the page, as an
     * .xlsx (openxml::variables_export): a sheet of all variables, one per vector or matrix,
     * and one of the inputs.
     */
    private async saveXlsx(): Promise<void> {
        const content = this.getActiveEditorContent();
        const res = await fetch(`${window.location.origin}/api/calcpad/xlsx`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ content, settings: buildApiSettings(this.settings), forPrint: true }),
        });
        if (!res.ok) {
            void this.showAlert('Excel export', await res.text(), 'error');
            return;
        }
        await this.saveExportedFile({
            defaultName: 'calcpad-variables.xlsx',
            data: new Uint8Array(await res.arrayBuffer()),
            mime: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
            extensions: ['xlsx'],
            dialogTitle: 'Save Excel Workbook',
        });
    }

    protected async saveExportedFile(req: ExportRequest): Promise<string | null> {
        const part: BlobPart = req.data instanceof Uint8Array
            ? new Uint8Array(req.data)
            : req.data;
        triggerBlobDownload(new Blob([part], { type: req.mime }), req.defaultName);
        return null;
    }

    /**
     * Where the active worksheet lives, for resolving `#include`: its library path when it has one, else the library root
     * (an untitled sheet is treated as sitting there, which is where the engine looks too). Without a path the highlighter
     * finds no included file, and every macro and variable defined in one is reported as undefined.
     */
    protected async buildFileContext(_content: string): Promise<{ sourceFilePath?: string }> {
        const tab = (window as unknown as { calcpadTabs?: { activeTab: { title: string; filePath: string | null } | null } }).calcpadTabs?.activeTab;
        if (!tab) return {};
        const path = tab.filePath ?? `/${(tab.title || 'Untitled').replace(/[\\/:*?"<>|]/g, '_')}.cpd`;
        return { sourceFilePath: path.startsWith('/') ? path : `/${path}` };
    }

    protected getVariablesOrigin(): string {
        return 'web-editor';
    }

    /**
     * calcpad-rs: render, then print the HTML from the browser, whose print dialog saves it
     * as a PDF (wasm/print.ts). There is no server PDF endpoint, and nothing is returned to
     * save, since the dialog does that.
     */
    protected async generatePdfBytes(
        content: string,
        apiSettings: unknown,
        sourceFilePath: string | undefined,
        variant: ExportVariant,
    ): Promise<ArrayBuffer | null> {
        const html = await this.renderForExport(content, apiSettings, sourceFilePath, variant);
        if (html == null) return null;

        await printHtml(html, this.getEffectivePdfOptions(content));
        return null;
    }

    private loadSettings(): CalcpadSettings {
        const stored = localStorage.getItem(SETTINGS_KEY);
        if (stored) {
            try { return JSON.parse(stored); } catch { /* fall through */ }
        }
        return getDefaultSettings();
    }
}
