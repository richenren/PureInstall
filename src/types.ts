export interface CatalogApp {
  id: string
  name: string
  wingetId: string
  category: string
  description: string
  homepage: string
  aliases: string[]
  recommended?: boolean
}

export interface InstalledApp {
  name: string
  version?: string | null
  publisher?: string | null
  installDate?: string | null
  hasUninstallCommand: boolean
}

export type InstallStatus = 'idle' | 'queued' | 'installing' | 'success' | 'failed'

export interface InstallResult {
  packageId: string
  success: boolean
  exitCode?: number | null
  message: string
}

export interface InstallManifest {
  version: 1
  appIds: string[]
}
