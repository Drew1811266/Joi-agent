# Joi Agent 0.2 Roadmap

## 0.2 Vision

Joi Agent 0.2 的目标是成为一款可用的服装广告内容工作流 AI Agent。

到 0.2，用户应该可以在 Joi 中完成一次真实服装广告项目的核心链路：

```text
project brief
  -> brand and product understanding
  -> creative direction
  -> storyboard script
  -> image and video prompt packages
  -> review and iteration
  -> delivery report
  -> memory capture
```

0.2 不追求完全自动化替代人类创意团队，而是要做到“人类审核 + Agent 执行”的可用工作台。Joi 应该能主动拆解任务、读取项目上下文、调用本地工具、生成结构化结果，并把用户反馈沉淀成长期记忆。

## Product Scope

0.2 聚焦服装广告内容生产，覆盖短视频和模拍图两类主要输出。

### Primary Users

- 服装品牌内容团队
- 电商视觉和投放团队
- AI 影像创作者
- 短视频广告策划和分镜人员
- 需要批量生成服装广告 prompt 的运营团队

### Primary Workflows

- 输入品牌资料、商品资料和广告 brief。
- 上传或管理项目参考素材。
- 让 Joi 整理品牌理解、商品卖点、目标人群和视觉方向。
- 生成 15 到 30 秒服装广告短视频分镜。
- 为镜头或模拍图生成多平台提示词。
- 根据用户反馈迭代分镜、prompt 和报告。
- 输出项目交付包。
- 将项目决策、品牌偏好和用户偏好写入长期记忆。

### Target Model Adapters

Video:

- Jimeng
- Grok

Image:

- Banana 2
- Jimeng Image
- GPT Image 2

## Version Milestones

0.2 由 0.11 到 0.20 十个小版本推进。每个小版本都应该可以独立验收，并为下一个版本提供稳定基础。

## 0.11 Workspace UI

Goal:

把 Joi 从 placeholder 页面升级为真正的项目工作台。

Scope:

- 三栏式桌面布局。
- 左侧项目导航：品牌、项目、素材、版本。
- 中间工作区：brief、研究、创意方向、分镜、提示词、报告。
- 右侧 Agent 面板：对话、任务计划、操作日志、记忆建议。
- 基础 project CRUD UI。
- brand/project 切换。
- 后端 health、brand、project、memory、snapshot command 的前端接线。

Acceptance Criteria:

- 用户可以在 UI 中创建品牌和项目。
- 用户可以打开项目并看到项目状态。
- 用户可以触发 health check 并看到后端 ready 状态。
- UI 能展示 memory、versions、assets 的基础列表入口。

Non-goals:

- 不做复杂 Agent 任务执行。
- 不做完整 prompt 生成。
- 不做最终视觉设计系统。

## 0.12 Brief And Material Understanding

Goal:

让 Joi 能接收项目 brief、商品资料和品牌资料，并整理出结构化理解。

Scope:

- Brief editor。
- Product info editor。
- Target platform and duration settings。
- Upload/reference material panel。
- Product understanding generation。
- Brand context summary。
- Missing information questions。
- Save generated understanding into local repository。

Acceptance Criteria:

- 用户输入 brief 后，Joi 能生成结构化项目理解。
- 输出包含品牌、商品、人群、卖点、视觉方向、禁忌项和补充问题。
- 结果可编辑、可保存、可进入 snapshot。

Non-goals:

- 不做 web research。
- 不做多轮 Agent planner。

## 0.13 Agent Runtime Integration

Goal:

引入可执行的 Agent runtime，让 Joi 不只是 UI + prompt，而是能规划和执行工作流。

Scope:

- 选定并接入基础 Agent runtime。
- 保留 Joi 自己的数据层、项目模型和命令接口。
- 建立 Joi agent roles:
  - planner
  - researcher
  - storyboard writer
  - prompt adapter
  - reviewer
  - memory curator
- Tool bridge:
  - read project context
  - write structured records
  - create snapshots
  - read/write memory entries
  - export package
- Task run model:
  - plan
  - execute
  - review
  - save

Acceptance Criteria:

