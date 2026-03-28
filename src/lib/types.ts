export type Workspace = {
  id: string
  name: string
  rootPath: string
  createdAt: string
  updatedAt: string
  lastScannedAt?: string | null
}

export type Note = {
  id: string
  workspaceId: string
  title: string
  relativePath: string
  absolutePath: string
  fileName: string
  extension: string
  frontmatterJson?: string | null
  bodyMarkdown: string
  bodyPlaintext?: string | null
  fileCreatedAt?: string | null
  fileModifiedAt?: string | null
  indexedAt: string
  isDeleted: boolean
}
