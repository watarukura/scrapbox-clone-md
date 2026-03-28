import type { Page } from "@playwright/test"
import * as fs from "node:fs"
import * as path from "node:path"
import * as crypto from "node:crypto"
import { fileURLToPath } from "node:url"

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const FIXTURES_DIR = path.resolve(__dirname, "fixtures", "test-workspace")

type NoteData = {
  id: string
  workspaceId: string
  title: string
  relativePath: string
  absolutePath: string
  fileName: string
  extension: string
  frontmatterJson: string | null
  bodyMarkdown: string
  bodyPlaintext: string
  fileCreatedAt: string
  fileModifiedAt: string
  indexedAt: string
  isDeleted: boolean
}

type WorkspaceData = {
  id: string
  name: string
  rootPath: string
  createdAt: string
  updatedAt: string
  lastScannedAt: string
}

function extractTitle(markdown: string, fileName: string): string {
  const h1Match = markdown.match(/^#\s+(.+)$/m)
  if (h1Match) return h1Match[1].trim()
  return fileName.replace(/\.md$/, "")
}

function stripMarkdown(markdown: string): string {
  return markdown
    .replace(/^#+\s+/gm, "")
    .replace(/\n{2,}/g, "\n")
    .trim()
}

function loadFixtureNotes(workspaceRootPath: string): NoteData[] {
  const files = fs.readdirSync(FIXTURES_DIR).filter((f) => f.endsWith(".md")).sort()
  const workspaceId = "ws-fixture-001"
  const now = new Date().toISOString()

  return files.map((fileName) => {
    const filePath = path.join(FIXTURES_DIR, fileName)
    const content = fs.readFileSync(filePath, "utf-8")
    const title = extractTitle(content, fileName)
    const id = crypto.createHash("md5").update(fileName).digest("hex").slice(0, 12)

    return {
      id: `note-${id}`,
      workspaceId,
      title,
      relativePath: fileName,
      absolutePath: path.join(workspaceRootPath, fileName),
      fileName,
      extension: ".md",
      frontmatterJson: null,
      bodyMarkdown: content,
      bodyPlaintext: stripMarkdown(content),
      fileCreatedAt: now,
      fileModifiedAt: now,
      indexedAt: now,
      isDeleted: false,
    }
  })
}

function loadFixtureWorkspace(): WorkspaceData {
  const now = new Date().toISOString()
  return {
    id: "ws-fixture-001",
    name: "test-workspace",
    rootPath: FIXTURES_DIR,
    createdAt: now,
    updatedAt: now,
    lastScannedAt: now,
  }
}

export const FIXTURE_WORKSPACE = loadFixtureWorkspace()
export const FIXTURE_NOTES = loadFixtureNotes(FIXTURE_WORKSPACE.rootPath)

/**
 * Inject Tauri IPC layer backed by fixture markdown files.
 * The dialog plugin is the only simulated part — it returns the fixture
 * workspace path instead of opening a native dialog.
 * All note data is derived from actual .md files in e2e/fixtures/test-workspace/.
 */
export async function setupTauriFromFixtures(
  page: Page,
  opts?: {
    workspace?: WorkspaceData
    notes?: NoteData[]
    searchResults?: NoteData[]
  },
) {
  const workspace = opts?.workspace ?? FIXTURE_WORKSPACE
  const notes = opts?.notes ?? FIXTURE_NOTES
  const searchResults = opts?.searchResults

  await page.addInitScript(
    ({ workspace, notes, searchResults }) => {
      const savedNotes = new Map<string, (typeof notes)[0]>()
      for (const n of notes) {
        savedNotes.set(n.id, { ...n })
      }

      ;(window as any).__TAURI_INTERNALS__ = (window as any).__TAURI_INTERNALS__ || {}
      ;(window as any).__TAURI_EVENT_PLUGIN_INTERNALS__ =
        (window as any).__TAURI_EVENT_PLUGIN_INTERNALS__ || {}

      const callbacks = new Map<number, (data: any) => void>()

      function registerCallback(callback: (data: any) => void, once = false): number {
        const id = Math.floor(Math.random() * 1_000_000)
        callbacks.set(id, (data: any) => {
          if (once) callbacks.delete(id)
          return callback?.(data)
        })
        return id
      }

      function unregisterCallback(id: number) {
        callbacks.delete(id)
      }

      ;(window as any).__TAURI_INTERNALS__.transformCallback = registerCallback
      ;(window as any).__TAURI_INTERNALS__.unregisterCallback = unregisterCallback
      ;(window as any).__TAURI_INTERNALS__.callbacks = callbacks
      ;(window as any).__TAURI_INTERNALS__.runCallback = (id: number, data: any) => {
        const cb = callbacks.get(id)
        if (cb) cb(data)
      }
      ;(window as any).__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener = (
        _event: string,
        id: number,
      ) => {
        unregisterCallback(id)
      }

      ;(window as any).__TAURI_INTERNALS__.invoke = async (
        cmd: string,
        args?: Record<string, unknown>,
      ) => {
        switch (cmd) {
          case "open_workspace":
            return workspace

          case "list_notes":
            return Array.from(savedNotes.values()).filter((n) => !n.isDeleted)

          case "get_note": {
            const noteId = args?.noteId as string
            return savedNotes.get(noteId) ?? null
          }

          case "save_note": {
            const id = args?.noteId as string
            const markdown = args?.markdown as string
            const existing = savedNotes.get(id)
            if (existing) {
              const updated = {
                ...existing,
                bodyMarkdown: markdown,
                fileModifiedAt: new Date().toISOString(),
              }
              savedNotes.set(id, updated)
              return updated
            }
            throw new Error(`Note not found: ${id}`)
          }

          case "search_notes": {
            if (searchResults) {
              return searchResults
            }
            const query = ((args?.query as string) || "").toLowerCase()
            return Array.from(savedNotes.values()).filter(
              (n) =>
                !n.isDeleted &&
                (n.title.toLowerCase().includes(query) ||
                  n.bodyPlaintext.toLowerCase().includes(query)),
            )
          }

          case "save_new_file":
            return null

          // Dialog plugin: return fixture workspace path
          case "plugin:dialog|open":
            return workspace.rootPath

          case "plugin:event|listen":
            return registerCallback(() => {})

          case "plugin:event|unlisten":
            return null

          default:
            console.warn(`[E2E Fixture] Unhandled Tauri command: ${cmd}`, args)
            return null
        }
      }

      ;(window as any).__TAURI_INTERNALS__.metadata = {
        currentWindow: { label: "main" },
        currentWebview: { windowLabel: "main", label: "main" },
      }
    },
    { workspace, notes, searchResults: searchResults ?? null },
  )
}