- 用户可以让 Joi 为一个项目生成任务计划。
- Agent 可以读取本地项目上下文。
- Agent 可以把输出写入 Joi 的结构化数据层。
- Agent 执行过程有可见日志。

Non-goals:

- 不做完全自主长时间运行。
- 不把 Joi 数据模型替换成外部 Agent 框架的数据模型。

## 0.14 Research And Report Drafting

Goal:

让 Joi 能检索资料、整理结论，并输出服装广告策划所需的研究报告。

Scope:

- Web research tool integration。
- Source collection and citation metadata。
- Fashion advertising reference analysis。
- Competitor and platform style notes。
- Structured research report writer。
- Research report persistence。

Acceptance Criteria:

- 用户可以发起一个项目研究任务。
- Joi 输出包含 findings、sources、rationale 和 creative implications。
- 报告可保存到 project，并可参与后续创意方向生成。

Non-goals:

- 不做无来源的“伪研究”。
- 不做大规模爬取或绕过站点限制。

## 0.15 Practical Long-Term Memory

Goal:

让长期记忆进入真实工作流，而不是只作为数据库表存在。

Scope:

- Memory panel。
- Proposed / accepted / rejected memory workflow。
- Agent 读取 user/brand/project memory。
- Agent 根据用户反馈提出 memory candidates。
- Memory conflict detection。
- Memory source trace。

Acceptance Criteria:

- 用户修改分镜或 prompt 后，Joi 可以提出记忆建议。
- 用户可以接受或拒绝记忆。
- 后续生成会引用 accepted memory。
- brand memory 和 project memory 不混淆。

Non-goals:

- 不写入外部 Agent runtime memory。
- 不做跨用户云同步。

## 0.16 Storyboard Generation

Goal:

让 Joi 能生成可用的 15 到 30 秒服装广告分镜脚本。

Scope:

- Storyboard generation from brief and product understanding。
- Shot planning by duration。
- Shot fields:
  - shot number
  - duration
  - visual description
  - model action
  - garment focus
  - camera movement
  - scene
  - transition
  - subtitle or text suggestion
  - rationale
- Storyboard editing UI。
- Regenerate selected shot。
- Save storyboard and shots。

Acceptance Criteria:

- 用户输入 brief 后，Joi 可以生成完整短视频分镜。
- 总时长符合项目设置。
- 每个镜头都明确展示服装卖点或品牌氛围。
- 用户可以编辑单个镜头并保存版本。

Non-goals:

- 不直接生成视频文件。
- 不做复杂 timeline editing。

## 0.17 Multi-Model Prompt Adapters

Goal:

把分镜和模拍图需求转换为多平台可用 prompt。

Scope:

- Prompt adapter architecture。
- Platform-specific prompt templates。
- Image prompt generation。
- Video prompt generation。
- Negative prompt strategy。
- Prompt package editor。
- Per-shot prompt generation。
- Batch prompt generation。
- Platform validation rules。

Acceptance Criteria:

- 同一个 shot 可以生成 Jimeng 和 Grok 视频 prompt。
- 同一个 image brief 可以生成 Banana 2、Jimeng Image 和 GPT Image 2 prompt。
- Prompt package 绑定 project 和 shot。
- 用户可以编辑、保存、复制 prompt。
- Joi 能指出 prompt 是否缺少主体、场景、动作、镜头、材质、光线或风格信息。

Non-goals:

- 不直接调用外部生成模型 API。
- 不实现模型账号管理。

## 0.18 Reports And Delivery Package

Goal:

让 Joi 可以把项目内容整理成可交付成果。

Scope:

- Project report generator。
- Report sections:
  - project brief summary
  - brand understanding
  - product understanding
  - research findings
  - creative direction
  - storyboard
  - prompt packages
  - asset list
  - version notes
- Markdown export。
- `.joi-project.json` export integration。
- Delivery package preview。

Acceptance Criteria:

- 用户可以一键生成项目交付报告。
- 报告可编辑并可导出。
- 报告引用当前 project 的结构化数据。
- 项目包可以随报告一起交付。

Non-goals:

- 不做 PowerPoint / PDF 高级排版。
- 不做团队权限管理。

## 0.19 Quality Review And Iteration

Goal:

让 Joi 不只生成内容，还能检查内容质量并辅助迭代。

