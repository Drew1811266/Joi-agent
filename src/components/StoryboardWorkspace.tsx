import { useMemo, useState, type FormEvent } from "react";

import type {
  Project,
  Shot,
  ShotUpdateInput,
  StoryboardGenerationResult,
  StoryboardShotView,
  StoryboardWithShots,
} from "../types/joi";

export type StoryboardDraft = {
  user_direction: string;
  preferred_duration_seconds: string;
  preferred_shot_count: string;
  regeneration_note: string;
};

type StoryboardWorkspaceProps = {
  generatingStoryboard: boolean;
  onRegenerateShot: (storyboardId: string, shotId: string) => void;
  onStoryboardDraftChange: (field: keyof StoryboardDraft, value: string) => void;
  onSubmitStoryboard: () => void;
  onUpdateShot: (input: ShotUpdateInput) => void;
  regeneratingShotId: string | null;
  savingShotId: string | null;
  selectedProject: Project | null;
  storyboardDraft: StoryboardDraft;
  storyboardResult: StoryboardGenerationResult | null;
  storyboards: StoryboardWithShots[];
};

type ShotEditDraft = {
  duration_seconds: string;
  visual_description: string;
  model_action: string;
  garment_focus: string;
  camera_movement: string;
  scene: string;
  lighting: string;
  transition: string;
  subtitle_or_text: string;
  rationale: string;
  is_locked: boolean;
};

