<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import catalogData from './data/apps.json'
import type { CatalogApp, InstalledApp, InstallManifest, InstallResult, InstallStatus } from './types'

const catalog = catalogData as CatalogApp[]
const activeTab = ref<'catalog' | 'installed'>('catalog')
const search = ref('')
const installedSearch = ref('')
const category = ref('全部')
const selected = ref<Set<string>>(new Set())
const installedApps = ref<InstalledApp[]>([])
const installStatuses = ref<Record<string, InstallStatus>>({})
const installMessages = ref<Record<string, string>>({})
const wingetVersion = ref<string | null>(null)
const scanning = ref(false)
const installing = ref(false)
const statusMessage = ref('正在检查本机环境…')
const fileInput = ref<HTMLInputElement | null>(null)

const categories = computed(() => ['全部', ...Array.from(new Set(catalog.map((item) => item.category)))])

const filteredCatalog = computed(() => {
  const keyword = search.value.trim().toLowerCase()
  return catalog.filter((item) => {
    const categoryMatched = category.value === '全部' || item.category === category.value
    const keywordMatched = !keyword || [item.name, item.description, item.category, ...item.aliases]
      .join(' ')
      .toLowerCase()
      .includes(keyword)
    return categoryMatched && keywordMatched
  })
})

const filteredInstalledApps = computed(() => {
  const keyword = installedSearch.value.trim().toLowerCase()
  if (!keyword) return installedApps.value
  return installedApps.value.filter((item) =>
    [item.name, item.publisher ?? '', item.version ?? ''].join(' ').toLowerCase().includes(keyword),
  )
})

const installedNames = computed(() => installedApps.value.map((item) => item.name.toLowerCase()))

function isInstalled(app: CatalogApp) {
  return app.aliases.some((alias) => {
    const needle = alias.toLowerCase()
    return installedNames.value.some((name) => name === needle || name.includes(needle))
  })
}

