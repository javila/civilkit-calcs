import * as monaco from 'monaco-editor';
import { compileSheetNow, isSheetText, loadSheet } from '../wasm/sheet';

/** Monaco language id of a sheet (docs/sheet-syntax.md). Calcpad's own providers do not apply to it. */
export const SHEET_LANGUAGE = 'calcpad-sheet';

const GREEK = ['alpha', 'beta', 'gamma', 'delta', 'epsilon', 'zeta', 'eta', 'theta', 'iota', 'kappa', 'lambda', 'mu', 'nu', 'xi', 'pi', 'rho', 'sigma', 'tau', 'upsilon', 'phi', 'chi', 'psi', 'omega'];

const configuration: monaco.languages.LanguageConfiguration = {
    brackets: [['(', ')'], ['[', ']'], ['{', '}']],
    autoClosingPairs: [
        { open: '(', close: ')' },
        { open: '[', close: ']' },
        { open: '{', close: '}' },
        { open: '"', close: '"', notIn: ['string'] },
        { open: '`', close: '`' },
    ],
    surroundingPairs: [{ open: '*', close: '*' }, { open: '`', close: '`' }, { open: '$', close: '$' }, { open: '(', close: ')' }, { open: '[', close: ']' }],
    onEnterRules: [
        // A block header continues one level in.
        { beforeText: /^\s*(if|else if|else|while|repeat|for)\b.*:\s*$/, action: { indentAction: monaco.languages.IndentAction.Indent } },
    ],
};

const ID = '[A-Za-z_\\u0370-\\u03ff][A-Za-z0-9_\\u0370-\\u03ff′″]*';

const monarch: monaco.languages.IMonarchLanguage = {
    defaultToken: '',
    tokenPostfix: '.sheet',
    unicode: true,
    tokenizer: {
        root: [
            [/<!--sheet-->/, 'comment'],
            [/^\s*>>.*$/, 'type'],
            [/^\s*```\s*calcpad\s*$/, { token: 'type', next: '@calcpadFence' }],
            [/^\s*```.*$/, { token: 'string', next: '@codeFence' }],
            [/^\s*#{1,6}\s.*$/, 'keyword.heading'],
            [/^\s*\$\$\s*$/, { token: 'string.math', next: '@displayMath' }],
            [/\$\$.+?\$\$/, 'string.math'],
            [/\$[^\s$][^$]*\$/, 'string.math'],
            [/^\s*(if|else if|else|while|repeat|for|plot)\b/, 'keyword'],
            [/\b(in)\b/, 'keyword'],
            [new RegExp(`^(\\s*)(${ID}(?:\\([^)]*\\))?)(\\s*)(:|=(?!=))`), ['', 'variable.name', '', 'delimiter']],
            [/\|\s.*$/, 'comment'],
            [/->/, 'operator'],
            [/\*\*[^*]+\*\*/, 'strong'],
            [/\*[^*\s][^*]*\*/, 'emphasis'],
            [/`[^`]*`/, 'string'],
            [/"[^"]*"/, 'string'],
            [/\d+(\.\d+)?([eE][+-]?\d+)?/, 'number'],
        ],
        calcpadFence: [[/^\s*```\s*$/, { token: 'type', next: '@pop' }], [/.*/, 'type']],
        codeFence: [[/^\s*```\s*$/, { token: 'string', next: '@pop' }], [/.*/, 'string']],
        displayMath: [[/^\s*\$\$\s*$/, { token: 'string.math', next: '@pop' }], [/.*/, 'string.math']],
    },
};

let registered = false;

/** Registers the sheet language, its tokenizer and completions. Call once with the Calcpad language. */
export function registerSheetLanguage(): void {
    if (registered) return;
    registered = true;
    monaco.languages.register({ id: SHEET_LANGUAGE, extensions: ['.sheet'] });
    monaco.languages.setLanguageConfiguration(SHEET_LANGUAGE, configuration);
    monaco.languages.setMonarchTokensProvider(SHEET_LANGUAGE, monarch);
    monaco.languages.registerCompletionItemProvider(SHEET_LANGUAGE, {
        provideCompletionItems(model, position) {
            const word = model.getWordUntilPosition(position);
            const range = { startLineNumber: position.lineNumber, endLineNumber: position.lineNumber, startColumn: word.startColumn, endColumn: word.endColumn };
            const out: monaco.languages.CompletionItem[] = [];
            const compiled = compileSheetNow(model.getValue());
            if (!compiled) void loadSheet();
            for (const d of compiled?.defs ?? []) {
                out.push({ label: d.name, kind: monaco.languages.CompletionItemKind.Variable, insertText: d.name, range, detail: d.calcpad === d.name ? 'defined in this sheet' : `Calcpad name: ${d.calcpad}` });
            }
            for (const g of GREEK) {
                out.push({ label: g, kind: monaco.languages.CompletionItemKind.Text, insertText: g, range, detail: 'Greek letter' });
            }
            const snip = (label: string, text: string, detail: string) => out.push({
                label, kind: monaco.languages.CompletionItemKind.Snippet, insertText: text, range, detail,
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            });
            snip('if', 'if ${1:condition}:\n\t$0', 'conditional block');
            snip('else', 'else:\n\t$0', 'else branch');
            snip('for', 'for ${1:i} in ${2:1}..${3:10}:\n\t$0', 'loop over a range');
            snip('while', 'while ${1:condition}:\n\t$0', 'while loop');
            snip('plot', 'plot ${1:f(x)} for ${2:x} in ${3:0}..${4:10}', 'plot');
            return { suggestions: out };
        },
    });
}

/** Sets a model's language from its text: a sheet when the first line is the marker. */
export function syncModelLanguage(model: monaco.editor.ITextModel): void {
    const want = isSheetText(model.getValue()) ? SHEET_LANGUAGE : 'calcpad';
    if (model.getLanguageId() !== want) monaco.editor.setModelLanguage(model, want);
}