export function StoryboardWorkspace({
  generatingStoryboard,
  onRegenerateShot,
  onStoryboardDraftChange,
  onSubmitStoryboard,
  onUpdateShot,
  regeneratingShotId,
  savingShotId,
  selectedProject,
  storyboardDraft,
  storyboardResult,
  storyboards,
}: StoryboardWorkspaceProps) {
  const [editingShotId, setEditingShotId] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState<ShotEditDraft | null>(null);
  const activeStoryboard = useMemo(
    () => activeStoryboardView(storyboardResult, storyboards),
    [storyboardResult, storyboards],
  );

  function startEdit(shot: StoryboardShotView) {
    setEditingShotId(shot.shot.id);
    setEditDraft({
      duration_seconds: String(shot.shot.duration_seconds),
      visual_description: shot.visual_description,
      model_action: shot.shot.model_action,
      garment_focus: shot.garment_focus,
      camera_movement: shot.shot.camera_movement,
      scene: shot.shot.scene,
      lighting: shot.shot.lighting,
      transition: shot.transition,
      subtitle_or_text: shot.shot.subtitle_or_voiceover,
      rationale: shot.shot.rationale,
      is_locked: shot.shot.is_locked,
    });
  }

  function updateEditDraft(field: keyof ShotEditDraft, value: string | boolean) {
    setEditDraft((draft) => (draft ? { ...draft, [field]: value } : draft));
  }

  function submitShotEdit(shot: StoryboardShotView) {
    if (!editDraft) {
      return;
    }
    const duration = Number(editDraft.duration_seconds);
    onUpdateShot({
      id: shot.shot.id,
      duration_seconds: Number.isFinite(duration) && duration > 0 ? duration : shot.shot.duration_seconds,
      visual_description: editDraft.visual_description,
      model_action: editDraft.model_action,
      garment_focus: editDraft.garment_focus,
      camera_movement: editDraft.camera_movement,
      scene: editDraft.scene,
      lighting: editDraft.lighting,
      transition: editDraft.transition,
      subtitle_or_text: editDraft.subtitle_or_text,
      rationale: editDraft.rationale,
      is_locked: editDraft.is_locked,
    });
    setEditingShotId(null);
    setEditDraft(null);
  }

  return (
    <div className="storyboard-layout">
      <section className="workspace-panel wide">
        <h2>Storyboard Generator</h2>
        <form className="storyboard-toolbar" onSubmit={submit(onSubmitStoryboard)}>
          <label className="wide-field">
            Storyboard direction
            <textarea
              disabled={!selectedProject || generatingStoryboard}
              onChange={(event) => onStoryboardDraftChange("user_direction", event.target.value)}
              placeholder="Make the opening tactile and premium."
              rows={3}
              value={storyboardDraft.user_direction}
            />
          </label>
          <label>
            Duration seconds
            <input
              disabled={!selectedProject || generatingStoryboard}
              min="15"
              max="30"
              onChange={(event) =>
                onStoryboardDraftChange("preferred_duration_seconds", event.target.value)
              }
              type="number"
              value={storyboardDraft.preferred_duration_seconds}
            />
          </label>
          <label>
            Shot count
            <input
              disabled={!selectedProject || generatingStoryboard}
              min="3"
              max="10"
              onChange={(event) =>
                onStoryboardDraftChange("preferred_shot_count", event.target.value)
              }
              type="number"
              value={storyboardDraft.preferred_shot_count}
            />
          </label>
          <button disabled={!selectedProject || generatingStoryboard} type="submit">
            {generatingStoryboard ? "Generating" : "Generate Storyboard"}
          </button>
          <label className="wide-field">
            Regeneration note
            <textarea
              disabled={!selectedProject}
              onChange={(event) => onStoryboardDraftChange("regeneration_note", event.target.value)}
              placeholder="Make shot 2 a clearer macro fabric insert."
              rows={2}
              value={storyboardDraft.regeneration_note}
            />
          </label>
        </form>
      </section>

      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>{activeStoryboard?.storyboard.title ?? "Storyboard"}</h2>
          <span className="muted">
            {activeStoryboard
              ? `${activeStoryboard.shots.length} shots · ${activeStoryboard.storyboard.duration_seconds}s`
              : "No storyboard yet"}
          </span>
        </div>

        {activeStoryboard ? (
          <div className="storyboard-shot-grid">
            {activeStoryboard.shots.map((shot) => (
              <article className="shot-row" key={shot.shot.id}>
                <div className="shot-heading">
                  <strong>Shot {shot.shot.shot_number}</strong>
                  <span>{shot.shot.duration_seconds}s</span>
                </div>

                {editingShotId === shot.shot.id && editDraft ? (
                  <form className="shot-edit-form" onSubmit={submit(() => submitShotEdit(shot))}>
                    <label className="wide-field">
                      Visual description
                      <textarea
                        onChange={(event) =>
                          updateEditDraft("visual_description", event.target.value)
                        }
                        rows={3}
                        value={editDraft.visual_description}
                      />
                    </label>
                    <label>
                      Duration
                      <input
                        min="1"
                        onChange={(event) =>
                          updateEditDraft("duration_seconds", event.target.value)
                        }
                        type="number"
                        value={editDraft.duration_seconds}
                      />
                    </label>
                    <label>
                      Model action
                      <input
                        onChange={(event) => updateEditDraft("model_action", event.target.value)}
                        value={editDraft.model_action}
                      />
                    </label>
                    <label>
                      Garment focus
                      <input
                        onChange={(event) => updateEditDraft("garment_focus", event.target.value)}
                        value={editDraft.garment_focus}
                      />
                    </label>
                    <label>
                      Camera movement
                      <input
                        onChange={(event) =>
                          updateEditDraft("camera_movement", event.target.value)
                        }
                        value={editDraft.camera_movement}
                      />
                    </label>
                    <label>
                      Scene
                      <input
                        onChange={(event) => updateEditDraft("scene", event.target.value)}
                        value={editDraft.scene}
                      />
                    </label>
                    <label>
                      Lighting
                      <input
                        onChange={(event) => updateEditDraft("lighting", event.target.value)}
                        value={editDraft.lighting}
                      />
                    </label>
                    <label>
                      Transition
                      <input
                        onChange={(event) => updateEditDraft("transition", event.target.value)}
                        value={editDraft.transition}
                      />
                    </label>
                    <label>
                      Subtitle or text
                      <input
                        onChange={(event) =>
                          updateEditDraft("subtitle_or_text", event.target.value)
                        }
                        value={editDraft.subtitle_or_text}
                      />
                    </label>
                    <label className="wide-field">
                      Rationale
                      <textarea
                        onChange={(event) => updateEditDraft("rationale", event.target.value)}
                        rows={3}
                        value={editDraft.rationale}
                      />
                    </label>
                    <label className="checkbox-label">
                      <input
                        checked={editDraft.is_locked}
                        onChange={(event) => updateEditDraft("is_locked", event.target.checked)}
                        type="checkbox"
                      />
                      Lock shot
                    </label>
                    <div className="shot-actions">
                      <button disabled={savingShotId === shot.shot.id} type="submit">
                        {savingShotId === shot.shot.id ? "Saving" : "Save Shot"}
                      </button>
                      <button
                        onClick={() => {
                          setEditingShotId(null);
                          setEditDraft(null);
                        }}
                        type="button"
                      >
                        Cancel
                      </button>
                    </div>
                  </form>
                ) : (
                  <>
                    <p>{shot.visual_description}</p>
                    <dl className="shot-meta-grid">
                      <div>
                        <dt>Action</dt>
                        <dd>{shot.shot.model_action}</dd>
                      </div>
                      <div>
                        <dt>Garment</dt>
                        <dd>{shot.garment_focus}</dd>
                      </div>
                      <div>
                        <dt>Camera</dt>
                        <dd>{shot.shot.camera_movement}</dd>
                      </div>
                      <div>
                        <dt>Scene</dt>
                        <dd>{shot.shot.scene}</dd>
                      </div>
                      <div>
                        <dt>Lighting</dt>
                        <dd>{shot.shot.lighting}</dd>
                      </div>
                      <div>
                        <dt>Transition</dt>
                        <dd>{shot.transition}</dd>
                      </div>
                    </dl>
                    <small>{shot.shot.subtitle_or_voiceover}</small>
                    <small>{shot.shot.rationale}</small>
                    <div className="shot-actions">
                      <button onClick={() => startEdit(shot)} type="button">
                        Edit Shot
                      </button>
                      <button
                        disabled={regeneratingShotId === shot.shot.id || shot.shot.is_locked}
                        onClick={() => onRegenerateShot(shot.shot.storyboard_id, shot.shot.id)}
                        type="button"
                      >
                        {regeneratingShotId === shot.shot.id ? "Regenerating" : "Regenerate Shot"}
                      </button>
                    </div>
                  </>
                )}
              </article>
            ))}
          </div>
        ) : (
          <p className="muted">No saved storyboard.</p>
        )}
      </section>
    </div>
  );
}

function activeStoryboardView(
  storyboardResult: StoryboardGenerationResult | null,
  storyboards: StoryboardWithShots[],
): { storyboard: StoryboardWithShots["storyboard"]; shots: StoryboardShotView[] } | null {
  if (storyboardResult) {
    return {
      storyboard: storyboardResult.storyboard,
      shots: storyboardResult.shots,
    };
  }
  const latest = storyboards[storyboards.length - 1];
  if (!latest) {
    return null;
  }
  return {
    storyboard: latest.storyboard,
    shots: latest.shots.map(shotToView),
  };
}

function shotToView(shot: Shot): StoryboardShotView {
  return {
    shot,
    visual_description: shot.description,
    garment_focus: metadataString(shot.metadata_json, "garment_focus"),
    transition: metadataString(shot.metadata_json, "transition"),
  };
}

function metadataString(value: unknown, key: string): string {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return "";
  }
  const record = value as Record<string, unknown>;
  return typeof record[key] === "string" ? record[key] : "";
}

function submit(action: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    action();
  };
}
