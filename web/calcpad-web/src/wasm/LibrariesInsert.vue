<template>
  <!-- calcpad-rs: Insert > Libraries, adds the script tag of a listed library at the cursor. -->
  <div class="cprs-insert-libraries">
    <label class="cprs-insert-label" for="cprs-insert-library">Libraries</label>
    <select id="cprs-insert-library" v-model="chosen" class="cprs-insert-select" @change="insert">
      <option value="">Insert a script library...</option>
      <option v-for="lib in libs" :key="lib.id" :value="lib.id">{{ lib.name }} {{ lib.version }}</option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { insertText, libraries, onLibrariesChanged, type ScriptLibrary } from './library-list'

const emit = defineEmits<{ insertText: [text: string] }>()
const libs = ref<ScriptLibrary[]>(libraries())
const chosen = ref('')
const off = onLibrariesChanged(() => { libs.value = libraries() })
onBeforeUnmount(off)

function insert() {
  const lib = libs.value.find(l => l.id === chosen.value)
  chosen.value = ''
  if (lib) emit('insertText', insertText(lib))
}
</script>

<style scoped>
.cprs-insert-libraries { display: flex; align-items: center; gap: 6px; margin: 6px 12px; font-size: 12px; }
.cprs-insert-select { flex: 1; min-width: 0; }
</style>
