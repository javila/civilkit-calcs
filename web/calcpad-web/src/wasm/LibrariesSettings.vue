<template>
  <!-- calcpad-rs: Settings > Libraries, the script files worksheets may load (library-list.ts). -->
  <section class="settings-section cprs-libraries">
    <h3 class="section-header" @click="open = !open">
      <span class="expand-icon" :class="{ collapsed: !open }">&#x25BC;</span>
      Libraries
    </h3>
    <div v-show="open" class="section-body">
      <p class="cprs-hint">
        Worksheet scripts may load only these files, each from its exact address and checked
        against its hash. Anything else is blocked and the preview names it.
      </p>
      <div v-for="lib in libs" :key="lib.id" class="cprs-lib" :data-library="lib.id">
        <div class="cprs-lib-head">
          <span><b>{{ lib.name }}</b> {{ lib.version }}</span>
          <button class="btn cprs-remove" @click="remove(lib.id)">Remove</button>
        </div>
        <div v-for="f in lib.files" :key="f.url" class="cprs-file">
          <code class="cprs-url">{{ f.url }}</code>
          <code class="cprs-hash">{{ f.integrity }}</code>
        </div>
      </div>
      <div v-if="removed.length" class="cprs-removed">
        <div class="cprs-sub">Removed</div>
        <div v-for="lib in removed" :key="lib.id" class="cprs-lib-head">
          <span>{{ lib.name }} {{ lib.version }}</span>
          <button class="btn cprs-restore" @click="restore(lib.id)">Restore</button>
        </div>
      </div>
      <div class="cprs-add">
        <div class="cprs-sub">Add library</div>
        <div class="cprs-row">
          <input v-model="url" class="cprs-url-input" type="url" placeholder="https://cdn.jsdelivr.net/npm/name@1.2.3/dist/name.min.js" />
          <button class="btn cprs-fetch" :disabled="busy || !url.trim()" @click="fetchFile">Fetch</button>
        </div>
        <p v-if="error" class="cprs-error">{{ error }}</p>
        <div v-if="pending" class="cprs-pending">
          <div>Size: <span class="cprs-size">{{ pending.size.toLocaleString() }}</span> bytes</div>
          <div>Hash: <code class="cprs-hash cprs-new-hash">{{ pending.integrity }}</code></div>
          <label>Name <input v-model="pending.name" class="cprs-name" /></label>
          <label>Version <input v-model="pending.version" class="cprs-version" /></label>
          <label><input v-model="pending.module" type="checkbox" /> ES module</label>
          <div class="cprs-row">
            <button class="btn cprs-confirm" :disabled="!pending.name.trim()" @click="confirmAdd">Add to the list</button>
            <button class="btn cprs-cancel" @click="pending = null">Cancel</button>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import {
  addLibrary, guessNameVersion, libraries, onLibrariesChanged, removeLibrary, removedBuiltins,
  restoreLibrary, sriHash, type ScriptLibrary,
} from './library-list'

const open = ref(false)
const libs = ref<ScriptLibrary[]>(libraries())
const removed = ref<ScriptLibrary[]>(removedBuiltins())
const url = ref('')
const busy = ref(false)
const error = ref('')
const pending = ref<null | { url: string; size: number; integrity: string; name: string; version: string; module: boolean }>(null)

const refresh = () => {
  libs.value = libraries()
  removed.value = removedBuiltins()
}
const off = onLibrariesChanged(refresh)
onBeforeUnmount(off)

function remove(id: string) {
  removeLibrary(id)
}

function restore(id: string) {
  restoreLibrary(id)
}

/** Fetches the file once, to show its size and hash before anything is added. */
async function fetchFile() {
  error.value = ''
  pending.value = null
  const u = url.value.trim()
  let parsed: URL
  try {
    parsed = new URL(u)
  } catch {
    error.value = 'Enter the full address of one file, starting with https://.'
    return
  }
  if (parsed.protocol !== 'https:') {
    error.value = 'Only https:// addresses can be added.'
    return
  }
  if (parsed.pathname.endsWith('/')) {
    error.value = 'Enter the address of one file, not a folder or a whole site.'
    return
  }
  busy.value = true
  try {
    const res = await fetch(u, { credentials: 'omit', cache: 'no-store' })
    if (!res.ok) throw new Error(`the server answered ${res.status}`)
    const bytes = await res.arrayBuffer()
    const { name, version } = guessNameVersion(u)
    pending.value = {
      url: u,
      size: bytes.byteLength,
      integrity: await sriHash(bytes),
      name,
      version,
      module: /\+esm$|\.mjs$/.test(parsed.pathname),
    }
  } catch (e) {
    error.value = `Could not fetch the file (${e instanceof Error ? e.message : String(e)}). `
      + 'The site must allow cross-origin requests, which public CDNs do.'
  } finally {
    busy.value = false
  }
}

function confirmAdd() {
  const p = pending.value
  if (!p) return
  addLibrary({
    id: `user-${Date.now()}`,
    name: p.name.trim(),
    version: p.version.trim(),
    files: [{ url: p.url, integrity: p.integrity, size: p.size, ...(p.module ? { module: true } : {}) }],
  })
  pending.value = null
  url.value = ''
}
</script>

<style scoped>
.cprs-hint { margin: 0 0 8px; opacity: .8; font-size: 12px; }
.cprs-lib { border-top: 1px solid var(--vscode-panel-border, #ddd); padding: 6px 0; }
.cprs-lib-head { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
.cprs-file { margin-top: 3px; }
.cprs-file code { display: block; font-size: 10.5px; word-break: break-all; }
.cprs-hash { opacity: .7; }
.cprs-sub { margin-top: 10px; font-weight: 600; font-size: 12px; }
.cprs-row { display: flex; gap: 6px; margin-top: 4px; }
.cprs-url-input { flex: 1; min-width: 0; }
.cprs-error { color: var(--vscode-errorForeground, #c00); font-size: 12px; }
.cprs-pending { display: flex; flex-direction: column; gap: 4px; margin-top: 6px; font-size: 12px; }
.cprs-pending code { word-break: break-all; }
</style>
