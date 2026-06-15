import type { FormEvent } from "react";

import { ReferenceMaterialPanel } from "./ReferenceMaterialPanel";
import { UnderstandingResultPanel } from "./UnderstandingResultPanel";
import type {
  Asset,
  BriefUnderstandingResult,
  CreativeDirection,
  ProductUnderstanding,
  Project,
} from "../types/joi";

export type BriefDraft = {
  brief_text: string;
  product_name: string;
  category: string;
  audience: string;
  target_platforms_text: string;
  selling_points_text: string;
  visual_direction: string;
  constraints_text: string;
};

export type ReferenceAssetDraft = {
  kind: string;
  display_name: string;
  source_uri: string;
};

type BriefWorkspaceProps = {
  assets: Asset[];
  briefDraft: BriefDraft;
  creativeDirections: CreativeDirection[];
  generatingUnderstanding: boolean;
  onBriefDraftChange: (field: keyof BriefDraft, value: string) => void;
  onReferenceAssetDraftChange: (field: keyof ReferenceAssetDraft, value: string) => void;
  onSubmitBriefUnderstanding: () => void;
  onSubmitReferenceAsset: () => void;
  productUnderstandings: ProductUnderstanding[];
  referenceAssetDraft: ReferenceAssetDraft;
  selectedProject: Project | null;
  understandingResult: BriefUnderstandingResult | null;
};

export function BriefWorkspace({
  assets,
  briefDraft,
  creativeDirections,
  generatingUnderstanding,
  onBriefDraftChange,
  onReferenceAssetDraftChange,
  onSubmitBriefUnderstanding,
  onSubmitReferenceAsset,
  productUnderstandings,
  referenceAssetDraft,
  selectedProject,
  understandingResult,
}: BriefWorkspaceProps) {
  return (
    <div className="brief-layout">
      <section className="workspace-panel brief-form-panel">
        <h2>Brief Understanding</h2>
        <form className="brief-form" onSubmit={submit(onSubmitBriefUnderstanding)}>
          <label className="wide-field">
            Project brief
            <textarea
              disabled={!selectedProject || generatingUnderstanding}
              onChange={(event) => onBriefDraftChange("brief_text", event.target.value)}
              rows={4}
              value={briefDraft.brief_text}
            />
          </label>
          <label>
            Product name
            <input
              disabled={!selectedProject || generatingUnderstanding}
              onChange={(event) => onBriefDraftChange("product_name", event.target.value)}
              value={briefDraft.product_name}
            />
          </label>
          <label>
            Product category
            <input
              disabled={!selectedProject || generatingUnderstanding}
              onChange={(event) => onBriefDraftChange("category", event.target.value)}
              value={briefDraft.category}
            />
          </label>
          <label>
            Audience
            <input
              disabled={!selectedProject || generatingUnderstanding}
              onChange={(event) => onBriefDraftChange("audience", event.target.value)}
              value={briefDraft.audience}
            />
          </label>
          <label>
            Target platforms
            <input
              disabled={!selectedProject || generatingUnderstanding}
              onChange={(event) => onBriefDraftChange("target_platforms_text", event.target.value)}
              value={briefDraft.target_platforms_text}
            />
          </label>
          <label>
            Selling points
            <textarea
              disabled={!selectedProject || generatingUnderstanding}
              onChange={(event) => onBriefDraftChange("selling_points_text", event.target.value)}
              rows={3}
              value={briefDraft.selling_points_text}
            />
          </label>
          <label>
            Constraints
            <textarea
              disabled={!selectedProject || generatingUnderstanding}
              onChange={(event) => onBriefDraftChange("constraints_text", event.target.value)}
              rows={3}
              value={briefDraft.constraints_text}
            />
          </label>
          <label className="wide-field">
            Visual direction
            <textarea
              disabled={!selectedProject || generatingUnderstanding}
              onChange={(event) => onBriefDraftChange("visual_direction", event.target.value)}
              rows={3}
              value={briefDraft.visual_direction}
            />
          </label>
          <button disabled={!selectedProject || generatingUnderstanding} type="submit">
            {generatingUnderstanding ? "Generating" : "Generate Understanding"}
          </button>
        </form>
      </section>

      <ReferenceMaterialPanel
        assets={assets}
        draft={referenceAssetDraft}
        onDraftChange={onReferenceAssetDraftChange}
        onSubmit={onSubmitReferenceAsset}
        selectedProject={selectedProject}
      />

      <UnderstandingResultPanel
        creativeDirections={creativeDirections}
        productUnderstandings={productUnderstandings}
        result={understandingResult}
      />
    </div>
  );
}

function submit(action: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    action();
  };
}
