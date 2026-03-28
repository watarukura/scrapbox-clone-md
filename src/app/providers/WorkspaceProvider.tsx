import { createContext, useContext, useState, useCallback, ReactNode } from "react"
import { invoke } from "@tauri-apps/api/core"
import type { Workspace } from "../../lib/types"

type WorkspaceContextType = {
  workspace: Workspace | null
  openWorkspace: (rootPath: string) => Promise<void>
}

const WorkspaceContext = createContext<WorkspaceContextType | null>(null)

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [workspace, setWorkspace] = useState<Workspace | null>(null)

  const openWorkspace = useCallback(async (rootPath: string) => {
    const ws = await invoke<Workspace>("open_workspace", { rootPath })
    setWorkspace(ws)
  }, [])

  return (
    <WorkspaceContext.Provider value={{ workspace, openWorkspace }}>
      {children}
    </WorkspaceContext.Provider>
  )
}

export function useWorkspace() {
  const ctx = useContext(WorkspaceContext)
  if (!ctx) throw new Error("useWorkspace must be used within WorkspaceProvider")
  return ctx
}
