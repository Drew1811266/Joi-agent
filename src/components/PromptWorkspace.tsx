import { useEffect, useMemo, useState, type FormEvent } from "react";

import type {
  PromptAdapterProfile,
  PromptPackageUpdateInput,
  PromptPackageView,
  Project,
  StoryboardWithShots,
} from "../types/joi";

export type PromptDraft = {
  selected_video_platforms: string[];
  selected_image_platforms: string[];
  selected_shot_ids: string[];
  image_brief: string;
  user_direction: string;
};

type PromptWorkspaceProps = {
  adapterProfiles: PromptAdapterProfile[];
  generatingPrompts: boolean;
  onCopyPrompt: (copyText: string, packageId: string) => void;
  onPromptDraftChange: (field: keyof PromptDraft, value: string | string[]) => void;
  onSubmitImagePrompts: () => void;
  onSubmitVideoPrompts: () => void;
  onUpdatePromptPackage: (input: PromptPackageUpdateInput) => void;
  promptDraft: PromptDraft;
  promptPackages: PromptPackageView[];
  savingPromptId: string | null;
  selectedProject: Project | null;
  storyboards: StoryboardWithShots[];
};

type PromptEditDraft = {
  prompt_text: string;
  negative_prompt: string;
  is_locked: boolean;
};

