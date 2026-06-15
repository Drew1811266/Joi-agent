import type { FormEvent } from "react";

import type {
  MemoryCandidateResult,
  MemoryCurationResult,
  MemoryEntry,
  Project,
} from "../types/joi";

export type MemoryCurationDraft = {
  feedback_text: string;
  include_research_reports: boolean;
};

type MemoryWorkspaceProps = {
  curatingMemory: boolean;
  memoryCurationDraft: MemoryCurationDraft;
  memoryCurationResult: MemoryCurationResult | null;
  memoryDraft: {
    content: string;
    source: string;
  };
  memoryEntries: MemoryEntry[];
  onMemoryCurationDraftChange: (
    field: keyof MemoryCurationDraft,
    value: string | boolean,
  ) => void;
  onMemoryDraftChange: (field: "content" | "source", value: string) => void;
  onSubmitMemory: () => void;
  onSubmitMemoryCandidates: () => void;
  onUpdateMemoryStatus: (id: string, status: "accepted" | "rejected") => void;
  selectedProject: Project | null;
};

export function MemoryWorkspace({
  curatingMemory,
  memoryCurationDraft,
  memoryCurationResult,
  memoryDraft,
  memoryEntries,
  onMemoryCurationDraftChange,
  onMemoryDraftChange,
  onSubmitMemory,
  onSubmitMemoryCandidates,
  onUpdateMemoryStatus,
  selectedProject,
}: MemoryWorkspaceProps) {
  const entries = mergedMemoryEntries(memoryEntries, memoryCurationResult);
  const candidatesById = new Map(
    memoryCurationResult?.candidates.map((candidate) => [candidate.entry.id, candidate]) ?? [],
  );

  return (
    <div className="memory-layout">
      <section className="workspace-panel wide">
        <h2>Manual Memory</h2>
        <form className="inline-form" onSubmit={submit(onSubmitMemory)}>
          <label>
            Project memory
            <input
              disabled={!selectedProject}
              onChange={(event) => onMemoryDraftChange("content", event.target.value)}
              placeholder="Keep fabric texture visible in close-up shots"
              value={memoryDraft.content}
            />
          </label>
          <label>
            Source
            <input
              disabled={!selectedProject}
              onChange={(event) => onMemoryDraftChange("source", event.target.value)}
              placeholder="user note"
              value={memoryDraft.source}
            />
          </label>
          <button disabled={!selectedProject} type="submit">
            Add Memory
          </button>
        </form>
      </section>

      <section className="workspace-panel wide">
        <h2>Memory Candidates</h2>
        <form className="memory-candidate-form" onSubmit={submit(onSubmitMemoryCandidates)}>
          <label>
            Feedback for memory
            <textarea
              disabled={!selectedProject || curatingMemory}
              onChange={(event) => onMemoryCurationDraftChange("feedback_text", event.target.value)}
              placeholder="Keep tactile product proof in the opening shot."
              rows={3}
              value={memoryCurationDraft.feedback_text}
            />
          </label>
          <label className="checkbox-label">
            <input
              checked={memoryCurationDraft.include_research_reports}
              disabled={!selectedProject || curatingMemory}
              onChange={(event) =>
                onMemoryCurationDraftChange("include_research_reports", event.target.checked)
              }
              type="checkbox"
            />
            Use research reports
          </label>
          <button
            disabled={!selectedProject || curatingMemory || !canCurate(memoryCurationDraft)}
            type="submit"
          >
            {curatingMemory ? "Generating" : "Generate Memory Candidates"}
          </button>
        </form>
      </section>

      <MemoryStatusSection
        candidatesById={candidatesById}
        entries={entries.filter((entry) => entry.status === "proposed")}
        onUpdateMemoryStatus={onUpdateMemoryStatus}
        status="Proposed"
      />
      <MemoryStatusSection
        candidatesById={candidatesById}
        entries={entries.filter((entry) => entry.status === "accepted")}
        onUpdateMemoryStatus={onUpdateMemoryStatus}
        status="Accepted"
      />
      <MemoryStatusSection
        candidatesById={candidatesById}
        entries={entries.filter((entry) => entry.status === "rejected")}
        onUpdateMemoryStatus={onUpdateMemoryStatus}
        status="Rejected"
      />
    </div>
  );
}

function MemoryStatusSection({
  candidatesById,
  entries,
  onUpdateMemoryStatus,
  status,
}: {
  candidatesById: Map<string, MemoryCandidateResult>;
  entries: MemoryEntry[];
  onUpdateMemoryStatus: (id: string, status: "accepted" | "rejected") => void;
  status: "Proposed" | "Accepted" | "Rejected";
}) {
  return (
    <section className="workspace-panel">
      <h2>{status}</h2>
      {entries.length === 0 ? (
        <p className="muted">No {status.toLowerCase()} memory.</p>
      ) : (
        <div className="data-list compact-data-list">
          {entries.map((entry) => {
            const candidate = candidatesById.get(entry.id);
            return (
              <article className="data-row" key={entry.id}>
                <strong>{entry.content}</strong>
                <span>
                  {entry.scope} · {entry.status}
                  {candidate?.has_conflict ? " · conflict" : ""}
                </span>
                <small>{sourceTrace(entry)}</small>
                {candidate?.reason ? <small>{candidate.reason}</small> : null}
                {candidate?.has_conflict ? (
                  <small>Conflicts: {candidate.conflict_memory_ids.join(", ")}</small>
                ) : null}
                {entry.status === "proposed" ? (
                  <div className="row-actions">
                    <button onClick={() => onUpdateMemoryStatus(entry.id, "accepted")} type="button">
                      Accept
                    </button>
                    <button onClick={() => onUpdateMemoryStatus(entry.id, "rejected")} type="button">
                      Reject
                    </button>
                  </div>
                ) : null}
              </article>
            );
          })}
        </div>
      )}
    </section>
  );
}

function mergedMemoryEntries(
  memoryEntries: MemoryEntry[],
  memoryCurationResult: MemoryCurationResult | null,
): MemoryEntry[] {
  const entries = new Map(memoryEntries.map((entry) => [entry.id, entry]));
  for (const candidate of memoryCurationResult?.candidates ?? []) {
    entries.set(candidate.entry.id, candidate.entry);
  }
  return [...entries.values()];
}

function canCurate(draft: MemoryCurationDraft): boolean {
  return draft.include_research_reports || Boolean(draft.feedback_text.trim());
}

function sourceTrace(entry: MemoryEntry): string {
  const source = entry.source || "unspecified source";
  const trace =
    entry.source_entity_type && entry.source_entity_id
      ? `${entry.source_entity_type}:${entry.source_entity_id}`
      : entry.source_entity_type || "manual";
  return `${source} · ${trace} · ${entry.confidence.toFixed(2)}`;
}

function submit(action: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    action();
  };
}
