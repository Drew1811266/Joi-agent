import type { FormEvent } from "react";

import { BriefWorkspace, type BriefDraft, type ReferenceAssetDraft } from "./BriefWorkspace";
import { EmptyState } from "./EmptyState";
import { MemoryWorkspace, type MemoryCurationDraft } from "./MemoryWorkspace";
import { MetricStrip } from "./MetricStrip";
import { ResearchWorkspace, type ResearchDraft } from "./ResearchWorkspace";
import { StoryboardWorkspace, type StoryboardDraft } from "./StoryboardWorkspace";
import type {
  Asset,
  Brand,
  BriefUnderstandingResult,
  CreativeDirection,
  MemoryCurationResult,
  MemoryEntry,
  ProductUnderstanding,
  Project,
  ProjectVersion,
  ResearchReport,
  ResearchReportResult,
  ShotUpdateInput,
  StoryboardGenerationResult,
  StoryboardWithShots,
} from "../types/joi";

type ProjectWorkspaceProps = {
  activeTab: string;
  assets: Asset[];
  brandDraft: {
    name: string;
    description: string;
  };
  briefDraft: BriefDraft;
  creativeDirections: CreativeDirection[];
  curatingMemory: boolean;
  generatingStoryboard: boolean;
  generatingUnderstanding: boolean;
  generatingResearch: boolean;
  memoryCurationDraft: MemoryCurationDraft;
  memoryCurationResult: MemoryCurationResult | null;
  memoryDraft: {
    content: string;
    source: string;
  };
  memoryEntries: MemoryEntry[];
  onBriefDraftChange: (field: keyof BriefDraft, value: string) => void;
  onBrandDraftChange: (field: "name" | "description", value: string) => void;
  onMemoryCurationDraftChange: (field: keyof MemoryCurationDraft, value: string | boolean) => void;
  onMemoryDraftChange: (field: "content" | "source", value: string) => void;
  onProjectDraftChange: (field: "title" | "advertising_goal" | "duration_seconds", value: string) => void;
  onReferenceAssetDraftChange: (field: keyof ReferenceAssetDraft, value: string) => void;
  onResearchDraftChange: (field: keyof ResearchDraft, value: string) => void;
  onRegenerateShot: (storyboardId: string, shotId: string) => void;
  onStoryboardDraftChange: (field: keyof StoryboardDraft, value: string) => void;
  onSubmitBrand: () => void;
  onSubmitBriefUnderstanding: () => void;
  onSubmitMemory: () => void;
  onSubmitMemoryCandidates: () => void;
  onSubmitProject: () => void;
  onSubmitReferenceAsset: () => void;
  onSubmitResearchReport: () => void;
  onSubmitStoryboard: () => void;
  onUpdateMemoryStatus: (id: string, status: "accepted" | "rejected") => void;
  onUpdateShot: (input: ShotUpdateInput) => void;
  productUnderstandings: ProductUnderstanding[];
  projectDraft: {
    title: string;
    advertising_goal: string;
    duration_seconds: string;
  };
  referenceAssetDraft: ReferenceAssetDraft;
  regeneratingShotId: string | null;
  researchDraft: ResearchDraft;
  researchReports: ResearchReport[];
  researchResult: ResearchReportResult | null;
  savingShotId: string | null;
  selectedBrand: Brand | null;
  selectedProject: Project | null;
  storyboardDraft: StoryboardDraft;
  storyboardResult: StoryboardGenerationResult | null;
  storyboards: StoryboardWithShots[];
  understandingResult: BriefUnderstandingResult | null;
  versions: ProjectVersion[];
};

