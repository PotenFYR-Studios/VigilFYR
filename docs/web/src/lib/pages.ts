export type DocMeta = {
  slug: string;
  title: string;
  description: string;
  group: string;
  body: string;
};

import agents from "/workspace_root/agents.md?raw";
import configuration from "/workspace_root/configuration.md?raw";
import daemon from "/workspace_root/daemon.md?raw";
import extensions from "/workspace_root/extensions.md?raw";
import gettingStarted from "/workspace_root/getting-started.md?raw";
import masking from "/workspace_root/masking.md?raw";
import rules from "/workspace_root/rules.md?raw";
import securityModel from "/workspace_root/security-model.md?raw";

export const DOCS: DocMeta[] = [
  { slug: "getting-started", title: "Getting Started", description: "Install, set up hooks and validate the guard.", group: "Get started", body: gettingStarted },
  { slug: "rules", title: "Rules Reference", description: "Layering, verdicts, schema and policy cookbook.", group: "Get started", body: rules },
  { slug: "masking", title: "Masking", description: "Opt-in redaction with provider token coverage.", group: "Core topics", body: masking },
  { slug: "agents", title: "Agents Matrix", description: "Hook, audit and shim support for each agent.", group: "Core topics", body: agents },
  { slug: "configuration", title: "Configuration", description: "Every config key and override workflow.", group: "Core topics", body: configuration },
  { slug: "daemon", title: "Daemon & Live Report", description: "Event bus, TUI, exports and health checks.", group: "Core topics", body: daemon },
  { slug: "extensions", title: "Extensions", description: "Manifests, rules, patterns and event hooks.", group: "Reference", body: extensions },
  { slug: "security-model", title: "Security Model", description: "Threat model and honest limitations.", group: "Reference", body: securityModel },
];

export const DOC_GROUPS = ["Get started", "Core topics", "Reference"] as const;
