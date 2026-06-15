import type { Brand, Project } from "../types/joi";

type BrandProjectRailProps = {
  brands: Brand[];
  projects: Project[];
  selectedBrandId: string | null;
  selectedProjectId: string | null;
  onSelectBrand: (brandId: string) => void;
  onSelectProject: (projectId: string) => void;
  onNewBrand: () => void;
  onNewProject: () => void;
  activeTab: string;
  onSelectTab: (tab: string) => void;
};

const workspaceTabs = [
  "Overview",
  "Brief",
  "Research",
  "Storyboard",
  "Prompts",
  "Review",
  "Delivery",
  "Assets",
  "Memory",
  "Versions",
];

export function BrandProjectRail({
  brands,
  projects,
  selectedBrandId,
  selectedProjectId,
  onSelectBrand,
  onSelectProject,
  onNewBrand,
  onNewProject,
  activeTab,
  onSelectTab,
}: BrandProjectRailProps) {
  return (
    <nav aria-label="Workspace navigation" className="left-rail">
      <section className="rail-section">
        <div className="section-heading">
          <h2>Brands</h2>
          <button onClick={onNewBrand} type="button">
            New Brand
          </button>
        </div>
        <div className="rail-list">
          {brands.length === 0 ? (
            <p className="rail-note">Create a brand to start.</p>
          ) : (
            brands.map((brand) => (
              <button
                aria-current={brand.id === selectedBrandId ? "true" : undefined}
                className={brand.id === selectedBrandId ? "rail-item selected" : "rail-item"}
                key={brand.id}
                onClick={() => onSelectBrand(brand.id)}
                type="button"
              >
                <span>{brand.name}</span>
                <small>{brand.description || "No description"}</small>
              </button>
            ))
          )}
        </div>
      </section>

      <section className="rail-section">
        <div className="section-heading">
          <h2>Projects</h2>
          <button disabled={!selectedBrandId} onClick={onNewProject} type="button">
            New Project
          </button>
        </div>
        <div className="rail-list">
          {projects.length === 0 ? (
            <p className="rail-note">No projects for this brand.</p>
          ) : (
            projects.map((project) => (
              <button
                aria-current={project.id === selectedProjectId ? "true" : undefined}
                className={project.id === selectedProjectId ? "rail-item selected" : "rail-item"}
                key={project.id}
                onClick={() => onSelectProject(project.id)}
                type="button"
              >
                <span>{project.title}</span>
                <small>{project.duration_seconds}s · {project.status}</small>
              </button>
            ))
          )}
        </div>
      </section>

      <section className="rail-section">
        <h2>Workflow</h2>
        <div className="tab-list">
          {workspaceTabs.map((tab) => (
            <button
              className={tab === activeTab ? "tab selected" : "tab"}
              key={tab}
              onClick={() => onSelectTab(tab)}
              type="button"
            >
              {tab}
            </button>
          ))}
        </div>
      </section>
    </nav>
  );
}