Scope:

- Storyboard review。
- Prompt review。
- Brand consistency review。
- Duration consistency check。
- Shot repetition detection。
- Garment visibility check。
- Platform prompt completeness check。
- Review result as structured checklist。
- Apply suggested revision。

Acceptance Criteria:

- Joi 能指出分镜中的重复、时长不一致和卖点不足。
- Joi 能指出 prompt 缺失的关键字段。
- 用户可以接受某条修改建议并更新对应记录。
- Review 结果可以进入 snapshot。

Non-goals:

- 不做自动发布。
- 不替代人工最终审核。

## 0.20 Usable Beta

Goal:

整合前面能力，形成可用于真实服装广告项目的闭环版本。

End-to-End Workflow:

```text
create project
  -> enter brief
  -> upload/reference materials
  -> generate product understanding
  -> generate creative direction
  -> generate storyboard
  -> generate image/video prompt packages
  -> review and revise
  -> save snapshot
  -> generate delivery report
  -> write accepted memory
  -> export project package
```

Acceptance Criteria:

- 一个真实服装广告项目可以在 Joi 中从 brief 推进到可交付分镜和 prompt 包。
- 用户可以在 UI 中完成主要工作，不需要直接调用测试或命令行。
- Agent 可以读取项目上下文并写入结构化结果。
- 长期记忆可以参与生成并接受用户审核。
- 生成结果可以保存版本、导出报告和导出项目包。
- 失败状态有明确错误信息，不丢失已有项目数据。

## 0.2 Acceptance Criteria

0.2 完成时，Joi 应满足以下标准：

- UI 可以支撑完整服装广告项目工作流。
- Agent 可以规划并执行多步骤内容任务。
- Brief、品牌、商品、素材、研究、创意方向、分镜、prompt 和报告都进入结构化数据层。
- 分镜生成针对 15 到 30 秒服装广告场景可用。
- Prompt adapters 覆盖 Jimeng、Grok、Banana 2、Jimeng Image 和 GPT Image 2。
- Long-term memory 可以被读取、建议、审核和保存。
- Project snapshots 和 `.joi-project.json` export/import 保持可用。
- 真实项目 smoke test 可以产出一套可交付分镜、prompt package 和报告。

## Non-Goals For 0.2

- 不做云端多用户协作。
- 不做账号、权限和团队管理。
- 不做模型生成 API 的账号托管。
- 不做完整视频剪辑器。
- 不做复杂设计资产管理系统。
- 不做移动端应用。
- 不把 Joi 完全改造成通用 Agent 平台。

## Technical Tracks

### Frontend

- React workspace UI。
- Project navigation。
- Structured editors。
- Agent run panel。
- Prompt package editor。
- Review checklist UI。
- Report preview。

### Backend

- Rust command surface hardening。
- Repository split by aggregate。
- Transactional import for full project restore。
- Agent task run persistence。
- Prompt adapter registry。
- Report generation service。

### Agent Runtime

- Runtime selection and integration。
- Tool bridge into Joi commands/services。
- Task planning schema。
- Execution logs。
- Review loop。
- Memory curation.

### Data And Packaging

- Stable project package schema。
- Asset manifest。
- Full import restoration。
- Snapshot compatibility checks。
- Migration strategy for future schema changes。

## Open Decisions

- Which open-source Agent runtime should become Joi's core execution engine?
- Should prompt adapters call model APIs directly or only produce copyable prompts?
- Should report export target Markdown first, then PDF/PPT later?
- How strict should memory acceptance be before memory participates in generation?
- Should full import preserve original IDs or always create new IDs?
- How much of web research should be automated versus user-confirmed?
- What is the first real benchmark project for 0.20 acceptance?

## Benchmark Scenario

The 0.2 benchmark should be a realistic project:

```text
Brand: contemporary womenswear label
Product: new spring outerwear collection
Goal: 15 second launch ad for short-video platforms
Inputs: brand style notes, product selling points, 3 to 5 reference images
Expected output: creative direction, storyboard, image prompts, video prompts, delivery report
```

Joi 0.2 is successful if this benchmark can be completed inside the app with human review and without manual database or command-line operations.
