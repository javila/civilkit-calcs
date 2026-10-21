import * as monaco from 'monaco-editor';
import { convertLegacyInputsInLine } from '../wasm/legacy-input';

/** The linter's code for a deprecated `name = ? {default}` field (CPD-3419). */
const CODE = 'CPD-3419';

/**
 * Quick fix on CPD-3419: "Convert to #UI" rewrites the line as the #UI declaration the migration
 * guide gives (see wasm/legacy-input.ts). It is offered only where the rewrite applies: every
 * field on the line a plain number assigned to a name, and the line not another directive.
 */
export function registerLegacyInputFix(): monaco.IDisposable {
    return monaco.languages.registerCodeActionProvider('calcpad', {
        provideCodeActions(model, _range, context) {
            const actions: monaco.languages.CodeAction[] = [];
            const done = new Set<number>();
            for (const marker of context.markers) {
                const code = typeof marker.code === 'string' ? marker.code : marker.code?.value;
                if (code !== CODE || done.has(marker.startLineNumber)) continue;
                done.add(marker.startLineNumber);
                const lineNumber = marker.startLineNumber;
                const text = model.getLineContent(lineNumber);
                const fixed = convertLegacyInputsInLine(text);
                if (!fixed.converted) continue;
                actions.push({
                    title: 'Convert to #UI',
                    kind: 'quickfix',
                    diagnostics: [marker],
                    isPreferred: true,
                    edit: {
                        edits: [{
                            resource: model.uri,
                            versionId: model.getVersionId(),
                            textEdit: {
                                range: new monaco.Range(lineNumber, 1, lineNumber, text.length + 1),
                                text: fixed.line,
                            },
                        }],
                    },
                });
            }
            return { actions, dispose() {} };
        },
    }, { providedCodeActionKinds: ['quickfix'] });
}
