export type DocPage = {
  href: string;
  title: string;
  blurb: string;
};

export const pages: DocPage[] = [
  {
    href: "/getting-started",
    title: "Getting Started",
    blurb:
      "Install one-liner, the setup wizard walkthrough, and your first rules check.",
  },
  {
    href: "/rules",
    title: "Rules Reference",
    blurb:
      "Rule schema, precedence and overrides by id, verdict actions, cookbook, built-in rules table.",
  },
  {
    href: "/masking",
    title: "Masking",
    blurb:
      "Opt-in redaction of AWS keys, GitHub PATs, OpenAI keys, JWTs, private keys, env values — 4-char prefix preserved.",
  },
  {
    href: "/agents",
    title: "Agents Matrix",
    blurb:
      "Hook / audit / shim coverage per agent, per-agent modes, and why Vigil is proxy-proof.",
  },
  {
    href: "/extensions",
    title: "Extensions",
    blurb:
      "Drop-in manifest.toml folders adding rules, masking patterns, and event hooks.",
  },
  {
    href: "/configuration",
    title: "Configuration Reference",
    blurb:
      "Full config.toml schema, vigil config get/set, file locations, remote rule sync.",
  },
  {
    href: "/security-model",
    title: "Security Model & Limitations",
    blurb:
      "Threat model, fail-open rationale, verdict contract, and what each layer cannot catch.",
  },
];