function toggleSelected(id: string) {
  const next = new Set(selected.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selected.value = next
}

function clearSelection() {
  selected.value = new Set()
}

async function checkWinget() {
  try {
    const version = await invoke<string>('check_winget')
    wingetVersion.value = version
  } catch (error) {
    wingetVersion.value = null
    console.error(error)
  }
}

async function scanInstalledApps() {
  scanning.value = true
  statusMessage.value = '正在扫描已安装软件…'
  try {
    installedApps.value = await invoke<InstalledApp[]>('scan_installed_apps')
    statusMessage.value = '已识别 ' + installedApps.value.length + ' 个软件记录'
  } catch (error) {
    console.error(error)
    statusMessage.value = '扫描失败，但不影响使用软件安装功能'
  } finally {
    scanning.value = false
  }
}

async function installSelected() {
  if (installing.value || selected.value.size === 0 || !wingetVersion.value) return

  installing.value = true
  const ids = Array.from(selected.value)
  for (const id of ids) installStatuses.value[id] = 'queued'

  for (const id of ids) {
    const app = catalog.find((item) => item.id === id)
    if (!app) continue

    installStatuses.value[id] = 'installing'
    installMessages.value[id] = '正在调用 winget…'

    try {
      const result = await invoke<InstallResult>('install_app', { packageId: app.wingetId })
      installStatuses.value[id] = result.success ? 'success' : 'failed'
      installMessages.value[id] = result.message
    } catch (error) {
      installStatuses.value[id] = 'failed'
      installMessages.value[id] = String(error)
    }
  }

  installing.value = false
  await scanInstalledApps()
}

function exportManifest() {
  if (selected.value.size === 0) return
  const manifest: InstallManifest = {
    version: 1,
    appIds: Array.from(selected.value),
  }
  const blob = new Blob([JSON.stringify(manifest, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = 'pureinstall-list.json'
  anchor.click()
  URL.revokeObjectURL(url)
}

function openImportDialog() {
  fileInput.value?.click()
}

async function importManifest(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return

  try {
    const manifest = JSON.parse(await file.text()) as Partial<InstallManifest>
    if (manifest.version !== 1 || !Array.isArray(manifest.appIds)) throw new Error('不支持的装机清单格式')

    const validIds = new Set(catalog.map((item) => item.id))
    selected.value = new Set(manifest.appIds.filter((id): id is string => typeof id === 'string' && validIds.has(id)))
    statusMessage.value = '已导入 ' + selected.value.size + ' 个可识别的软件'
  } catch (error) {
    statusMessage.value = '导入失败：' + String(error)
  }
}

function statusLabel(id: string) {
  const value = installStatuses.value[id] ?? 'idle'
  if (value === 'queued') return '等待安装'
  if (value === 'installing') return '安装中'
  if (value === 'success') return '安装完成'
  if (value === 'failed') return '安装失败'
  return ''
}

onMounted(async () => {
  await Promise.all([checkWinget(), scanInstalledApps()])
})
</script>

<template>
  <div class="app-shell">
    <header class="topbar">
      <div class="brand">
        <div class="brand-mark">P</div>
        <div>
          <strong>PureInstall</strong>
          <span>纯净软件安装器</span>
        </div>
      </div>
      <div class="top-actions">
        <button class="ghost-button" @click="openImportDialog">导入清单</button>
        <button class="ghost-button" :disabled="selected.size === 0" @click="exportManifest">导出清单</button>
        <input ref="fileInput" class="hidden-input" type="file" accept="application/json,.json" @change="importManifest" />
      </div>
    </header>

    <main>
      <section class="hero">
        <div>
          <p class="eyebrow">NO ADS · NO BUNDLE · NO BACKGROUND SERVICE</p>
          <h1>只装真正需要的软件。</h1>
          <p class="hero-copy">精选可信软件，通过 winget 安装；需要时打开，不常驻、不自启、不打扰。</p>
        </div>
        <div class="environment-card">
          <div>
            <span>winget</span>
            <strong :class="wingetVersion ? 'ok-text' : 'warn-text'">{{ wingetVersion ? '可用' : '不可用' }}</strong>
            <small>{{ wingetVersion || '安装功能已禁用' }}</small>
          </div>
          <div>
            <span>本机软件</span>
            <strong>{{ installedApps.length }}</strong>
            <small>{{ scanning ? '扫描中…' : '条已识别记录' }}</small>
          </div>
        </div>
      </section>

      <section class="workspace">
        <nav class="tabs">
          <button :class="{ active: activeTab === 'catalog' }" @click="activeTab = 'catalog'">安装软件</button>
          <button :class="{ active: activeTab === 'installed' }" @click="activeTab = 'installed'">已安装软件</button>
          <span class="status-line">{{ statusMessage }}</span>
        </nav>

        <div v-if="activeTab === 'catalog'" class="catalog-view">
          <div class="toolbar">
            <div class="search-box">
              <span>⌕</span>
              <input v-model="search" placeholder="搜索软件、分类或用途" />
            </div>
            <div class="categories">
              <button
                v-for="item in categories"
                :key="item"
                :class="{ active: category === item }"
                @click="category = item"
              >
                {{ item }}
              </button>
            </div>
          </div>

          <div class="app-grid">
            <article v-for="app in filteredCatalog" :key="app.id" class="app-card" :class="{ selected: selected.has(app.id) }">
              <div class="app-card-head">
                <div class="app-icon">{{ app.name.slice(0, 1).toUpperCase() }}</div>
                <div class="app-meta">
                  <div class="title-row">
                    <h3>{{ app.name }}</h3>
                    <span v-if="app.recommended" class="recommend-badge">推荐</span>
                  </div>
                  <span>{{ app.category }}</span>
                </div>
                <button
                  class="select-button"
                  :class="{ checked: selected.has(app.id) }"
                  :aria-label="selected.has(app.id) ? '取消选择' : '选择软件'"
                  @click="toggleSelected(app.id)"
                >
                  {{ selected.has(app.id) ? '✓' : '+' }}
                </button>
              </div>

              <p>{{ app.description }}</p>

              <div class="app-card-footer">
                <a :href="app.homepage" target="_blank" rel="noreferrer">官方网站 ↗</a>
                <span v-if="isInstalled(app)" class="installed-badge">已检测安装</span>
                <span v-else-if="installStatuses[app.id]" class="task-badge" :class="installStatuses[app.id]">
                  {{ statusLabel(app.id) }}
                </span>
              </div>
              <small v-if="installMessages[app.id]" class="install-message">{{ installMessages[app.id] }}</small>
            </article>
          </div>

          <div v-if="selected.size > 0" class="selection-bar">
            <div>
              <strong>已选择 {{ selected.size }} 个软件</strong>
              <button class="text-button" :disabled="installing" @click="clearSelection">清空</button>
            </div>
            <button class="primary-button" :disabled="installing || !wingetVersion" @click="installSelected">
              {{ installing ? '正在安装…' : '安装所选 (' + selected.size + ')' }}
            </button>
          </div>
        </div>

        <div v-else class="installed-view">
          <div class="installed-toolbar">
            <div class="search-box installed-search">
              <span>⌕</span>
              <input v-model="installedSearch" placeholder="搜索本机已安装软件" />
            </div>
            <button class="ghost-button" :disabled="scanning" @click="scanInstalledApps">
              {{ scanning ? '扫描中…' : '重新扫描' }}
            </button>
          </div>

          <div class="table-card">
            <table>
              <thead>
                <tr>
                  <th>软件</th>
                  <th>版本</th>
                  <th>发行商</th>
                  <th>安装日期</th>
                  <th>卸载入口</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in filteredInstalledApps" :key="[item.name, item.version, item.publisher].join('|')">
                  <td><strong>{{ item.name }}</strong></td>
                  <td>{{ item.version || '—' }}</td>
                  <td>{{ item.publisher || '未知' }}</td>
                  <td>{{ item.installDate || '—' }}</td>
                  <td>{{ item.hasUninstallCommand ? '已登记' : '未登记' }}</td>
                </tr>
              </tbody>
            </table>
            <div v-if="filteredInstalledApps.length === 0" class="empty-state">没有匹配的软件记录</div>
          </div>
        </div>
      </section>
    </main>

    <footer>
      <span>PureInstall v0.1</span>
      <span>扫描只读 · 不常驻 · 不自启 · 不自动清理</span>
    </footer>
  </div>
</template>
