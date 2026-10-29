// The session values the rum-director mod keeps in $.state, so that a reload
// of the mod (an edit to it reloads it at the turn end) does not lose them.

// A commit that still owes its compaction: the short hash, for the toast.
export type Pending = { hash: string }

declare module 'claude-code' {
  interface PluginState {
    'rum-director': { pending: Pending | null; injectAtTurnEnd: boolean }
  }
}
