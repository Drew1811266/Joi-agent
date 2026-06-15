import { useEffect, useMemo, useState } from "react";

import {
  createBrand,
  createMemoryEntry,
  createProject,
  createReferenceAsset,
  formatError,
  generateBriefUnderstanding,
  generateResearchReport,
  getAgentRuntimeStatus,
  healthCheck,
  listAgentRuns,
  listAssets,
  listBrands,
  listCreativeDirections,
  listMemoryEntries,
  listProductUnderstandings,
  listProjectVersions,
  listProjects,
  listResearchReports,
  saveProjectSnapshot,
  startAgentPlan,
  updateBrand,
  updateProject,
} from "./api/joiApi";
import { AgentPanel } from "./components/AgentPanel";
import { BrandProjectRail } from "./components/BrandProjectRail";
import type { BriefDraft, ReferenceAssetDraft } from "./components/BriefWorkspace";
import { ProjectWorkspace } from "./components/ProjectWorkspace";
import { researchSourceFromDraft, type ResearchDraft } from "./components/ResearchWorkspace";
import { TopBar } from "./components/TopBar";
import type {
  AgentRunWithEvents,
  AgentRuntimeStatus,
  Asset,
  Brand,
  BriefUnderstandingResult,
  CreativeDirection,
  HealthResponse,
  MemoryEntry,
  ProductUnderstanding,
  Project,
  ProjectVersion,
  ResearchReport,
  ResearchReportResult,
} from "./types/joi";

type BrandDraft = {
  name: string;
  description: string;
};

type ProjectDraft = {
  title: string;
  advertising_goal: string;
  duration_seconds: string;
};

type MemoryDraft = {
  content: string;
  source: string;
};

type FormMode = "create" | "edit";

const emptyBrandDraft: BrandDraft = {
  name: "",
  description: "",
};

const emptyProjectDraft: ProjectDraft = {
  title: "",
  advertising_goal: "",
  duration_seconds: "15",
};

const emptyMemoryDraft: MemoryDraft = {
  content: "",
  source: "user note",
};

const emptyBriefDraft: BriefDraft = {
  brief_text: "",
  product_name: "",
  category: "",
  audience: "",
  target_platforms_text: "",
  selling_points_text: "",
  visual_direction: "",
  constraints_text: "",
};

const emptyReferenceAssetDraft: ReferenceAssetDraft = {
  kind: "link",
  display_name: "",
  source_uri: "",
};

const emptyResearchDraft: ResearchDraft = {
  research_goal: "",
  market_focus: "",
  platform_focus_text: "",
  source_title: "",
  source_url: "",
  source_type: "reference",
  source_excerpt: "",
};

const defaultAgentGoal = "Plan the next content workflow steps for this project";

