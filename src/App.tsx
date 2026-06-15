import { useEffect, useMemo, useState } from "react";

import {
  createBrand,
  createMemoryEntry,
  createProject,
  formatError,
  healthCheck,
  listAssets,
  listBrands,
  listMemoryEntries,
  listProjectVersions,
  listProjects,
  saveProjectSnapshot,
  updateBrand,
  updateProject,
} from "./api/joiApi";
import { AgentPanel } from "./components/AgentPanel";
import { BrandProjectRail } from "./components/BrandProjectRail";
import { ProjectWorkspace } from "./components/ProjectWorkspace";
import { TopBar } from "./components/TopBar";
import type { Asset, Brand, HealthResponse, MemoryEntry, Project, ProjectVersion } from "./types/joi";

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

export default function App() {
  const [activeTab, setActiveTab] = useState("Overview");
  const [activityLog, setActivityLog] = useState<string[]>([]);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [brandDraft, setBrandDraft] = useState<BrandDraft>(emptyBrandDraft);
  const [brandMode, setBrandMode] = useState<FormMode>("edit");
  const [brands, setBrands] = useState<Brand[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [memoryDraft, setMemoryDraft] = useState<MemoryDraft>(emptyMemoryDraft);
  const [memoryEntries, setMemoryEntries] = useState<MemoryEntry[]>([]);
  const [projectDraft, setProjectDraft] = useState<ProjectDraft>(emptyProjectDraft);
  const [projectMode, setProjectMode] = useState<FormMode>("edit");
  const [projects, setProjects] = useState<Project[]>([]);
  const [savingSnapshot, setSavingSnapshot] = useState(false);
  const [selectedBrandId, setSelectedBrandId] = useState<string | null>(null);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [versions, setVersions] = useState<ProjectVersion[]>([]);

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
      void refreshProjectState(selectedProject.id);
    } else {
      setProjectDraft(emptyProjectDraft);
      setAssets([]);
      setMemoryEntries([]);
      setVersions([]);
    }
  }, [selectedProject]);

  async function loadInitialState() {
    try {
      setError(null);
      const [healthResult, brandList] = await Promise.all([healthCheck(), listBrands()]);
      setHealth(healthResult);
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
      const [assetList, versionList, projectMemory] = await Promise.all([
        listAssets(projectId),
        listProjectVersions(projectId),
        listMemoryEntries({ scope: "project", brand_id: null, project_id: projectId }),
      ]);
      setAssets(assetList);
      setVersions(versionList);
      setMemoryEntries(projectMemory);
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
    setMemoryEntries([]);
    setVersions([]);
    setActiveTab("Overview");
    setActivityLog((entries) => [...entries, "Started a new brand draft."]);
  }

  function startNewProject() {
    setProjectMode("create");
    setSelectedProjectId(null);
    setProjectDraft(emptyProjectDraft);
    setAssets([]);
    setMemoryEntries([]);
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
          onSubmitBrand={submitBrand}
          onSubmitMemory={submitMemory}
          onSubmitProject={submitProject}
          projectDraft={projectDraft}
          selectedBrand={selectedBrand}
          selectedProject={selectedProject}
          versions={versions}
        />

        <AgentPanel activityLog={activityLog} selectedBrand={selectedBrand} selectedProject={selectedProject} />
      </div>

      {error ? (
        <div className="toast" role="alert">
          {error}
        </div>
      ) : null}
    </div>
  );
}
