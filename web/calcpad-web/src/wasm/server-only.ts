// calcpad-rs: exports that need Calcpad.Server. The browser build has no server, so these
// buttons explain that instead of failing silently.

const SERVER_ONLY: Record<string, string> = {
    generatePdf: 'PDF export',
    saveDocx: 'Word export',
    saveCompiled: 'Saving a compiled .cpdz worksheet',
    savePortable: 'Saving a portable package',
};

/** The message to show for a bridge message that needs the server, or null. */
export function serverOnlyMessage(type: unknown): string | null {
    const what = typeof type === 'string' ? SERVER_ONLY[type] : undefined;
    if (!what) return null;
    return `${what} is not available in the browser build of CalcpadCE (calcpad-rs): it needs Calcpad.Server. `
        + 'Save HTML works, and the saved page prints to PDF from the browser.';
}