export function PromptWorkspace({
  adapterProfiles,
  generatingPrompts,
  onCopyPrompt,
  onPromptDraftChange,
  onSubmitImagePrompts,
  onSubmitVideoPrompts,
  onUpdatePromptPackage,
  promptDraft,
  promptPackages,
  savingPromptId,
  selectedProject,
  storyboards,
}: PromptWorkspaceProps) {
  const [editDrafts, setEditDrafts] = useState<Record<string, PromptEditDraft>>({});
  const videoProfiles = adapterProfiles.filter((profile) => profile.modality === "video");
  const imageProfiles = adapterProfiles.filter((profile) => profile.modality === "image");
  const shotChoices = useMemo(() => latestShotChoices(storyboards), [storyboards]);

  useEffect(() => {
    setEditDrafts((current) => {
      const next: Record<string, PromptEditDraft> = {};
      for (const item of promptPackages) {
        next[item.package.id] = current[item.package.id] ?? {
          prompt_text: item.package.prompt_text,
          negative_prompt: item.package.negative_prompt,
          is_locked: item.package.is_locked,
        };
      }
      return next;
    });
  }, [promptPackages]);

  function toggleArray(field: keyof PromptDraft, id: string, checked: boolean) {
    const current = arrayValue(promptDraft[field]);
    const next = checked ? [...new Set([...current, id])] : current.filter((item) => item !== id);
    onPromptDraftChange(field, next);
  }

  function updateEditDraft(packageId: string, field: keyof PromptEditDraft, value: string | boolean) {
    setEditDrafts((drafts) => ({
      ...drafts,
      [packageId]: {
        ...drafts[packageId],
        [field]: value,
      },
    }));
  }

  function submitPromptEdit(item: PromptPackageView) {
    const draft = editDrafts[item.package.id];
    if (!draft) {
      return;
    }
    onUpdatePromptPackage({
      id: item.package.id,
      prompt_text: draft.prompt_text,
      negative_prompt: draft.negative_prompt,
      parameters_json: item.package.parameters_json,
      is_locked: draft.is_locked,
    });
  }

  const canGenerateVideo =
    Boolean(selectedProject) &&
    promptDraft.selected_video_platforms.length > 0 &&
    promptDraft.selected_shot_ids.length > 0 &&
    !generatingPrompts;
  const canGenerateImage =
    Boolean(selectedProject) &&
    promptDraft.selected_image_platforms.length > 0 &&
    promptDraft.image_brief.trim().length > 0 &&
    !generatingPrompts;

  return (
    <div className="prompt-layout">
      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>Prompt Generator</h2>
          <span className="muted">
            {promptPackages.length > 0
              ? `${promptPackages.length} prompt package(s)`
              : "No prompt packages yet"}
          </span>
        </div>

        <form className="prompt-generator-form" onSubmit={submit(onSubmitVideoPrompts)}>
          <label className="wide-field">
            Prompt direction
            <textarea
              disabled={!selectedProject || generatingPrompts}
              onChange={(event) => onPromptDraftChange("user_direction", event.target.value)}
              placeholder="Keep output production-ready."
              rows={3}
              value={promptDraft.user_direction}
            />
          </label>

          <fieldset>
            <legend>Storyboard shots</legend>
            {shotChoices.length > 0 ? (
              <div className="prompt-option-grid">
                {shotChoices.map((shot) => (
                  <label className="checkbox-label" key={shot.id}>
                    <input
                      checked={promptDraft.selected_shot_ids.includes(shot.id)}
                      disabled={!selectedProject || generatingPrompts}
                      onChange={(event) =>
                        toggleArray("selected_shot_ids", shot.id, event.target.checked)
                      }
                      type="checkbox"
                    />
                    {shot.label}
                  </label>
                ))}
              </div>
            ) : (
              <p className="muted">No storyboard shots available.</p>
            )}
          </fieldset>

          <fieldset>
            <legend>Video adapters</legend>
            <div className="prompt-option-grid">
              {videoProfiles.map((profile) => (
                <label className="checkbox-label" key={profile.id}>
                  <input
                    checked={promptDraft.selected_video_platforms.includes(profile.id)}
                    disabled={!selectedProject || generatingPrompts}
                    onChange={(event) =>
                      toggleArray("selected_video_platforms", profile.id, event.target.checked)
                    }
                    type="checkbox"
                  />
                  {profile.display_name}
                </label>
              ))}
            </div>
          </fieldset>

          <button disabled={!canGenerateVideo} type="submit">
            {generatingPrompts ? "Generating" : "Generate Video Prompts"}
          </button>
        </form>

        <form className="prompt-generator-form image" onSubmit={submit(onSubmitImagePrompts)}>
          <label className="wide-field">
            Image brief
            <textarea
              disabled={!selectedProject || generatingPrompts}
              onChange={(event) => onPromptDraftChange("image_brief", event.target.value)}
              placeholder="Full-body ecommerce model photo, warm studio."
              rows={3}
              value={promptDraft.image_brief}
            />
          </label>
          <fieldset>
            <legend>Image adapters</legend>
            <div className="prompt-option-grid">
              {imageProfiles.map((profile) => (
                <label className="checkbox-label" key={profile.id}>
                  <input
                    checked={promptDraft.selected_image_platforms.includes(profile.id)}
                    disabled={!selectedProject || generatingPrompts}
                    onChange={(event) =>
                      toggleArray("selected_image_platforms", profile.id, event.target.checked)
                    }
                    type="checkbox"
                  />
                  {profile.display_name}
                </label>
              ))}
            </div>
          </fieldset>
          <button disabled={!canGenerateImage} type="submit">
            {generatingPrompts ? "Generating" : "Generate Image Prompts"}
          </button>
        </form>
      </section>

      <section className="workspace-panel wide">
        <h2>Prompt Packages</h2>
        {promptPackages.length > 0 ? (
          <div className="prompt-package-list">
            {promptPackages.map((item) => {
              const draft = editDrafts[item.package.id] ?? {
                prompt_text: item.package.prompt_text,
                negative_prompt: item.package.negative_prompt,
                is_locked: item.package.is_locked,
              };
              return (
                <article className="prompt-package-row" key={item.package.id}>
                  <div className="prompt-package-heading">
                    <div>
                      <strong>{firstPromptLine(item.package.prompt_text)}</strong>
                      <span>
                        {item.adapter_display_name} · {item.package.modality}
                      </span>
                    </div>
                    <small>{shotLabel(item, shotChoices)}</small>
                  </div>

                  {item.missing_fields.length > 0 ? (
                    <div className="tag-list">
                      {item.missing_fields.map((field) => (
                        <span key={field}>{field}</span>
                      ))}
                    </div>
                  ) : null}

                  <form className="prompt-edit-form" onSubmit={submit(() => submitPromptEdit(item))}>
                    <label className="wide-field">
                      Prompt text
                      <textarea
                        onChange={(event) =>
                          updateEditDraft(item.package.id, "prompt_text", event.target.value)
                        }
                        rows={5}
                        value={draft.prompt_text}
                      />
                    </label>
                    <label className="wide-field">
                      Negative prompt
                      <textarea
                        onChange={(event) =>
                          updateEditDraft(item.package.id, "negative_prompt", event.target.value)
                        }
                        rows={3}
                        value={draft.negative_prompt}
                      />
                    </label>
                    <label className="checkbox-label">
                      <input
                        checked={draft.is_locked}
                        onChange={(event) =>
                          updateEditDraft(item.package.id, "is_locked", event.target.checked)
                        }
                        type="checkbox"
                      />
                      Lock prompt
                    </label>
                    <div className="prompt-actions">
                      <button disabled={savingPromptId === item.package.id} type="submit">
                        {savingPromptId === item.package.id ? "Saving" : "Save Prompt"}
                      </button>
                      <button
                        onClick={() => onCopyPrompt(item.copy_text, item.package.id)}
                        type="button"
                      >
                        Copy Prompt
                      </button>
                    </div>
                  </form>
                </article>
              );
            })}
          </div>
        ) : (
          <p className="muted">Generated video and image prompts will appear here.</p>
        )}
      </section>
    </div>
  );
}

function latestShotChoices(storyboards: StoryboardWithShots[]): { id: string; label: string }[] {
  const latest = storyboards[storyboards.length - 1];
  if (!latest) {
    return [];
  }
  return [...latest.shots]
    .sort((left, right) => left.shot_number - right.shot_number)
    .map((shot) => ({
      id: shot.id,
      label: `Shot ${shot.shot_number}`,
    }));
}

function shotLabel(item: PromptPackageView, choices: { id: string; label: string }[]): string {
  if (!item.package.shot_id) {
    return "Image brief";
  }
  return choices.find((choice) => choice.id === item.package.shot_id)?.label ?? item.package.shot_id;
}

function firstPromptLine(value: string): string {
  return value.split("\n").find((line) => line.trim())?.trim() ?? "Prompt package";
}

function arrayValue(value: string | string[]): string[] {
  return Array.isArray(value) ? value : [];
}

function submit(action: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    action();
  };
}
