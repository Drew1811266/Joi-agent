import type { FormEvent } from "react";

import type { Asset, Project } from "../types/joi";
import type { ReferenceAssetDraft } from "./BriefWorkspace";

type ReferenceMaterialPanelProps = {
  assets: Asset[];
  draft: ReferenceAssetDraft;
  onDraftChange: (field: keyof ReferenceAssetDraft, value: string) => void;
  onSubmit: () => void;
  selectedProject: Project | null;
};

export function ReferenceMaterialPanel({
  assets,
  draft,
  onDraftChange,
  onSubmit,
  selectedProject,
}: ReferenceMaterialPanelProps) {
  const canSubmit = Boolean(selectedProject && draft.display_name.trim() && draft.source_uri.trim());

  return (
    <section className="workspace-panel">
      <h2>Reference Materials</h2>
      <form onSubmit={submit(onSubmit)}>
        <label>
          Reference name
          <input
            disabled={!selectedProject}
            onChange={(event) => onDraftChange("display_name", event.target.value)}
            value={draft.display_name}
          />
        </label>
        <label>
          Reference URL
          <input
            disabled={!selectedProject}
            onChange={(event) => onDraftChange("source_uri", event.target.value)}
            value={draft.source_uri}
          />
        </label>
        <label>
          Reference kind
          <input
            disabled={!selectedProject}
            onChange={(event) => onDraftChange("kind", event.target.value)}
            value={draft.kind}
          />
        </label>
        <button disabled={!canSubmit} type="submit">
          Add Reference
        </button>
      </form>
      <div className="data-list compact-data-list">
        {assets.length === 0 ? (
          <p className="muted">No reference materials yet.</p>
        ) : (
          assets.map((asset) => (
            <article className="data-row" key={asset.id}>
              <strong>{asset.display_name}</strong>
              <span>{asset.kind}</span>
              <small>{asset.source_uri || asset.relative_path}</small>
            </article>
          ))
        )}
      </div>
    </section>
  );
}

function submit(action: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    action();
  };
}
