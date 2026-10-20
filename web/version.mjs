// Replaces CalcpadCE's tools/version.mjs (which reads Directory.Build.props): the version of
// the upstream frontend this copy was taken from, with the calcpad-rs engine.
export function resolveVersion() {
    return process.env.CALCPAD_VERSION || '8.0.0-beta3-preview+calcpad-rs';
}