export function ProjectWorkspace({
  activeTab,
  assets,
  brandDraft,
  briefDraft,
  creativeDirections,
  curatingMemory,
  generatingStoryboard,
  generatingUnderstanding,
  generatingResearch,
  memoryCurationDraft,
  memoryCurationResult,
  memoryDraft,
  memoryEntries,
  onBriefDraftChange,
  onBrandDraftChange,
  onMemoryCurationDraftChange,
  onMemoryDraftChange,
  onProjectDraftChange,
  onReferenceAssetDraftChange,
  onResearchDraftChange,
  onRegenerateShot,
  onStoryboardDraftChange,
  onSubmitBrand,
  onSubmitBriefUnderstanding,
  onSubmitMemory,
  onSubmitMemoryCandidates,
  onSubmitProject,
  onSubmitReferenceAsset,
  onSubmitResearchReport,
  onSubmitStoryboard,
  onUpdateMemoryStatus,
  onUpdateShot,
  productUnderstandings,
  projectDraft,
  referenceAssetDraft,
  regeneratingShotId,
  researchDraft,
  researchReports,
  researchResult,
  savingShotId,
  selectedBrand,
  selectedProject,
  storyboardDraft,
  storyboardResult,
  storyboards,
  understandingResult,
  versions,
}: ProjectWorkspaceProps) {
  return (
    <main aria-label="Project workspace" className="workspace-main">
      <section className="workspace-header">
        <div>
          <p className="eyebrow">{activeTab}</p>
          <h1>{selectedProject?.title ?? "Create a fashion advertising project"}</h1>
          <p className="muted">
            {selectedProject
              ? selectedProject.advertising_goal
              : "Set up brand and project context before generating briefs, storyboards, prompts, and reports."}
          </p>
        </div>
        <MetricStrip
          metrics={[
            { label: "Assets", value: assets.length },
            { label: "Memory", value: memoryEntries.length },
            { label: "Versions", value: versions.length },
            { label: "Duration", value: selectedProject ? `${selectedProject.duration_seconds}s` : "--" },
          ]}
        />
      </section>

      {activeTab === "Overview" ? (
        <div className="workspace-grid">
          <section className="workspace-panel">
            <h2>Brand Setup</h2>
            <form onSubmit={submit(onSubmitBrand)}>
              <label>
                Brand name
                <input
                  onChange={(event) => onBrandDraftChange("name", event.target.value)}
                  placeholder="Atelier Joi"
                  value={brandDraft.name}
                />
              </label>
              <label>
                Description
                <textarea
                  onChange={(event) => onBrandDraftChange("description", event.target.value)}
                  placeholder="Editorial womenswear, premium fabrics, clean studio lighting"
                  rows={3}
                  value={brandDraft.description}
                />
              </label>
              <button type="submit">{selectedBrand ? "Update Brand" : "Create Brand"}</button>
            </form>
          </section>

          <section className="workspace-panel">
            <h2>Project Setup</h2>
            <form onSubmit={submit(onSubmitProject)}>
              <label>
                Project title
                <input
                  disabled={!selectedBrand}
                  onChange={(event) => onProjectDraftChange("title", event.target.value)}
                  placeholder="15s spring launch film"
                  value={projectDraft.title}
                />
              </label>
              <label>
                Advertising goal
                <textarea
                  disabled={!selectedBrand}
                  onChange={(event) => onProjectDraftChange("advertising_goal", event.target.value)}
                  placeholder="Drive awareness for the new outerwear collection"
                  rows={3}
                  value={projectDraft.advertising_goal}
                />
              </label>
              <label>
                Duration seconds
                <input
                  disabled={!selectedBrand}
                  min="1"
                  onChange={(event) => onProjectDraftChange("duration_seconds", event.target.value)}
                  type="number"
                  value={projectDraft.duration_seconds}
                />
              </label>
              <button disabled={!selectedBrand} type="submit">
                {selectedProject ? "Update Project" : "Create Project"}
              </button>
            </form>
          </section>

          <section className="workspace-panel wide">
            <h2>Workflow Map</h2>
            <div className="workflow-map">
              {["Brief", "Research", "Creative Direction", "Storyboard", "Prompts", "Delivery"].map(
                (step) => (
                  <div className="workflow-step" key={step}>
                    <span>{step}</span>
                    <small>{step === "Brief" ? "Next" : "Prepared"}</small>
                  </div>
                ),
              )}
            </div>
          </section>
        </div>
      ) : null}

      {activeTab === "Brief" ? (
        <BriefWorkspace
          assets={assets}
          briefDraft={briefDraft}
          creativeDirections={creativeDirections}
          generatingUnderstanding={generatingUnderstanding}
          onBriefDraftChange={onBriefDraftChange}
          onReferenceAssetDraftChange={onReferenceAssetDraftChange}
          onSubmitBriefUnderstanding={onSubmitBriefUnderstanding}
          onSubmitReferenceAsset={onSubmitReferenceAsset}
          productUnderstandings={productUnderstandings}
          referenceAssetDraft={referenceAssetDraft}
          selectedProject={selectedProject}
          understandingResult={understandingResult}
        />
      ) : null}
      {activeTab === "Research" ? (
        <ResearchWorkspace
          generatingResearch={generatingResearch}
          onResearchDraftChange={onResearchDraftChange}
          onSubmitResearchReport={onSubmitResearchReport}
          researchDraft={researchDraft}
          researchReports={researchReports}
          researchResult={researchResult}
          selectedProject={selectedProject}
        />
      ) : null}
      {activeTab === "Storyboard" ? (
        <StoryboardWorkspace
          generatingStoryboard={generatingStoryboard}
          onRegenerateShot={onRegenerateShot}
          onStoryboardDraftChange={onStoryboardDraftChange}
          onSubmitStoryboard={onSubmitStoryboard}
          onUpdateShot={onUpdateShot}
          regeneratingShotId={regeneratingShotId}
          savingShotId={savingShotId}
          selectedProject={selectedProject}
          storyboardDraft={storyboardDraft}
          storyboardResult={storyboardResult}
          storyboards={storyboards}
        />
      ) : null}
      {activeTab === "Assets" ? <AssetsPanel assets={assets} /> : null}
      {activeTab === "Memory" ? (
        <MemoryWorkspace
          curatingMemory={curatingMemory}
          memoryCurationDraft={memoryCurationDraft}
          memoryCurationResult={memoryCurationResult}
          memoryDraft={memoryDraft}
          memoryEntries={memoryEntries}
          onMemoryCurationDraftChange={onMemoryCurationDraftChange}
          onMemoryDraftChange={onMemoryDraftChange}
          onSubmitMemory={onSubmitMemory}
          onSubmitMemoryCandidates={onSubmitMemoryCandidates}
          onUpdateMemoryStatus={onUpdateMemoryStatus}
          selectedProject={selectedProject}
        />
      ) : null}
      {activeTab === "Versions" ? <VersionsPanel versions={versions} /> : null}
      {!["Overview", "Brief", "Research", "Storyboard", "Assets", "Memory", "Versions"].includes(activeTab) ? (
        <EmptyState
          body="This workspace section is reserved for the next content workflow milestone."
          title={`${activeTab} workspace`}
        />
      ) : null}
    </main>
  );
}

function AssetsPanel({ assets }: { assets: Asset[] }) {
  if (assets.length === 0) {
    return <EmptyState body="Imported project assets will appear here." title="No assets yet" />;
  }
  return (
    <section className="workspace-panel wide">
      <h2>Assets</h2>
      <div className="data-list">
        {assets.map((asset) => (
          <article className="data-row" key={asset.id}>
            <strong>{asset.display_name}</strong>
            <span>{asset.kind}</span>
            <small>{asset.mime_type} · {asset.relative_path}</small>
          </article>
        ))}
      </div>
    </section>
  );
}

function VersionsPanel({ versions }: { versions: ProjectVersion[] }) {
  if (versions.length === 0) {
    return <EmptyState body="Use Save Snapshot after selecting a project." title="No versions yet" />;
  }
  return (
    <section className="workspace-panel wide">
      <h2>Versions</h2>
      <div className="data-list">
        {versions.map((version) => (
          <article className="data-row" key={version.id}>
            <strong>Version {version.version_number}</strong>
            <span>{version.label || "Untitled snapshot"}</span>
            <small>{version.change_reason || "No change reason"}</small>
          </article>
        ))}
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
