<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useData, withBase } from 'vitepress'
import { enqueue } from '../live'

/**
 * A scene from `examples/scenes`, running the real library through WebAssembly. It starts
 * by itself when it comes near the viewport (one demo at a time, see `live.ts`) and
 * follows the site's light/dark switch without losing its state.
 */
const props = defineProps<{
  /** A scene name from `examples/scenes/scenes.rs`. */
  scene: string
  /** A style preset: `compact-sharp`, `comfortable-rounded`, `large-borderless`. */
  look?: string
  /** A fixed theme setting (`dracula`); otherwise GitHub Light/Dark follow the site. */
  theme?: string
  /** A brand accent, `rrggbb`. */
  accent?: string
  /** Height in CSS pixels. */
  height?: string | number
  alt: string
  caption?: string
}>()

const { isDark } = useData()
const root = ref<HTMLElement | null>(null)
const frame = ref<HTMLIFrameElement | null>(null)
const status = ref<'waiting' | 'loading' | 'ready' | 'failed'>('waiting')
const error = ref('')

const height = computed(() => Number(props.height ?? 420))
const themeId = computed(() => props.theme ?? (isDark.value ? 'github-dark' : 'github-light'))
const dark = computed(() => !/light|alucard/.test(themeId.value))
const src = computed(() => {
  const query = new URLSearchParams({ scene: props.scene, theme: themeId.value })
  if (props.look) query.set('style', props.look)
  if (props.accent) query.set('accent', props.accent.replace('#', ''))
  // `index.html` explicitly: the dev server would route `/demo/` to the site's 404.
  return withBase(`/demo/index.html?${query}`)
})
// The iframe URL is fixed once loaded; theme changes go through postMessage.
const loadedSrc = ref('')

let settle: (() => void) | null = null
let observer: IntersectionObserver | null = null

function onMessage(event: MessageEvent) {
  if (!frame.value || event.source !== frame.value.contentWindow) return
  const data = event.data as { type?: string; message?: string }
  if (data?.type === 'egui-workbench:ready') {
    status.value = 'ready'
    settle?.()
  } else if (data?.type === 'egui-workbench:error') {
    status.value = 'failed'
    error.value = data.message ?? 'The demo could not start.'
    settle?.()
  }
}

function start(): Promise<void> {
  return new Promise((resolve) => {
    settle = () => {
      settle = null
      resolve()
    }
    status.value = 'loading'
    loadedSrc.value = src.value
    // Never hold the queue forever (a blocked WebGL context, a slow network).
    setTimeout(() => settle?.(), 30_000)
  })
}

onMounted(() => {
  window.addEventListener('message', onMessage)
  observer = new IntersectionObserver(
    (entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        observer?.disconnect()
        enqueue(start)
      }
    },
    { rootMargin: '400px 0px' },
  )
  if (root.value) observer.observe(root.value)
})

onBeforeUnmount(() => {
  window.removeEventListener('message', onMessage)
  observer?.disconnect()
  settle?.()
})

watch(themeId, (theme) => {
  frame.value?.contentWindow?.postMessage({ type: 'egui-workbench:theme', theme }, '*')
})
</script>

<template>
  <figure ref="root" class="wb-live">
    <div
      class="wb-live-stage"
      :class="{ dark, ready: status === 'ready' }"
      :style="{ '--wb-live-height': `${height}px` }"
    >
      <iframe
        v-if="loadedSrc"
        ref="frame"
        :src="loadedSrc"
        :title="alt"
        allow="clipboard-read; clipboard-write"
      />
      <div v-if="status !== 'ready'" class="wb-live-status" role="status">
        <template v-if="status === 'failed'">{{ error }}</template>
        <template v-else>
          <span class="wb-live-spinner" aria-hidden="true" />
          <span>{{ status === 'loading' ? 'Starting the live demo…' : 'Live demo' }}</span>
        </template>
      </div>
    </div>
    <figcaption>
      <span v-if="caption">{{ caption }}</span>
      <span class="wb-live-actions">
        <a :href="src" target="_blank" rel="noopener">Open in a tab ↗</a>
      </span>
    </figcaption>
  </figure>
</template>