export default function App() {
  const [activeTab, setActiveTab] = useState("Overview");
  const [activityLog, setActivityLog] = useState<string[]>([]);
  const [agentGoalDraft, setAgentGoalDraft] = useState(defaultAgentGoal);
  const [agentRuns, setAgentRuns] = useState<AgentRunWithEvents[]>([]);
  const [agentRuntimeStatus, setAgentRuntimeStatus] = useState<AgentRuntimeStatus | null>(null);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [brandDraft, setBrandDraft] = useState<BrandDraft>(emptyBrandDraft);
  const [brandMode, setBrandMode] = useState<FormMode>("edit");
  const [brands, setBrands] = useState<Brand[]>([]);
  const [briefDraft, setBriefDraft] = useState<BriefDraft>(emptyBriefDraft);
  const [creativeDirections, setCreativeDirections] = useState<CreativeDirection[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [generatingUnderstanding, setGeneratingUnderstanding] = useState(false);
  const [generatingResearch, setGeneratingResearch] = useState(false);
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [memoryDraft, setMemoryDraft] = useState<MemoryDraft>(emptyMemoryDraft);
  const [memoryEntries, setMemoryEntries] = useState<MemoryEntry[]>([]);
  const [productUnderstandings, setProductUnderstandings] = useState<ProductUnderstanding[]>([]);
  const [projectDraft, setProjectDraft] = useState<ProjectDraft>(emptyProjectDraft);
  const [projectMode, setProjectMode] = useState<FormMode>("edit");
  const [projects, setProjects] = useState<Project[]>([]);
  const [referenceAssetDraft, setReferenceAssetDraft] =
    useState<ReferenceAssetDraft>(emptyReferenceAssetDraft);
  const [researchDraft, setResearchDraft] = useState<ResearchDraft>(emptyResearchDraft);
  const [researchReports, setResearchReports] = useState<ResearchReport[]>([]);
  const [researchResult, setResearchResult] = useState<ResearchReportResult | null>(null);
  const [savingSnapshot, setSavingSnapshot] = useState(false);
  const [selectedBrandId, setSelectedBrandId] = useState<string | null>(null);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [startingAgentPlan, setStartingAgentPlan] = useState(false);
  const [understandingResult, setUnderstandingResult] = useState<BriefUnderstandingResult | null>(null);
  const [versions, setVersions] = useState<ProjectVersion[]>([]);

  const latestAgentRun = agentRuns[0] ?? null;

  const selectedBrand = useMemo(
    () => (brandMode === "edit" ? brands.find((brand) => brand.id === selectedBrandId) ?? null : null),
    [brandMode, brands, selectedBrandId],
  );
  const selectedProject = useMemo(
    () => (projectMode === "edit" ? projects.find((project) => project.id === selectedProjectId) ?? null : null),
    [projectMode, projects, selectedProjectId],
  );

  useEffect(() => {
    void loadInitialState();
  }, []);

  useEffect(() => {
    if (brandMode === "edit" && !selectedBrandId && brands.length > 0) {
      setSelectedBrandId(brands[0].id);
    }
  }, [brandMode, brands, selectedBrandId]);

  useEffect(() => {
    if (selectedBrand) {
      setBrandDraft({
        name: selectedBrand.name,
        description: selectedBrand.description,
      });
      void refreshProjects(selectedBrand.id);
    } else {
      setBrandDraft(emptyBrandDraft);
      setProjects([]);
      setSelectedProjectId(null);
    }
  }, [selectedBrand]);

  useEffect(() => {
    if (projectMode === "edit" && !selectedProjectId && projects.length > 0) {
      setSelectedProjectId(projects[0].id);
    }
    if (projectMode === "edit" && selectedProjectId && !projects.some((project) => project.id === selectedProjectId)) {
      setSelectedProjectId(projects[0]?.id ?? null);
    }
  }, [projectMode, projects, selectedProjectId]);

  useEffect(() => {
    if (selectedProject) {
      setProjectDraft({
        title: selectedProject.title,
        advertising_goal: selectedProject.advertising_goal,
        duration_seconds: String(selectedProject.duration_seconds),
      });
      setBriefDraft(emptyBriefDraft);
      setReferenceAssetDraft(emptyReferenceAssetDraft);
      setResearchDraft(emptyResearchDraft);
      setResearchResult(null);
      setUnderstandingResult(null);
      void refreshProjectState(selectedProject.id);
    } else {
      setProjectDraft(emptyProjectDraft);
      setBriefDraft(emptyBriefDraft);
      setReferenceAssetDraft(emptyReferenceAssetDraft);
      setResearchDraft(emptyResearchDraft);
      setResearchReports([]);
      setResearchResult(null);
      setUnderstandingResult(null);
      setAssets([]);
      setAgentRuns([]);
      setCreativeDirections([]);
      setMemoryEntries([]);
      setProductUnderstandings([]);
      setVersions([]);
    }
  }, [selectedProject]);

  async function loadInitialState() {
    try {
      setError(null);
      const [healthResult, runtimeStatus, brandList] = await Promise.all([
        healthCheck(),
        getAgentRuntimeStatus(),
        listBrands(),
      ]);
      setHealth(healthResult);
      setAgentRuntimeStatus(runtimeStatus);
      setBrands(brandList);
      setActivityLog((entries) => [...entries, "Workspace connected to Joi backend."]);
    } catch (loadError) {
      setError(formatError(loadError));
    }
  }

  async function refreshBrands(preferredBrandId?: string) {
    const brandList = await listBrands();
    setBrands(brandList);
    if (preferredBrandId) {
      setBrandMode("edit");
      setSelectedBrandId(preferredBrandId);
    }
  }

  async function refreshProjects(brandId: string, preferredProjectId?: string) {
    try {
      const projectList = await listProjects(brandId);
      setProjects(projectList);
      if (preferredProjectId) {
        setProjectMode("edit");
        setSelectedProjectId(preferredProjectId);
      }
    } catch (loadError) {
      setError(formatError(loadError));
    }
  }

  async function refreshProjectState(projectId: string) {
    try {
      const [
        assetList,
        versionList,
        projectMemory,
        understandingList,
        directionList,
        reportList,
        runList,
      ] = await Promise.all([
        listAssets(projectId),
        listProjectVersions(projectId),
        listMemoryEntries({ scope: "project", brand_id: null, project_id: projectId }),
        listProductUnderstandings(projectId),
        listCreativeDirections(projectId),
        listResearchReports(projectId),
        listAgentRuns(projectId),
      ]);
      setAssets(assetList);
      setVersions(versionList);
      setMemoryEntries(projectMemory);
      setProductUnderstandings(understandingList);
      setCreativeDirections(directionList);
      setResearchReports(reportList);
      setAgentRuns(runList);
    } catch (loadError) {
      setError(formatError(loadError));
    }
  }

  function startNewBrand() {
    setBrandMode("create");
    setProjectMode("create");
    setSelectedBrandId(null);
    setSelectedProjectId(null);
    setBrandDraft(emptyBrandDraft);
    setProjectDraft(emptyProjectDraft);
    setProjects([]);
    setAssets([]);
    setBriefDraft(emptyBriefDraft);
    setCreativeDirections([]);
    setAgentRuns([]);
    setMemoryEntries([]);
    setProductUnderstandings([]);
    setReferenceAssetDraft(emptyReferenceAssetDraft);
    setResearchDraft(emptyResearchDraft);
    setResearchReports([]);
    setResearchResult(null);
    setUnderstandingResult(null);
    setVersions([]);
    setActiveTab("Overview");
    setActivityLog((entries) => [...entries, "Started a new brand draft."]);
  }

  function startNewProject() {
    setProjectMode("create");
    setSelectedProjectId(null);
    setProjectDraft(emptyProjectDraft);
    setAssets([]);
    setBriefDraft(emptyBriefDraft);
    setCreativeDirections([]);
    setAgentRuns([]);
    setMemoryEntries([]);
    setProductUnderstandings([]);
    setReferenceAssetDraft(emptyReferenceAssetDraft);
    setResearchDraft(emptyResearchDraft);
    setResearchReports([]);
    setResearchResult(null);
    setUnderstandingResult(null);
    setVersions([]);
    setActiveTab("Overview");
    setActivityLog((entries) => [...entries, "Started a new project draft."]);
  }

  function selectBrand(brandId: string) {
    setBrandMode("edit");
    setProjectMode("edit");
    setSelectedBrandId(brandId);
    setSelectedProjectId(null);
    setActiveTab("Overview");
  }

  function selectProject(projectId: string) {
    setProjectMode("edit");
    setSelectedProjectId(projectId);
    setActiveTab("Overview");
  }

  async function submitBrand() {
    if (!brandDraft.name.trim()) {
      setError("Brand name is required.");
      return;
    }

    try {
      setError(null);
      const isEditingBrand = brandMode === "edit" && selectedBrand;
      const brand = isEditingBrand
        ? await updateBrand({
            id: selectedBrand.id,
            name: brandDraft.name,
            description: brandDraft.description,
          })
        : await createBrand(brandDraft);
      await refreshBrands(brand.id);
      setActivityLog((entries) => [...entries, `${isEditingBrand ? "Updated" : "Created"} brand ${brand.name}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    }
  }

  async function submitProject() {
    if (!selectedBrand) {
      setError("Select or create a brand first.");
      return;
    }
    if (!projectDraft.title.trim()) {
      setError("Project title is required.");
      return;
    }
    const duration = Number(projectDraft.duration_seconds);
    if (!Number.isFinite(duration) || duration <= 0) {
      setError("Project duration must be a positive number.");
      return;
    }

    try {
      setError(null);
      const isEditingProject = projectMode === "edit" && selectedProject;
      const project = isEditingProject
        ? await updateProject({
            id: selectedProject.id,
            title: projectDraft.title,
            advertising_goal: projectDraft.advertising_goal,
            duration_seconds: duration,
          })
        : await createProject({
            brand_id: selectedBrand.id,
            title: projectDraft.title,
            advertising_goal: projectDraft.advertising_goal,
            duration_seconds: duration,
          });
      await refreshProjects(selectedBrand.id, project.id);
      setActivityLog((entries) => [
        ...entries,
        `${isEditingProject ? "Updated" : "Created"} project ${project.title}.`,
      ]);
    } catch (submitError) {
      setError(formatError(submitError));
    }
  }

  async function submitMemory() {
    if (!selectedBrand || !selectedProject) {
      setError("Select a project before adding memory.");
      return;
    }
    if (!memoryDraft.content.trim()) {
      setError("Memory content is required.");
      return;
    }

    try {
      setError(null);
      const memory = await createMemoryEntry({
        scope: "project",
        brand_id: selectedBrand.id,
        project_id: selectedProject.id,
        content: memoryDraft.content,
        source: memoryDraft.source || "user note",
      });
      setMemoryDraft(emptyMemoryDraft);
      await refreshProjectState(selectedProject.id);
      setActivityLog((entries) => [...entries, `Added project memory ${memory.id}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    }
  }

  async function submitBriefUnderstanding() {
    if (!selectedProject) {
      setError("Select a project before generating understanding.");
      return;
    }

    try {
      setGeneratingUnderstanding(true);
      setError(null);
      const result = await generateBriefUnderstanding({
        project_id: selectedProject.id,
        brief_text: briefDraft.brief_text,
        product_name: briefDraft.product_name,
        category: briefDraft.category,
        audience: briefDraft.audience,
        target_platforms: splitListText(briefDraft.target_platforms_text),
        selling_points_text: briefDraft.selling_points_text,
        visual_direction: briefDraft.visual_direction,
        constraints_text: briefDraft.constraints_text,
        reference_asset_ids: assets.map((asset) => asset.id),
      });
      setUnderstandingResult(result);
      await refreshProjectState(selectedProject.id);
      setActivityLog((entries) => [
        ...entries,
        `Generated product understanding ${result.product_understanding.id}.`,
      ]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setGeneratingUnderstanding(false);
    }
  }

  async function submitReferenceAsset() {
    if (!selectedProject) {
      setError("Select a project before adding reference material.");
      return;
    }
    if (!referenceAssetDraft.display_name.trim()) {
      setError("Reference name is required.");
      return;
    }
    if (!referenceAssetDraft.source_uri.trim()) {
      setError("Reference URL is required.");
      return;
    }

    try {
      setError(null);
      const asset = await createReferenceAsset({
        project_id: selectedProject.id,
        kind: referenceAssetDraft.kind || "link",
        display_name: referenceAssetDraft.display_name,
        source_uri: referenceAssetDraft.source_uri,
      });
      setReferenceAssetDraft(emptyReferenceAssetDraft);
      await refreshProjectState(selectedProject.id);
      setActivityLog((entries) => [...entries, `Added reference material ${asset.display_name}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    }
  }

  async function submitResearchReport() {
    if (!selectedProject) {
      setError("Select a project before generating research.");
      return;
    }
    if (!researchDraft.research_goal.trim()) {
      setError("Research goal is required.");
      return;
    }
    if (!researchDraft.source_title.trim() || !researchDraft.source_excerpt.trim()) {
      setError("Source title and excerpt are required.");
      return;
    }

    try {
      setGeneratingResearch(true);
      setError(null);
      const result = await generateResearchReport({
        project_id: selectedProject.id,
        research_goal: researchDraft.research_goal,
        market_focus: researchDraft.market_focus,
        platform_focus: splitListText(researchDraft.platform_focus_text),
        source_materials: [researchSourceFromDraft(researchDraft)],
      });
      setResearchResult(result);
      await refreshProjectState(selectedProject.id);
      setAgentRuns((runs) => [
        { run: result.agent_run, events: result.agent_events },
        ...runs.filter((item) => item.run.id !== result.agent_run.id),
      ]);
      setActivityLog((entries) => [...entries, `Generated research report ${result.report.id}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setGeneratingResearch(false);
    }
  }

  async function handleSaveSnapshot() {
    if (!selectedProject) {
      setError("Select a project before saving a snapshot.");
      return;
    }

    try {
      setSavingSnapshot(true);
      setError(null);
      const version = await saveProjectSnapshot({
        project_id: selectedProject.id,
        label: `Workspace snapshot`,
        change_reason: "Saved from 0.11 workspace UI",
      });
      await refreshProjectState(selectedProject.id);
      setActivityLog((entries) => [...entries, `Saved snapshot version ${version.version_number}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setSavingSnapshot(false);
    }
  }

  async function submitAgentPlan() {
    if (!selectedProject) {
      setError("Select a project before starting an agent plan.");
      return;
    }
    if (!agentGoalDraft.trim()) {
      setError("Agent goal is required.");
      return;
    }

    try {
      setStartingAgentPlan(true);
      setError(null);
      const result = await startAgentPlan({
        project_id: selectedProject.id,
        user_goal: agentGoalDraft,
      });
      const runWithEvents = { run: result.run, events: result.events };
      setAgentRuns((runs) => [runWithEvents, ...runs.filter((item) => item.run.id !== result.run.id)]);
      setActivityLog((entries) => [...entries, `Started agent plan ${result.run.id}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setStartingAgentPlan(false);
    }
  }

  return (
    <div className="app-shell">
      <TopBar
        health={health}
        onSaveSnapshot={handleSaveSnapshot}
        savingSnapshot={savingSnapshot}
        selectedBrand={selectedBrand}
        selectedProject={selectedProject}
      />

      <div className="workspace-shell">
        <BrandProjectRail
          activeTab={activeTab}
          brands={brands}
          onNewBrand={startNewBrand}
          onNewProject={startNewProject}
          onSelectBrand={selectBrand}
          onSelectProject={selectProject}
          onSelectTab={setActiveTab}
          projects={projects}
          selectedBrandId={selectedBrandId}
          selectedProjectId={selectedProjectId}
        />

        <ProjectWorkspace
          activeTab={activeTab}
          assets={assets}
          brandDraft={brandDraft}
          briefDraft={briefDraft}
          creativeDirections={creativeDirections}
          generatingUnderstanding={generatingUnderstanding}
          generatingResearch={generatingResearch}
          onBriefDraftChange={(field, value) => setBriefDraft((draft) => ({ ...draft, [field]: value }))}
          memoryDraft={memoryDraft}
          memoryEntries={memoryEntries}
          onBrandDraftChange={(field, value) => setBrandDraft((draft) => ({ ...draft, [field]: value }))}
          onMemoryDraftChange={(field, value) => setMemoryDraft((draft) => ({ ...draft, [field]: value }))}
          onProjectDraftChange={(field, value) =>
            setProjectDraft((draft) => ({
              ...draft,
              [field]: value,
            }))
          }
          onReferenceAssetDraftChange={(field, value) =>
            setReferenceAssetDraft((draft) => ({ ...draft, [field]: value }))
          }
          onResearchDraftChange={(field, value) => setResearchDraft((draft) => ({ ...draft, [field]: value }))}
          onSubmitBrand={submitBrand}
          onSubmitBriefUnderstanding={submitBriefUnderstanding}
          onSubmitMemory={submitMemory}
          onSubmitProject={submitProject}
          onSubmitReferenceAsset={submitReferenceAsset}
          onSubmitResearchReport={submitResearchReport}
          productUnderstandings={productUnderstandings}
          projectDraft={projectDraft}
          referenceAssetDraft={referenceAssetDraft}
          researchDraft={researchDraft}
          researchReports={researchReports}
          researchResult={researchResult}
          selectedBrand={selectedBrand}
          selectedProject={selectedProject}
          understandingResult={understandingResult}
          versions={versions}
        />

        <AgentPanel
          activityLog={activityLog}
          agentGoalDraft={agentGoalDraft}
          agentRuntimeStatus={agentRuntimeStatus}
          agentRuns={agentRuns}
          latestAgentRun={latestAgentRun}
          onAgentGoalChange={setAgentGoalDraft}
          onStartAgentPlan={submitAgentPlan}
          selectedBrand={selectedBrand}
          selectedProject={selectedProject}
          startingAgentPlan={startingAgentPlan}
        />
      </div>

      {error ? (
        <div className="toast" role="alert">
          {error}
        </div>
      ) : null}
    </div>
  );
}

function splitListText(value: string): string[] {
  return value
    .split(/[\n,，;；]/)
    .map((item) => item.trim())
    .filter(Boolean);
}
