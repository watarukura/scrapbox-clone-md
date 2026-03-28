import { createContext, useContext, useState, useCallback, useEffect, ReactNode } from "react"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import type { Note } from "../../lib/types"

type NotesContextType = {
  notes: Note[]
  selectedNote: Note | null
  loadNotes: (workspaceId: string) => Promise<void>
  selectNote: (noteId: string) => Promise<void>
  clearSelection: () => void
  refreshNotes: () => Promise<void>
  workspaceId: string | null
}

const NotesContext = createContext<NotesContextType | null>(null)

export function NotesProvider({ children }: { children: ReactNode }) {
  const [notes, setNotes] = useState<Note[]>([])
  const [selectedNote, setSelectedNote] = useState<Note | null>(null)
  const [workspaceId, setWorkspaceId] = useState<string | null>(null)

  const loadNotes = useCallback(async (wsId: string) => {
    setWorkspaceId(wsId)
    const list = await invoke<Note[]>("list_notes", { workspaceId: wsId })
    setNotes(list)
  }, [])

  const selectNote = useCallback(async (noteId: string) => {
    const note = await invoke<Note | null>("get_note", { noteId })
    setSelectedNote(note)
  }, [])

  const clearSelection = useCallback(() => {
    setSelectedNote(null)
  }, [])

  const refreshNotes = useCallback(async () => {
    if (workspaceId) {
      const list = await invoke<Note[]>("list_notes", { workspaceId })
      setNotes(list)
    }
  }, [workspaceId])

  // Listen for file changes from backend
  useEffect(() => {
    const unlisten = listen("notes-changed", () => {
      refreshNotes()
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [refreshNotes])

  return (
    <NotesContext.Provider
      value={{ notes, selectedNote, loadNotes, selectNote, clearSelection, refreshNotes, workspaceId }}
    >
      {children}
    </NotesContext.Provider>
  )
}

export function useNotes() {
  const ctx = useContext(NotesContext)
  if (!ctx) throw new Error("useNotes must be used within NotesProvider")
  return ctx
}
