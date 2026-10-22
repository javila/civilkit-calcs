<template>
  <div class="rb" role="toolbar">
    <div class="rb-tabs" role="tablist">
      <button v-for="t in RIBBON" :key="t.id" class="rb-tab" :class="{ on: t.id === active }" role="tab"
        :aria-selected="t.id === active" :data-ribbon-tab="t.id" @click="active = t.id">{{ t.label }}</button>
    </div>
    <div class="rb-panel" role="tabpanel" :data-ribbon-panel="active">
      <div v-for="g in tab.groups" :key="g.name" class="rb-group">
        <div class="rb-btns" :class="{ compact: !g.buttons[0].icon }">
          <button v-for="b in g.buttons" :key="b.id" class="rb-btn" :class="{ big: !!b.icon, on: isOn(b) }"
            :title="b.title || b.label" :data-ribbon-btn="b.id" @mousedown.prevent @click="fire(b, $event)">
            <svg v-if="b.icon" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor"
              stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" v-html="ICONS[b.icon]"></svg>
            <span class="rb-label">{{ b.label }}</span>
          </button>
        </div>
        <div class="rb-gname">{{ g.name }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue';
import { ICONS, RIBBON, insertAtCursor } from './ribbon-data';
import type { RibbonButton, RibbonCtx } from './ribbon-data';

const props = defineProps<{ ctx: Omit<RibbonCtx, 'anchor'> }>();
const active = ref('home');
const tab = computed(() => RIBBON.find(t => t.id === active.value) ?? RIBBON[0]);
// Toggle states (theme, result mode, pane) live outside Vue; re-read them on a slow tick and after clicks.
const tick = ref(0);
const timer = setInterval(() => tick.value++, 500);
onBeforeUnmount(() => clearInterval(timer));

function isOn(b: RibbonButton): boolean {
    void tick.value;
    try {
        return !!b.active?.({ ...props.ctx, anchor: document.body });
    } catch {
        return false;
    }
}

function fire(b: RibbonButton, ev: MouseEvent): void {
    const c = { ...props.ctx, anchor: ev.currentTarget as HTMLElement };
    if (b.insertText !== undefined) insertAtCursor(c.editor(), b.insertText);
    else b.run?.(c);
    tick.value++;
}
</script>

<style>
.rb{flex:none;font-size:12px;user-select:none;
  background:var(--vscode-sideBar-background,#f3f3f3);color:var(--vscode-foreground,#333);
  border-bottom:1px solid var(--vscode-panel-border,#ddd)}
.rb-tabs{display:flex;overflow-x:auto;padding:0 4px;background:var(--vscode-titleBar-activeBackground,var(--vscode-sideBar-background,#f3f3f3));scrollbar-width:none}
.rb-tab{flex:none;height:26px;padding:0 14px;border:0;border-bottom:2px solid transparent;background:none;color:inherit;font:inherit;font-size:13px;cursor:pointer}
.rb-tab:hover{background:var(--vscode-toolbar-hoverBackground,rgba(0,0,0,.08))}
.rb-tab.on{border-bottom-color:var(--vscode-focusBorder,#0060c0);font-weight:600}
.rb-panel{display:flex;align-items:stretch;gap:0;height:84px;padding:3px 4px 0;overflow-x:auto;overflow-y:hidden;
  background:var(--vscode-editor-background,#fff);border-top:1px solid var(--vscode-panel-border,#ddd)}
.rb-group{flex:none;display:flex;flex-direction:column;padding:0 6px;border-right:1px solid var(--vscode-panel-border,#ddd)}
.rb-group:last-child{border-right:0}
.rb-btns{flex:1;display:flex;align-items:flex-start;gap:2px;min-height:0}
.rb-btns.compact{flex-wrap:wrap;align-content:flex-start;max-width:640px;gap:2px;padding-top:2px}
.rb-btn{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:2px;border:1px solid transparent;border-radius:3px;
  background:none;color:inherit;font:inherit;cursor:pointer;padding:2px 6px}
.rb-btn.big{min-width:46px;height:52px}
.rb-btn:not(.big){min-width:26px;height:24px;font-size:13px;font-family:var(--vscode-editor-font-family,monospace)}
.rb-btn:hover{background:var(--vscode-toolbar-hoverBackground,rgba(0,0,0,.08));border-color:var(--vscode-panel-border,#ccc)}
.rb-btn.on{background:var(--vscode-toolbar-activeBackground,rgba(0,96,192,.18));border-color:var(--vscode-focusBorder,#0060c0)}
.rb-btn.big .rb-label{font-size:11px;line-height:1.1;white-space:nowrap}
.rb-gname{flex:none;text-align:center;font-size:10px;opacity:.65;padding:1px 0 2px;white-space:nowrap}
@media (max-width:700px){
  .rb-panel{height:74px}
  .rb-btn.big{min-width:52px}
  .rb-btns.compact{flex-wrap:nowrap;max-width:none}
}
</style>
