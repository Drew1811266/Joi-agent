import { useEffect, useMemo, useState } from "react";

import {
  applyQualityReviewSuggestion,
  createBrand,
  createMemoryEntry,
  createProject,
  createReferenceAsset,
  exportProject,
  formatError,
  generateBriefUnderstanding,
  generateDeliveryReport,
  generateMemoryCandidates,
  generatePromptPackages,
  generateQualityReview,
  generateResearchReport,
  generateStoryboard,
  getAgentRuntimeStatus,
  getPromptAdapterProfiles,
  healthCheck,
  listAgentRuns,
  listAssets,
  listBrands,
  listCreativeDirections,
  listDeliveryReports,
  listMemoryEntries,
  listPromptPackages,
  listProductUnderstandings,
  listProjectVersions,
  listProjects,
  listQualityReviews,
  listResearchReports,
  listStoryboards,
  previewDeliveryPackage,
  regenerateShot,
  saveProjectSnapshot,
  startAgentPlan,
  updateBrand,
  updateDeliveryReport,
  updateMemoryStatus,
  updatePromptPackage,
  updateProject,
  updateShot,
} from "./api/joiApi";
import { AgentPanel } from "./components/AgentPanel";
import { BrandProjectRail } from "./components/BrandProjectRail";
import type { BriefDraft, ReferenceAssetDraft } from "./components/BriefWorkspace";
import type { MemoryCurationDraft } from "./components/MemoryWorkspace";
import type { DeliveryDraft } from "./components/DeliveryWorkspace";
import type { PromptDraft } from "./components/PromptWorkspace";
import { ProjectWorkspace } from "./components/ProjectWorkspace";
import { researchSourceFromDraft, type ResearchDraft } from "./components/ResearchWorkspace";
import type { ReviewDraft } from "./components/ReviewWorkspace";
import type { StoryboardDraft } from "./components/StoryboardWorkspace";
import { TopBar } from "./components/TopBar";
import type {
  AgentRunWithEvents,
  AgentRuntimeStatus,
  Asset,
  Brand,
  BriefUnderstandingResult,
  CreativeDirection,
  DeliveryPackagePreview,
  DeliveryReport,
  DeliveryReportUpdateInput,
  HealthResponse,
  MemoryCurationResult,
  MemoryEntry,
  ProductUnderstanding,
  PromptAdapterProfile,
  PromptPackageUpdateInput,
  PromptPackageView,
  Project,
  ProjectVersion,
  QualityReview,
  QualityReviewCheck,
  QualityReviewSuggestion,
  ResearchReport,
  ResearchReportResult,
  ShotUpdateInput,
  StoryboardGenerationResult,
  StoryboardWithShots,
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

const emptyMemoryCurationDraft: MemoryCurationDraft = {
  feedback_text: "",
  include_research_reports: true,
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

const emptyStoryboardDraft: StoryboardDraft = {
  user_direction: "",
  preferred_duration_seconds: "15",
  preferred_shot_count: "5",
  regeneration_note: "",
};

const emptyPromptDraft: PromptDraft = {
  selected_video_platforms: ["jimeng_video", "grok_video"],
  selected_image_platforms: ["banana_2_image", "jimeng_image", "gpt_image_2"],
  selected_shot_ids: [],
  image_brief: "",
  user_direction: "",
};

const emptyDeliveryDraft: DeliveryDraft = {
  user_direction: "",
  export_dir: "",
};

const emptyReviewDraft: ReviewDraft = {
  user_direction: "",
};

const defaultAgentGoal = "Plan the next content workflow steps for this project";

export default function App() {
  const [activeTab, setActiveTab] = useState("Overview");
  const [activityLog, setActivityLog] = useState<string[]>([]);
  const [agentGoalDraft, setAgentGoalDraft] = useState(defaultAgentGoal);
  const [agentRuns, setAgentRuns] = useState<AgentRunWithEvents[]>([]);
  const [agentRuntimeStatus, setAgentRuntimeStatus] = useState<AgentRuntimeStatus | null>(null);
  const [adapterProfiles, setAdapterProfiles] = useState<PromptAdapterProfile[]>([]);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [brandDraft, setBrandDraft] = useState<BrandDraft>(emptyBrandDraft);
  const [brandMode, setBrandMode] = useState<FormMode>("edit");
  const [brands, setBrands] = useState<Brand[]>([]);
  const [briefDraft, setBriefDraft] = useState<BriefDraft>(emptyBriefDraft);
  const [creativeDirections, setCreativeDirections] = useState<CreativeDirection[]>([]);
  const [curatingMemory, setCuratingMemory] = useState(false);
  const [deliveryDraft, setDeliveryDraft] = useState<DeliveryDraft>(emptyDeliveryDraft);
  const [deliveryReports, setDeliveryReports] = useState<DeliveryReport[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [applyingSuggestionId, setApplyingSuggestionId] = useState<string | null>(null);
  const [exportingDeliveryPackage, setExportingDeliveryPackage] = useState(false);
  const [generatingDeliveryReport, setGeneratingDeliveryReport] = useState(false);
  const [generatingQualityReview, setGeneratingQualityReview] = useState(false);
  const [generatingStoryboard, setGeneratingStoryboard] = useState(false);
  const [generatingPrompts, setGeneratingPrompts] = useState(false);
  const [generatingUnderstanding, setGeneratingUnderstanding] = useState(false);
  const [generatingResearch, setGeneratingResearch] = useState(false);
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [memoryCurationDraft, setMemoryCurationDraft] =
    useState<MemoryCurationDraft>(emptyMemoryCurationDraft);
  const [memoryCurationResult, setMemoryCurationResult] = useState<MemoryCurationResult | null>(null);
  const [memoryDraft, setMemoryDraft] = useState<MemoryDraft>(emptyMemoryDraft);
  const [memoryEntries, setMemoryEntries] = useState<MemoryEntry[]>([]);
  const [packagePreview, setPackagePreview] = useState<DeliveryPackagePreview | null>(null);
  const [previewingDeliveryPackage, setPreviewingDeliveryPackage] = useState(false);
  const [productUnderstandings, setProductUnderstandings] = useState<ProductUnderstanding[]>([]);
  const [promptDraft, setPromptDraft] = useState<PromptDraft>(emptyPromptDraft);
  const [promptPackages, setPromptPackages] = useState<PromptPackageView[]>([]);
  const [projectDraft, setProjectDraft] = useState<ProjectDraft>(emptyProjectDraft);
  const [projectMode, setProjectMode] = useState<FormMode>("edit");
  const [projects, setProjects] = useState<Project[]>([]);
  const [referenceAssetDraft, setReferenceAssetDraft] =
    useState<ReferenceAssetDraft>(emptyReferenceAssetDraft);
  const [researchDraft, setResearchDraft] = useState<ResearchDraft>(emptyResearchDraft);
  const [researchReports, setResearchReports] = useState<ResearchReport[]>([]);
  const [researchResult, setResearchResult] = useState<ResearchReportResult | null>(null);
  const [latestReviewChecks, setLatestReviewChecks] = useState<QualityReviewCheck[]>([]);
  const [latestReviewSuggestions, setLatestReviewSuggestions] = useState<QualityReviewSuggestion[]>([]);
  const [qualityReviews, setQualityReviews] = useState<QualityReview[]>([]);
  const [reviewDraft, setReviewDraft] = useState<ReviewDraft>(emptyReviewDraft);
  const [regeneratingShotId, setRegeneratingShotId] = useState<string | null>(null);
  const [savingDeliveryReportId, setSavingDeliveryReportId] = useState<string | null>(null);
  const [savingPromptId, setSavingPromptId] = useState<string | null>(null);
  const [savingSnapshot, setSavingSnapshot] = useState(false);
  const [savingShotId, setSavingShotId] = useState<string | null>(null);
  const [selectedBrandId, setSelectedBrandId] = useState<string | null>(null);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [startingAgentPlan, setStartingAgentPlan] = useState(false);
  const [storyboardDraft, setStoryboardDraft] = useState<StoryboardDraft>(emptyStoryboardDraft);
  const [storyboardResult, setStoryboardResult] = useState<StoryboardGenerationResult | null>(null);
  const [storyboards, setStoryboards] = useState<StoryboardWithShots[]>([]);
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
      setDeliveryDraft(emptyDeliveryDraft);
      setDeliveryReports([]);
      setMemoryCurationDraft(emptyMemoryCurationDraft);
      setMemoryCurationResult(null);
      setPackagePreview(null);
      setPromptDraft(emptyPromptDraft);
      setPromptPackages([]);
      setReferenceAssetDraft(emptyReferenceAssetDraft);
      setResearchDraft(emptyResearchDraft);
      setResearchResult(null);
      setLatestReviewChecks([]);
      setLatestReviewSuggestions([]);
      setQualityReviews([]);
      setReviewDraft(emptyReviewDraft);
      setStoryboardDraft({
        ...emptyStoryboardDraft,
        preferred_duration_seconds: String(selectedProject.duration_seconds),
      });
      setStoryboardResult(null);
      setUnderstandingResult(null);
      void refreshProjectState(selectedProject.id);
    } else {
      setProjectDraft(emptyProjectDraft);
      setBriefDraft(emptyBriefDraft);
      setDeliveryDraft(emptyDeliveryDraft);
      setDeliveryReports([]);
      setMemoryCurationDraft(emptyMemoryCurationDraft);
      setMemoryCurationResult(null);
      setPackagePreview(null);
      setPromptDraft(emptyPromptDraft);
      setPromptPackages([]);
      setReferenceAssetDraft(emptyReferenceAssetDraft);
      setResearchDraft(emptyResearchDraft);
      setResearchReports([]);
      setResearchResult(null);
      setLatestReviewChecks([]);
      setLatestReviewSuggestions([]);
      setQualityReviews([]);
      setReviewDraft(emptyReviewDraft);
      setStoryboardDraft(emptyStoryboardDraft);
      setStoryboardResult(null);
      setStoryboards([]);
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
      const [healthResult, runtimeStatus, adapterProfileList, brandList] = await Promise.all([
        healthCheck(),
        getAgentRuntimeStatus(),
        getPromptAdapterProfiles(),
        listBrands(),
      ]);
      setHealth(healthResult);
      setAgentRuntimeStatus(runtimeStatus);
      setAdapterProfiles(adapterProfileList);
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
        storyboardList,
        promptPackageList,
        qualityReviewList,
        deliveryReportList,
        runList,
      ] = await Promise.all([
        listAssets(projectId),
        listProjectVersions(projectId),
        listMemoryEntries({ scope: "project", brand_id: null, project_id: projectId }),
        listProductUnderstandings(projectId),
        listCreativeDirections(projectId),
        listResearchReports(projectId),
        listStoryboards(projectId),
        listPromptPackages(projectId),
        listQualityReviews(projectId),
        listDeliveryReports(projectId),
        listAgentRuns(projectId),
      ]);
      setAssets(assetList);
      setVersions(versionList);
      setMemoryEntries(projectMemory);
      setProductUnderstandings(understandingList);
      setCreativeDirections(directionList);
      setResearchReports(reportList);
      setStoryboards(storyboardList);
      setPromptPackages(promptPackageList);
      setQualityReviews(qualityReviewList);
      setDeliveryReports(deliveryReportList);
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
    setDeliveryDraft(emptyDeliveryDraft);
    setDeliveryReports([]);
    setAgentRuns([]);
    setMemoryCurationDraft(emptyMemoryCurationDraft);
    setMemoryCurationResult(null);
    setMemoryEntries([]);
    setPackagePreview(null);
    setProductUnderstandings([]);
    setPromptDraft(emptyPromptDraft);
    setPromptPackages([]);
    setReferenceAssetDraft(emptyReferenceAssetDraft);
    setResearchDraft(emptyResearchDraft);
    setResearchReports([]);
    setResearchResult(null);
    setLatestReviewChecks([]);
    setLatestReviewSuggestions([]);
    setQualityReviews([]);
    setReviewDraft(emptyReviewDraft);
    setStoryboardDraft(emptyStoryboardDraft);
    setStoryboardResult(null);
    setStoryboards([]);
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
    setDeliveryDraft(emptyDeliveryDraft);
    setDeliveryReports([]);
    setAgentRuns([]);
    setMemoryCurationDraft(emptyMemoryCurationDraft);
    setMemoryCurationResult(null);
    setMemoryEntries([]);
    setPackagePreview(null);
    setProductUnderstandings([]);
    setPromptDraft(emptyPromptDraft);
    setPromptPackages([]);
    setReferenceAssetDraft(emptyReferenceAssetDraft);
    setResearchDraft(emptyResearchDraft);
    setResearchReports([]);
    setResearchResult(null);
    setLatestReviewChecks([]);
    setLatestReviewSuggestions([]);
    setQualityReviews([]);
    setReviewDraft(emptyReviewDraft);
    setStoryboardDraft(emptyStoryboardDraft);
    setStoryboardResult(null);
    setStoryboards([]);
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

  async function submitMemoryCandidates() {
    if (!selectedProject) {
      setError("Select a project before generating memory candidates.");
      return;
    }
    if (!memoryCurationDraft.include_research_reports && !memoryCurationDraft.feedback_text.trim()) {
      setError("Memory candidate generation needs feedback or research reports.");
      return;
    }

    try {
      setCuratingMemory(true);
      setError(null);
      const result = await generateMemoryCandidates({
        project_id: selectedProject.id,
        feedback_text: memoryCurationDraft.feedback_text,
        include_research_reports: memoryCurationDraft.include_research_reports,
      });
      setMemoryCurationResult(result);
      await refreshProjectState(selectedProject.id);
      setAgentRuns((runs) => [
        { run: result.agent_run, events: result.agent_events },
        ...runs.filter((item) => item.run.id !== result.agent_run.id),
      ]);
      setActivityLog((entries) => [
        ...entries,
        `Generated ${result.candidates.length} memory candidate(s).`,
      ]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setCuratingMemory(false);
    }
  }

  async function handleUpdateMemoryStatus(id: string, status: "accepted" | "rejected") {
    try {
      setError(null);
      const memory = await updateMemoryStatus({ id, status });
      setMemoryCurationResult((result) =>
        result
          ? {
              ...result,
              candidates: result.candidates.map((candidate) =>
                candidate.entry.id === memory.id ? { ...candidate, entry: memory } : candidate,
              ),
            }
          : result,
      );
      if (selectedProject) {
        await refreshProjectState(selectedProject.id);
      }
      setActivityLog((entries) => [...entries, `Updated memory ${memory.id} to ${memory.status}.`]);
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

  async function submitStoryboardGeneration() {
    if (!selectedProject) {
      setError("Select a project before generating a storyboard.");
      return;
    }

    const duration = Number(storyboardDraft.preferred_duration_seconds);
    const shotCount = Number(storyboardDraft.preferred_shot_count);
    const preferredDuration =
      Number.isFinite(duration) && duration > 0 ? duration : selectedProject.duration_seconds;
    const preferredShotCount = Number.isFinite(shotCount) && shotCount > 0 ? shotCount : null;

    try {
      setGeneratingStoryboard(true);
      setError(null);
      const result = await generateStoryboard({
        project_id: selectedProject.id,
        user_direction: storyboardDraft.user_direction,
        preferred_duration_seconds: preferredDuration,
        preferred_shot_count: preferredShotCount,
      });
      setStoryboardResult(result);
      await refreshProjectState(selectedProject.id);
      setAgentRuns((runs) => [
        { run: result.agent_run, events: result.agent_events },
        ...runs.filter((item) => item.run.id !== result.agent_run.id),
      ]);
      setActivityLog((entries) => [
        ...entries,
        `Generated storyboard ${result.storyboard.id}.`,
      ]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setGeneratingStoryboard(false);
    }
  }

  async function handleUpdateShot(input: ShotUpdateInput) {
    try {
      setSavingShotId(input.id);
      setError(null);
      const updated = await updateShot(input);
      setStoryboardResult((result) =>
        result
          ? {
              ...result,
              shots: result.shots.map((shot) =>
                shot.shot.id === updated.shot.id ? updated : shot,
              ),
            }
          : result,
      );
      if (selectedProject) {
        await refreshProjectState(selectedProject.id);
      }
      setActivityLog((entries) => [...entries, `Updated shot ${updated.shot.shot_number}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setSavingShotId(null);
    }
  }

  async function handleRegenerateShot(storyboardId: string, shotId: string) {
    if (!selectedProject) {
      setError("Select a project before regenerating a shot.");
      return;
    }

    try {
      setRegeneratingShotId(shotId);
      setError(null);
      const result = await regenerateShot({
        project_id: selectedProject.id,
        storyboard_id: storyboardId,
        shot_id: shotId,
        revision_note: storyboardDraft.regeneration_note,
      });
      setStoryboardResult((current) =>
        current
          ? {
              ...current,
              shots: current.shots.map((shot) =>
                shot.shot.id === result.shot.shot.id ? result.shot : shot,
              ),
            }
          : current,
      );
      await refreshProjectState(selectedProject.id);
      setAgentRuns((runs) => [
        { run: result.agent_run, events: result.agent_events },
        ...runs.filter((item) => item.run.id !== result.agent_run.id),
      ]);
      setActivityLog((entries) => [
        ...entries,
        `Regenerated shot ${result.shot.shot.shot_number}.`,
      ]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setRegeneratingShotId(null);
    }
  }

  function handlePromptDraftChange(field: keyof PromptDraft, value: string | string[]) {
    setPromptDraft((draft) => ({ ...draft, [field]: value }));
  }

  async function submitVideoPrompts() {
    if (!selectedProject) {
      setError("Select a project before generating prompt packages.");
      return;
    }
    if (promptDraft.selected_shot_ids.length === 0) {
      setError("Select at least one storyboard shot.");
      return;
    }
    if (promptDraft.selected_video_platforms.length === 0) {
      setError("Select at least one video adapter.");
      return;
    }
    await submitPromptGeneration(promptDraft.selected_video_platforms, promptDraft.selected_shot_ids, "");
  }

  async function submitImagePrompts() {
    if (!selectedProject) {
      setError("Select a project before generating prompt packages.");
      return;
    }
    if (!promptDraft.image_brief.trim()) {
      setError("Image brief is required.");
      return;
    }
    if (promptDraft.selected_image_platforms.length === 0) {
      setError("Select at least one image adapter.");
      return;
    }
    await submitPromptGeneration(promptDraft.selected_image_platforms, [], promptDraft.image_brief);
  }

  async function submitPromptGeneration(
    targetPlatforms: string[],
    shotIds: string[],
    imageBrief: string,
  ) {
    if (!selectedProject) {
      return;
    }

    try {
      setGeneratingPrompts(true);
      setError(null);
      const result = await generatePromptPackages({
        project_id: selectedProject.id,
        shot_ids: shotIds,
        image_brief: imageBrief,
        target_platforms: targetPlatforms,
        user_direction: promptDraft.user_direction,
      });
      await refreshProjectState(selectedProject.id);
      setPromptPackages(result.packages);
      setAgentRuns((runs) => [
        { run: result.agent_run, events: result.agent_events },
        ...runs.filter((item) => item.run.id !== result.agent_run.id),
      ]);
      setActivityLog((entries) => [
        ...entries,
        `Generated ${result.packages.length} prompt package(s).`,
      ]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setGeneratingPrompts(false);
    }
  }

  async function handleUpdatePromptPackage(input: PromptPackageUpdateInput) {
    try {
      setSavingPromptId(input.id);
      setError(null);
      const updated = await updatePromptPackage(input);
      if (selectedProject) {
        await refreshProjectState(selectedProject.id);
      }
      setPromptPackages((packages) => {
        let replaced = false;
        const next = packages.map((item) => {
          if (item.package.id !== updated.package.id) {
            return item;
          }
          replaced = true;
          return updated;
        });
        return replaced ? next : [updated, ...next];
      });
      setActivityLog((entries) => [...entries, `Updated prompt package ${updated.package.id}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setSavingPromptId(null);
    }
  }

  async function handleCopyPrompt(copyText: string, packageId: string) {
    if (!navigator.clipboard?.writeText) {
      setError("Clipboard is unavailable.");
      return;
    }

    try {
      setError(null);
      await navigator.clipboard.writeText(copyText);
      setActivityLog((entries) => [...entries, `Copied prompt package ${packageId}.`]);
    } catch (copyError) {
      setError(formatError(copyError));
    }
  }

  function handleReviewDraftChange(field: keyof ReviewDraft, value: string) {
    setReviewDraft((draft) => ({ ...draft, [field]: value }));
  }

  async function submitQualityReview() {
    if (!selectedProject) {
      setError("Select a project before generating a quality review.");
      return;
    }

    try {
      setGeneratingQualityReview(true);
      setError(null);
      const result = await generateQualityReview({
        project_id: selectedProject.id,
        user_direction: reviewDraft.user_direction,
      });
      await refreshProjectState(selectedProject.id);
      setLatestReviewChecks(result.checks);
      setLatestReviewSuggestions(result.suggestions);
      setQualityReviews((reviews) => replaceOrAppendReview(reviews, result.review));
      setAgentRuns((runs) => [
        { run: result.agent_run, events: result.agent_events },
        ...runs.filter((item) => item.run.id !== result.agent_run.id),
      ]);
      setActivityLog((entries) => [...entries, `Generated quality review ${result.review.score}/100.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setGeneratingQualityReview(false);
    }
  }

  async function applyReviewSuggestion(reviewId: string, suggestionId: string) {
    if (!selectedProject) {
      setError("Select a project before applying a review suggestion.");
      return;
    }

    try {
      setApplyingSuggestionId(suggestionId);
      setError(null);
      const result = await applyQualityReviewSuggestion({
        review_id: reviewId,
        suggestion_id: suggestionId,
      });
      await refreshProjectState(selectedProject.id);
      setQualityReviews((reviews) => replaceOrAppendReview(reviews, result.updated_review));
      setLatestReviewSuggestions((suggestions) => {
        const source = suggestions.length > 0 ? suggestions : normalizeReviewSuggestions(result.updated_review);
        return source.map((suggestion) =>
          suggestion.id === suggestionId ? result.suggestion : suggestion,
        );
      });
      setAgentRuns((runs) => [
        { run: result.agent_run, events: result.agent_events },
        ...runs.filter((item) => item.run.id !== result.agent_run.id),
      ]);
      setActivityLog((entries) => [
        ...entries,
        `Applied review suggestion to ${result.applied_target_type}.`,
      ]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setApplyingSuggestionId(null);
    }
  }

  async function submitDeliveryReport() {
    if (!selectedProject) {
      setError("Select a project before generating a delivery report.");
      return;
    }

    try {
      setGeneratingDeliveryReport(true);
      setError(null);
      const result = await generateDeliveryReport({
        project_id: selectedProject.id,
        user_direction: deliveryDraft.user_direction,
      });
      await refreshProjectState(selectedProject.id);
      setDeliveryReports((reports) => [
        result.report,
        ...reports.filter((report) => report.id !== result.report.id),
      ]);
      setPackagePreview(result.package_preview);
      setAgentRuns((runs) => [
        { run: result.agent_run, events: result.agent_events },
        ...runs.filter((item) => item.run.id !== result.agent_run.id),
      ]);
      setActivityLog((entries) => [...entries, `Generated delivery report ${result.report.id}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setGeneratingDeliveryReport(false);
    }
  }

  async function handleUpdateDeliveryReport(input: DeliveryReportUpdateInput) {
    try {
      setSavingDeliveryReportId(input.id);
      setError(null);
      const updated = await updateDeliveryReport(input);
      if (selectedProject) {
        await refreshProjectState(selectedProject.id);
      }
      setDeliveryReports((reports) => {
        let replaced = false;
        const next = reports.map((report) => {
          if (report.id !== updated.id) {
            return report;
          }
          replaced = true;
          return updated;
        });
        return replaced ? next : [updated, ...next];
      });
      setActivityLog((entries) => [...entries, `Updated delivery report ${updated.id}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setSavingDeliveryReportId(null);
    }
  }

  async function handlePreviewDeliveryPackage(reportId: string) {
    if (!selectedProject) {
      setError("Select a project before previewing a delivery package.");
      return;
    }

    try {
      setPreviewingDeliveryPackage(true);
      setError(null);
      const preview = await previewDeliveryPackage({
        project_id: selectedProject.id,
        delivery_report_id: reportId,
      });
      setPackagePreview(preview);
      setActivityLog((entries) => [...entries, `Previewed delivery package for ${selectedProject.title}.`]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setPreviewingDeliveryPackage(false);
    }
  }

  async function handleExportDeliveryPackage(reportId: string) {
    if (!selectedProject) {
      setError("Select a project before exporting a delivery package.");
      return;
    }
    if (!deliveryDraft.export_dir.trim()) {
      setError("Export directory is required.");
      return;
    }

    try {
      setExportingDeliveryPackage(true);
      setError(null);
      const result = await exportProject({
        project_id: selectedProject.id,
        export_dir: deliveryDraft.export_dir,
        delivery_report_id: reportId,
      });
      setActivityLog((entries) => [
        ...entries,
        `Exported delivery package to ${result.project_json_path}.`,
      ]);
    } catch (submitError) {
      setError(formatError(submitError));
    } finally {
      setExportingDeliveryPackage(false);
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
          adapterProfiles={adapterProfiles}
          applyingSuggestionId={applyingSuggestionId}
          assets={assets}
          brandDraft={brandDraft}
          briefDraft={briefDraft}
          creativeDirections={creativeDirections}
          curatingMemory={curatingMemory}
          deliveryDraft={deliveryDraft}
          deliveryReports={deliveryReports}
          exportingDeliveryPackage={exportingDeliveryPackage}
          generatingPrompts={generatingPrompts}
          generatingQualityReview={generatingQualityReview}
          generatingDeliveryReport={generatingDeliveryReport}
          generatingUnderstanding={generatingUnderstanding}
          generatingResearch={generatingResearch}
          memoryCurationDraft={memoryCurationDraft}
          memoryCurationResult={memoryCurationResult}
          onBriefDraftChange={(field, value) => setBriefDraft((draft) => ({ ...draft, [field]: value }))}
          onApplyReviewSuggestion={applyReviewSuggestion}
          onDeliveryDraftChange={(field, value) => setDeliveryDraft((draft) => ({ ...draft, [field]: value }))}
          memoryDraft={memoryDraft}
          memoryEntries={memoryEntries}
          onBrandDraftChange={(field, value) => setBrandDraft((draft) => ({ ...draft, [field]: value }))}
          onCopyPrompt={handleCopyPrompt}
          onMemoryCurationDraftChange={(field, value) =>
            setMemoryCurationDraft((draft) => ({ ...draft, [field]: value }))
          }
          onMemoryDraftChange={(field, value) => setMemoryDraft((draft) => ({ ...draft, [field]: value }))}
          onPromptDraftChange={handlePromptDraftChange}
          onProjectDraftChange={(field, value) =>
            setProjectDraft((draft) => ({
              ...draft,
              [field]: value,
            }))
          }
          onReferenceAssetDraftChange={(field, value) =>
            setReferenceAssetDraft((draft) => ({ ...draft, [field]: value }))
          }
          onReviewDraftChange={handleReviewDraftChange}
          onResearchDraftChange={(field, value) => setResearchDraft((draft) => ({ ...draft, [field]: value }))}
          onStoryboardDraftChange={(field, value) =>
            setStoryboardDraft((draft) => ({ ...draft, [field]: value }))
          }
          onSubmitBrand={submitBrand}
          onSubmitBriefUnderstanding={submitBriefUnderstanding}
          onSubmitDeliveryReport={submitDeliveryReport}
          onSubmitImagePrompts={submitImagePrompts}
          onSubmitMemory={submitMemory}
          onSubmitMemoryCandidates={submitMemoryCandidates}
          onSubmitProject={submitProject}
          onSubmitReferenceAsset={submitReferenceAsset}
          onSubmitResearchReport={submitResearchReport}
          onSubmitQualityReview={submitQualityReview}
          onSubmitStoryboard={submitStoryboardGeneration}
          onSubmitVideoPrompts={submitVideoPrompts}
          onUpdatePromptPackage={handleUpdatePromptPackage}
          onUpdateDeliveryReport={handleUpdateDeliveryReport}
          onPreviewDeliveryPackage={handlePreviewDeliveryPackage}
          onExportDeliveryPackage={handleExportDeliveryPackage}
          onUpdateShot={handleUpdateShot}
          onUpdateMemoryStatus={handleUpdateMemoryStatus}
          onRegenerateShot={handleRegenerateShot}
          packagePreview={packagePreview}
          previewingDeliveryPackage={previewingDeliveryPackage}
          productUnderstandings={productUnderstandings}
          promptDraft={promptDraft}
          promptPackages={promptPackages}
          projectDraft={projectDraft}
          referenceAssetDraft={referenceAssetDraft}
          researchDraft={researchDraft}
          researchReports={researchReports}
          researchResult={researchResult}
          latestReviewChecks={latestReviewChecks}
          latestReviewSuggestions={latestReviewSuggestions}
          qualityReviews={qualityReviews}
          reviewDraft={reviewDraft}
          regeneratingShotId={regeneratingShotId}
          savingDeliveryReportId={savingDeliveryReportId}
          savingPromptId={savingPromptId}
          savingShotId={savingShotId}
          selectedBrand={selectedBrand}
          selectedProject={selectedProject}
          storyboardDraft={storyboardDraft}
          storyboardResult={storyboardResult}
          storyboards={storyboards}
          generatingStoryboard={generatingStoryboard}
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

function replaceOrAppendReview(reviews: QualityReview[], review: QualityReview): QualityReview[] {
  let replaced = false;
  const next = reviews.map((item) => {
    if (item.id !== review.id) {
      return item;
    }
    replaced = true;
    return review;
  });
  return replaced ? next : [...next, review];
}

function normalizeReviewSuggestions(review: QualityReview): QualityReviewSuggestion[] {
  return Array.isArray(review.suggestions_json)
    ? (review.suggestions_json as QualityReviewSuggestion[])
    : [];
}
