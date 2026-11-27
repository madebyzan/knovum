export interface CodeLanguageOption {
  readonly value: string;
  readonly label: string;
  readonly aliases?: readonly string[];
}

export const CODE_BLOCK_LANGUAGES: readonly CodeLanguageOption[] = [
  { value: "typescript", label: "TypeScript", aliases: ["ts", "tsx"] },
  { value: "javascript", label: "JavaScript", aliases: ["js", "jsx", "mjs"] },
  { value: "python", label: "Python", aliases: ["py", "py3"] },
  { value: "rust", label: "Rust", aliases: ["rs"] },
  { value: "go", label: "Go", aliases: ["golang"] },
  { value: "java", label: "Java" },
  { value: "cpp", label: "C++", aliases: ["c++", "cxx", "cc"] },
  { value: "c", label: "C", aliases: ["h"] },
  { value: "swift", label: "Swift" },
  { value: "sql", label: "SQL", aliases: ["postgres", "sqlite", "mysql"] },
  { value: "bash", label: "Shell / Bash", aliases: ["sh", "zsh", "shell", "terminal"] },
  { value: "json", label: "JSON", aliases: ["jsonc"] },
  { value: "yaml", label: "YAML", aliases: ["yml"] },
  { value: "toml", label: "TOML" },
  { value: "html", label: "HTML", aliases: ["xml", "svg"] },
  { value: "css", label: "CSS", aliases: ["scss", "less"] },
  { value: "vue", label: "Vue", aliases: ["sfc"] },
  { value: "markdown", label: "Markdown", aliases: ["md", "mdx"] },
  { value: "mermaid", label: "Mermaid", aliases: ["mmd", "diagram", "flowchart"] },
  { value: "diff", label: "Diff", aliases: ["patch"] },
  { value: "dockerfile", label: "Dockerfile", aliases: ["docker"] },
];

const LABEL_BY_VALUE = new Map(CODE_BLOCK_LANGUAGES.map((item) => [item.value, item.label]));

export function codeLanguageLabel(value: string): string {
  const normalized = value.trim().toLowerCase();
  return LABEL_BY_VALUE.get(normalized) ?? normalized;
}

export function matchCodeLanguage(item: CodeLanguageOption, rawQuery: string): boolean {
  const query = rawQuery.trim().toLowerCase();
  if (!query) return true;
  if (item.label.toLowerCase().includes(query) || item.value.includes(query)) return true;
  return Boolean(item.aliases?.some((alias) => alias.includes(query)));
}
